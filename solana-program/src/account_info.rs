use std::sync::atomic::Ordering;
use {
    crate::{
        clock::Epoch, debug_account_data::*, entrypoint::MAX_PERMITTED_DATA_INCREASE,
        program_error::ProgramError, pubkey::Pubkey,
    },
    std::{
        cell::{Ref, RefCell, RefMut},
        fmt,
        rc::Rc,
        slice::from_raw_parts_mut,
    },
};
use crate::instruction::AccountMeta;
use crate::syscalls;


#[derive(Clone)]
#[repr(C)]
pub struct AccountInfo<'a> {
    pub key: &'a Pubkey,
    pub lamports: Rc<RefCell<&'a mut u64>>,
    pub data: Rc<RefCell<&'a mut [u8]>>,
    pub owner: &'a Pubkey,
    pub rent_epoch: Epoch,
    pub is_signer: bool,
    pub is_writable: bool,
    pub executable: bool,
}

impl<'a> fmt::Debug for AccountInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("AccountInfo");

        f.field("key", &self.key)
            .field("owner", &self.owner)
            .field("is_signer", &self.is_signer)
            .field("is_writable", &self.is_writable)
            .field("executable", &self.executable)
            .field("rent_epoch", &self.rent_epoch)
            .field("lamports", &self.lamports())
            .field("data.len", &self.data_len());
        debug_account_data(&self.data.borrow(), &mut f);

        f.finish_non_exhaustive()
    }
}

impl<'a> AccountInfo<'a> {
    pub fn signer_key(&self) -> Option<&Pubkey> {
        if self.is_signer {
            Some(self.key)
        } else {
            None
        }
    }

    pub fn unsigned_key(&self) -> &Pubkey {
        self.key
    }

    pub fn lamports(&self) -> u64 {
        **self.lamports.borrow()
    }

    pub fn try_lamports(&self) -> Result<u64, ProgramError> {
        Ok(**self.try_borrow_lamports()?)
    }

    pub unsafe fn original_data_len(&self) -> usize {
        let key_ptr = self.key as *const _ as *const u8;
        let original_data_len_ptr = key_ptr.offset(-4) as *const u32;
        *original_data_len_ptr as usize
    }

    pub fn data_len(&self) -> usize {
        self.data.borrow().len()
    }

    pub fn try_data_len(&self) -> Result<usize, ProgramError> {
        Ok(self.try_borrow_data()?.len())
    }

    pub fn data_is_empty(&self) -> bool {
        self.data.borrow().is_empty()
    }

    pub fn try_data_is_empty(&self) -> Result<bool, ProgramError> {
        Ok(self.try_borrow_data()?.is_empty())
    }

