use std::collections::HashMap;
use std::sync::atomic::Ordering::Relaxed;
use solana_program::{
    pubkey::Pubkey,
    instruction::AccountMeta,
    account_info::AccountInfo
};
use crate::executor::Package;
use crate::runtime::{
    ClusterSettings,
    meta::Meta,
    Receipt
};


pub struct Scope {
    pub(crate) metas: HashMap<Pubkey, Meta>,
    pub(crate) receipt: Receipt,
    pub(crate) settings: ClusterSettings,
}

impl Scope {
    pub fn new(metas: &Vec<(Pubkey, Meta)>) -> Self {
        Self {
            metas: metas.iter().map(|(key, meta)| (key.clone(), meta.clone())).collect(),
            receipt: Receipt::new(),
            settings: ClusterSettings::new(),
        }
    }

    pub fn clone(&self) -> Box<Self> {
        unsafe {
            Box::from_raw(self as *const Scope as *mut Scope)
        }
    }

    pub fn replicate_info<'a>(&self, account_info: &'a AccountInfo<'a>, account_meta: &'a AccountMeta, copy: bool) -> AccountInfo<'a> {
        let meta = self.metas.get(&account_meta.pubkey)
            .expect(&format!("Undefined Account {}", account_meta.pubkey.to_string()));

        if copy {
            meta.as_info_with_meta(account_info.key, account_meta.is_signer, account_meta.is_writable)
        }
        else {
            meta.shallow_copy_info(account_info, account_meta.is_signer, account_meta.is_writable)
        }
    }

    pub fn get_info<'a>(&self, pubkey: &'a Pubkey) -> AccountInfo<'a> {
        let meta = self.metas.get(pubkey)
            .expect(&format!("Undefined Account {}", pubkey.to_string()));

        meta.as_info(pubkey)
    }

    pub fn get_package<'a>(&self, pubkey: &Pubkey) -> &'a Box<dyn Package> {
        let meta = self.metas.get(pubkey)
            .expect(&format!("Undefined Account {}", pubkey.to_string()));
        if !meta.executable {
            panic!("Trying to call non-executable {}", pubkey.to_string());
        }
        meta.as_package()
    }
}

impl Into<Receipt> for Scope {
    fn into(self) -> Receipt {
        self.receipt
    }
}
