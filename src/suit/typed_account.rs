use std::io::Error;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;

use solana_program::pubkey::Pubkey;
use crate::runtime::{Account, Meta};
use crate::suit::rent_exempt_for_size;
use crate::suit::typed_account::borshed_wrapper::BorshedWrapper;
use crate::suit::typed_account::packed_wrapper::PackedWrapper;


pub struct TypedAccount<T: StructWrapper> {
    pub account: Account,
    modified_bytes: AtomicBool,
    modified_struct: AtomicBool,
    data: AtomicPtr<T>,
}

impl<B: BorshSerialize + BorshDeserialize> TypedAccount<BorshedWrapper<B>> {
    pub fn new_borshed(pubkey: Pubkey, owner: &Pubkey, data: B) -> Self {
        Self::new(pubkey, owner, BorshedWrapper::new(data))
    }

    pub fn from_account_borshed(account: Account) -> Option<Self> {
        match B::deserialize(&mut account.data.as_slice()) {
            Ok(data) => {
                Some(Self {
                    account,
                    modified_bytes: AtomicBool::new(false),
                    modified_struct: AtomicBool::new(true),
                    data: AtomicPtr::new(Box::leak(Box::new(BorshedWrapper::new(data))))
                })
            }
            Err(_) => None
        }
    }
}

impl<P: Pack> TypedAccount<PackedWrapper<P>> {
    pub fn new_packed(pubkey: Pubkey, owner: &Pubkey, data: P) -> Self {
        Self::new(pubkey, owner, PackedWrapper::new(data))
    }

    pub fn from_account_packed(account: Account) -> Option<Self> {
        match P::unpack_unchecked(&mut account.data.as_slice()) {
            Ok(data) => {
                Some(Self {
                    account,
                    modified_bytes: AtomicBool::new(false),
                    modified_struct: AtomicBool::new(true),
                    data: AtomicPtr::new(Box::leak(Box::new(PackedWrapper::new(data))))
                })
            }
            Err(_) => None
        }
    }
}

impl<T: StructWrapper> TypedAccount<T> {
    pub(crate) fn new(pubkey: Pubkey, owner: &Pubkey, data: T) -> Self {
        let mut bytes: Vec<u8> = Vec::new();
        data._serialize(&mut bytes);

        Self {
            account: Account::new(pubkey, rent_exempt_for_size(bytes.len()), owner, bytes),
            modified_bytes: AtomicBool::new(false),
            modified_struct: AtomicBool::new(false),
            data: AtomicPtr::new(Box::leak(Box::new(data))),
        }
    }

    pub fn pubkey(&self) -> &Pubkey {
        self.account.pubkey()
    }

    pub fn lamports(&self) -> u64 {
        *self.account.lamports.as_ref()
    }

    pub fn meta(&mut self, is_signer: bool, is_writable: bool) -> (Pubkey, Meta) {
        if self.modified_struct.load(Ordering::Relaxed) {
            self.copy_struct_to_bytes();
            self.modified_struct.store(false, Ordering::Relaxed);
        }
        if is_writable {
            self.modified_bytes.store(true, Ordering::Relaxed);
        }

        self.account.meta(is_signer, is_writable)
    }

    fn get(&self) -> &T {
        unsafe {
            self.data.load(Ordering::Relaxed)
                .as_ref()
                .unwrap()
        }
    }

    fn get_mut(&self) -> &mut T {
        unsafe {
            self.data.load(Ordering::Relaxed)
                .as_mut()
                .unwrap()
        }
    }

    fn copy_bytes_to_struct(&self) {
        // println!("copy_bytes_to_struct");
        unsafe { // Replacing data mut as non-mut
            let new_struct = Box::new(
                T::_deserialize(
                    &mut self.account.data.as_slice())
            );

            let _ = Box::from_raw(
                self.data.swap(
                    Box::leak(new_struct),
                    Ordering::Relaxed,
                )
            );
        }
    }

    fn copy_struct_to_bytes(&mut self) {
        // println!("copy_struct_to_bytes");
        unsafe {
            self.account.data.clear();
            self.data.load(Ordering::Relaxed)
                .as_ref()
                .unwrap()
                ._serialize(self.account.data.as_mut());
        }
    }
}

// Direct Struct Access
impl<T: StructWrapper> Deref for TypedAccount<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        if self.modified_bytes.load(Ordering::Relaxed) {
            self.copy_bytes_to_struct();
            self.modified_bytes.store(false, Ordering::Relaxed);
        }

        self.get()
    }
}

impl<T: StructWrapper> DerefMut for TypedAccount<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.modified_bytes.load(Ordering::Relaxed) {
            self.copy_bytes_to_struct();
            self.modified_bytes.store(false, Ordering::Relaxed);
        }
        self.modified_struct.store(true, Ordering::Relaxed);

        self.get_mut()
    }
}


// Impl this trait for custom serializable struct
// To use in TypedAccount
pub trait StructWrapper: Sized {
    fn _serialize(&self, dst: &mut Vec<u8>);
    fn _deserialize(input: &[u8]) -> Self;
}


pub(crate) mod borshed_wrapper {
    use std::ops::{Deref, DerefMut};
    use borsh::{BorshDeserialize, BorshSerialize};
    use crate::suit::typed_account::StructWrapper;

    pub(crate) struct BorshedWrapper<T: BorshSerialize + BorshDeserialize> {
        data: T,
    }

    impl<T: BorshSerialize + BorshDeserialize> BorshedWrapper<T> {
        pub fn new(data: T) -> Self {
            Self { data }
        }
    }

    impl<T: BorshSerialize + BorshDeserialize> StructWrapper for BorshedWrapper<T> {
        fn _serialize(&self, dst: &mut Vec<u8>) {
            self.data.serialize(dst)
                .expect("Failed to deserialize struct from bytes")
        }

        fn _deserialize(mut input: &[u8]) -> Self {
            Self {
                data: T::deserialize(&mut input)
                    .expect("Failed to deserialize struct from bytes")
            }
        }
    }

    impl<T: BorshSerialize + BorshDeserialize> Deref for BorshedWrapper<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    impl<T: BorshSerialize + BorshDeserialize> DerefMut for BorshedWrapper<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.data
        }
    }
}


pub(crate) mod packed_wrapper {
    use std::ops::{Deref, DerefMut};
    use solana_program::program_pack::Pack;
    use crate::suit::typed_account::StructWrapper;

    pub(crate) struct PackedWrapper<T: Pack> {
        data: T,
    }

    impl<T: Pack> PackedWrapper<T> {
        pub fn new(data: T) -> Self {
            Self { data }
        }
    }

    impl<T: Pack> StructWrapper for PackedWrapper<T> {
        fn _serialize(&self, dst: &mut Vec<u8>) {
            if dst.len() != T::get_packed_len() {
                dst.resize(T::get_packed_len(), 0)
            }

            self.data.pack_into_slice(dst.as_mut())
        }

        fn _deserialize(mut input: &[u8]) -> Self {
            Self {
                data: T::unpack_from_slice(&mut input)
                    .expect("Failed to deserialize struct from bytes")
            }
        }
    }

    impl<T: Pack> Deref for PackedWrapper<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    impl<T: Pack> DerefMut for PackedWrapper<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.data
        }
    }
}