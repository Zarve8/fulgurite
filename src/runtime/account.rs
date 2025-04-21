use std::ptr;
use std::sync::atomic::{AtomicPtr};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::runtime::compare_arrays;
use crate::runtime::meta::Meta;
use crate::suit::rent_exempt_for_size;


#[derive(Debug)]
pub struct Account {
    pub pubkey: Pubkey,
    pub lamports: Box<u64>,
    pub owner: Box<Pubkey>,
    pub data: Box<Vec<u8>>,
}

impl Account {
    pub fn new(pubkey: Pubkey, lamports: u64, owner: &Pubkey, data: Vec<u8>) -> Self {
        Self {
            pubkey,
            lamports: Box::new(lamports),
            owner: Box::new(owner.clone()),
            data: Box::new(data)
        }
    }

    pub fn meta(&mut self, is_signer: bool, is_writable: bool) -> (Pubkey, Meta) {
        (self.pubkey.clone(),
         Meta {
             is_signer,
             is_writable,
             executable: false,
             lamports: AtomicPtr::new(self.lamports.as_mut() as *mut u64),
             owner: AtomicPtr::new(self.owner.as_mut() as *mut Pubkey),
             data: AtomicPtr::new(self.data.as_mut() as *mut Vec<u8>),
             package: AtomicPtr::new(ptr::null_mut()),
         })
    }

    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }
}

// ++++++++ Suit Methods +++++++
impl Account {
    pub fn expect_balance(&self, balance: u64) {
        if *self.lamports.as_ref() != balance {
            println!("{} balance not matches {} != {}", self.pubkey.to_string(), *self.lamports.as_ref(), balance);
            assert_eq!(*self.lamports.as_ref(), balance);
        }
    }

    pub fn expect_owner(&self, owner: &Pubkey) {
        if !owner.eq(self.owner.as_ref()) {
            println!("{} owner not matches {} != {}", self.pubkey.to_string(), self.owner.to_string(), owner.to_string());
            assert!(owner.eq(self.owner.as_ref()));
        }
    }

    pub fn expect_data<T: BorshSerialize>(&self, data: &T) {
        let mut bytes: Vec<u8> = Vec::new();
        data.serialize(&mut bytes);
        self.expect_bytes(&bytes);
    }

    pub fn expect_bytes(&self, bytes: &[u8]) {
        if !compare_arrays(self.data.as_slice(), bytes) {
            println!("Data not match {:?} != {:?}", self.data.as_slice(), bytes);
            assert_eq!(self.data.as_slice(), bytes);
        }
    }

    pub fn new_with_borsh_data<T: BorshSerialize>(pubkey: Pubkey, owner: &Pubkey, data: &T) -> Self {
        let mut bytes: Vec<u8> = Vec::new();
        data.serialize(&mut bytes).expect("Failed to serialze data");
        Account::new(pubkey, rent_exempt_for_size(bytes.len()), owner, bytes)
    }

    pub fn borsh_serialize<T: BorshSerialize>(&mut self, data: &T) {
        data.serialize(self.data.as_mut())
            .expect("Failed to serialze data");
    }

    pub fn borsh_deserialize<T: BorshDeserialize>(&self) -> Option<T> {
        match T::deserialize(&mut self.data.as_slice()) {
            Ok(t) => Some(t),
            Err(_) => None
        }
    }
}