    pub fn try_borrow_lamports(&self) -> Result<Ref<&mut u64>, ProgramError> {
        self.lamports
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    pub fn try_borrow_mut_lamports(&self) -> Result<RefMut<&'a mut u64>, ProgramError> {
        self.lamports
            .try_borrow_mut()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    pub fn try_borrow_data(&self) -> Result<Ref<&mut [u8]>, ProgramError> {
        self.data
            .try_borrow()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    pub fn try_borrow_mut_data(&self) -> Result<RefMut<&'a mut [u8]>, ProgramError> {
        self.data
            .try_borrow_mut()
            .map_err(|_| ProgramError::AccountBorrowFailed)
    }

    pub fn realloc(&self, new_len: usize, zero_init: bool) -> Result<(), ProgramError> {
        let old_len = self.data_len();

        // Return early if length hasn't changed
        if new_len == old_len {
            return Ok(());
        }

        if new_len.saturating_sub(old_len) > MAX_PERMITTED_DATA_INCREASE {
            return Err(ProgramError::InvalidRealloc);
        }

        let min_lamports = unsafe {
            let mut scope = syscalls!();
            let data_ptr = scope.get_data_ptr(self.key);
            let vec = data_ptr.load(Ordering::Relaxed).as_mut().unwrap();

            if old_len > new_len {
                vec.drain(new_len..old_len);
            }
            else {
                vec.extend(vec![0; new_len - old_len].iter());
            }

            scope.set_data(self, vec.clone());
            scope.rent_exempt_for_size(new_len)
        };

        if self.lamports() < min_lamports {
            return Err(ProgramError::AccountNotRentExempt);
        }

        Ok(())
    }

    pub fn assign(&self, new_owner: &Pubkey) {
        // Set the non-mut owner field
        unsafe {
            let pubkey_ptr = self.owner as *const Pubkey as *mut Pubkey;
            std::ptr::replace(pubkey_ptr, new_owner.clone());
        }
    }

    pub fn new(
        key: &'a Pubkey,
        is_signer: bool,
        is_writable: bool,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
        executable: bool,
        rent_epoch: Epoch,
    ) -> Self {
        Self {
            key,
            is_signer,
            is_writable,
            lamports: Rc::new(RefCell::new(lamports)),
            data: Rc::new(RefCell::new(data)),
            owner,
            executable,
            rent_epoch,
        }
    }

    pub fn deserialize_data<T: serde::de::DeserializeOwned>(&self) -> Result<T, bincode::Error> {
        bincode::deserialize(&self.data.borrow())
    }

    pub fn serialize_data<T: serde::Serialize>(&self, state: &T) -> Result<(), bincode::Error> {
        if bincode::serialized_size(state)? > self.data_len() as u64 {
            return Err(Box::new(bincode::ErrorKind::SizeLimit));
        }
        bincode::serialize_into(&mut self.data.borrow_mut()[..], state)
    }
}

#[cfg(feature = "fulgurite")]
impl<'a> AccountInfo<'a> {
    pub fn assign_meta(&self, meta: &AccountMeta) -> Self {
        Self {
            key: self.key,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
            lamports: self.lamports.clone(),
            data: self.data.clone(),
            owner: self.owner,
            executable: self.executable,
            rent_epoch: self.rent_epoch
        }
    }
}

pub trait IntoAccountInfo<'a> {
    fn into_account_info(self) -> AccountInfo<'a>;
}

impl<'a, T: IntoAccountInfo<'a>> From<T> for AccountInfo<'a> {
    fn from(src: T) -> Self {
        src.into_account_info()
    }
}

pub trait Account {
    fn get(&mut self) -> (&mut u64, &mut [u8], &Pubkey, bool, Epoch);
}


impl<'a, T: Account> IntoAccountInfo<'a> for (&'a Pubkey, &'a mut T) {
    fn into_account_info(self) -> AccountInfo<'a> {
        let (key, account) = self;
        let (lamports, data, owner, executable, rent_epoch) = account.get();
        AccountInfo::new(
            key, false, false, lamports, data, owner, executable, rent_epoch,
        )
    }
}

impl<'a, T: Account> IntoAccountInfo<'a> for (&'a Pubkey, bool, &'a mut T) {
    fn into_account_info(self) -> AccountInfo<'a> {
        let (key, is_signer, account) = self;
        let (lamports, data, owner, executable, rent_epoch) = account.get();
        AccountInfo::new(
            key, is_signer, false, lamports, data, owner, executable, rent_epoch,
        )
    }
}


impl<'a, T: Account> IntoAccountInfo<'a> for &'a mut (Pubkey, T) {
    fn into_account_info(self) -> AccountInfo<'a> {
        let (ref key, account) = self;
        let (lamports, data, owner, executable, rent_epoch) = account.get();
        AccountInfo::new(
            key, false, false, lamports, data, owner, executable, rent_epoch,
        )
    }
}


pub fn next_account_info<'a, 'b, I: Iterator<Item=&'a AccountInfo<'b>>>(
    iter: &mut I,
) -> Result<I::Item, ProgramError> {
    iter.next().ok_or(ProgramError::NotEnoughAccountKeys)
}


pub fn next_account_infos<'a, 'b: 'a>(
    iter: &mut std::slice::Iter<'a, AccountInfo<'b>>,
    count: usize,
) -> Result<&'a [AccountInfo<'b>], ProgramError> {
    let accounts = iter.as_slice();
    if accounts.len() < count {
        return Err(ProgramError::NotEnoughAccountKeys);
    }
    let (accounts, remaining) = accounts.split_at(count);
    *iter = remaining.iter();
    Ok(accounts)
}

impl<'a> AsRef<AccountInfo<'a>> for AccountInfo<'a> {
    fn as_ref(&self) -> &AccountInfo<'a> {
        self
    }
}