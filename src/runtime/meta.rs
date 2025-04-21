use std::cell::{RefCell, RefMut};
use std::mem::forget;
use std::rc::Rc;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::atomic::Ordering::Relaxed;
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use crate::executor::Package;


#[derive(Debug)]
pub struct Meta {
    pub(crate) is_signer: bool,
    pub(crate) is_writable: bool,
    pub(crate) executable: bool,
    pub(crate) lamports: AtomicPtr<u64>, // Owned by Account or Program
    pub(crate) owner: AtomicPtr<Pubkey>, // Owned by Account or Program
    pub(crate) data: AtomicPtr<Vec<u8>>, // Owned by Account or Program
    pub(crate) package: AtomicPtr<Box<dyn Package>>, // Owned by Program or Null
}

impl Meta {
    pub fn as_info<'a>(&self, pubkey: &'a Pubkey) -> AccountInfo<'a> {
        self.as_info_with_meta(pubkey, self.is_signer, self.is_writable)
    }

    pub fn as_info_with_meta<'a>(&self, pubkey: &'a Pubkey, is_signer: bool, is_writable: bool) -> AccountInfo<'a> {
        unsafe {
            AccountInfo {
                key: pubkey,
                lamports: Rc::new(RefCell::new(self.lamports.load(Relaxed).as_mut().unwrap())),
                data: Rc::new(RefCell::new(self.data.load(Relaxed).as_mut().unwrap())),
                owner: self.owner.load(Relaxed).as_ref().unwrap(),
                rent_epoch: 0,
                is_signer,
                is_writable,
                executable: self.executable,
            }
        }
    }

    pub fn shallow_copy_info<'a>(&self, info: &'a AccountInfo<'a>, is_signer: bool, is_writable: bool) -> AccountInfo<'a> {
        unsafe {
            AccountInfo {
                key: info.key,
                lamports: Rc::clone(&info.lamports),
                data: Rc::clone(&info.data),
                owner: info.owner,
                rent_epoch: 0,
                is_signer,
                is_writable,
                executable: self.executable,
            }
        }
    }

    pub fn as_package<'a>(&self) -> &'a Box<dyn Package> {
        unsafe {
            self.package.load(Relaxed).as_mut().unwrap()
        }
    }

    pub fn set_owner(&mut self, owner: &Pubkey) {
        let value = self.owner.load(Relaxed);
        unsafe { *value = owner.clone() }
    }

    pub fn set_lamports(&mut self, lamports: u64) {
        let value = self.lamports.load(Relaxed);
        unsafe { *value = lamports; }
    }

    pub fn set_data(&mut self, info: &AccountInfo, data: Vec<u8>) {
        unsafe {
            let value = unsafe { self.data.load(Relaxed).as_mut().unwrap() };
            *value = data;

            *info.data.borrow_mut() = value;
        }
    }

    pub fn get_lamports(&self) -> u64 {
        let value = self.lamports.load(Relaxed);
        unsafe { *value }
    }

    pub fn get_owner(&self) -> &Pubkey {
        let value = self.owner.load(Relaxed);
        unsafe {value.as_mut().unwrap()}
    }

    pub fn get_data(&self) -> &Vec<u8> {
        let value = self.data.load(Relaxed);
        unsafe {value.as_mut().unwrap()}
    }
    
    pub fn get_data_ptr(&self) -> AtomicPtr<Vec<u8>> {
        AtomicPtr::new(self.data.load(Ordering::Relaxed))
    }
}

impl ToString for Meta {
    fn to_string(&self) -> String {
        format!("is_signer: {}, is_writable: {}, executable: {}, owner: {:?}, lamports: {}, data: {:?}",
            self.is_signer,
            self.is_writable,
            self.executable,
            self.get_owner(),
            self.get_lamports(),
            self.get_data()
        )
    }
}

impl Clone for Meta {
    fn clone(&self) -> Self {
        Self {
            is_signer: self.is_signer,
            is_writable: self.is_writable,
            executable: self.executable,
            lamports: AtomicPtr::new(self.lamports.load(Ordering::Relaxed)),
            owner: AtomicPtr::new(self.owner.load(Ordering::Relaxed)),
            data: AtomicPtr::new(self.data.load(Ordering::Relaxed)),
            package: AtomicPtr::new(self.package.load(Ordering::Relaxed))
        }
    }
}