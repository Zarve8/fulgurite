use std::sync::atomic::AtomicPtr;
use borsh::BorshSerialize;
use solana_program::{
    bpf_loader,
    pubkey::Pubkey,
};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProcessInstruction;
use crate::executor::{ExternalPackage, Package, InlinePackage};
use crate::runtime::{
    Receipt,
    Scope,
    Meta,
};
pub use built_in::*;


pub struct Program {
    pub pubkey: Pubkey,
    pub(crate) package: Box<dyn Package>,
    owner: Box<Pubkey>,
    proxy_lamports: Box<u64>,
    proxy_data: Box<Vec<u8>>,
}

impl Program {
    pub fn new(name: &str, pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            package: Box::new(ExternalPackage::new(name)),
            owner: Box::new(bpf_loader::ID.clone()),
            proxy_lamports: Box::new(0x1337),
            proxy_data: Box::new(Vec::new()),
        }
    }

    pub fn inline(entrypoint: ProcessInstruction, pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            package: Box::new(InlinePackage::new(entrypoint)),
            owner: Box::new(bpf_loader::ID.clone()),
            proxy_lamports: Box::new(0x1337),
            proxy_data: Box::new(Vec::new()),
        }
    }

    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }

    pub fn meta(&mut self) -> (Pubkey, Meta) {
        (
            self.pubkey.clone(),
            Meta {
                is_signer: false,
                is_writable: false,
                executable: true,
                lamports: AtomicPtr::new(self.proxy_lamports.as_mut() as *mut u64),
                owner: AtomicPtr::new(self.owner.as_mut() as *mut Pubkey),
                data: AtomicPtr::new(self.proxy_data.as_mut() as *mut Vec<u8>),
                package: AtomicPtr::new((&mut self.package) as *mut Box<dyn Package>),
            }
        )
    }

    pub fn invoke_with_borsh<T: BorshSerialize>(&self, instruction_data: &T, accounts: Vec<(Pubkey, Meta)>) -> Receipt {
        let mut bytes: Vec<u8> = Vec::new();
        instruction_data.serialize(&mut bytes).unwrap();
        self.invoke_with_bytes(bytes.as_slice(), accounts)
    }

    pub fn invoke_with_bytes<'e>(&self, instruction_data: &[u8], accounts: Vec<(Pubkey, Meta)>) -> Receipt {
        let mut scope = Box::new(Scope::new(&accounts));
        let infos: Vec<AccountInfo> = accounts.iter().map(|(key, meta)| meta.as_info(&key)).collect();

        scope.receipt.call_stack.push(self.pubkey.clone());
        scope.receipt.log_program_invoked(&self.pubkey);
        scope.receipt.return_data = None;

        scope.receipt.result = self.package.execute(
            infos.as_slice(),
            instruction_data,
            &self.pubkey,
            scope.clone(),
        );

        match &scope.receipt.result {
            Ok(_) => { scope.receipt.log_program_succeed(); }
            Err(err) => { scope.receipt.log_program_failed(err.clone()); }
        }
        scope.receipt.call_stack.pop();

        scope.receipt
    }
}


mod built_in {
    use solana_program::bpf_loader;
    use crate::executor::SystemProgramPackage;
    use crate::runtime::Program;
    use crate::suit::{ASSOCIATED_PROGRAM_ID, SPL_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_PROGRAM_ID};


    // Built-in Programs
    impl Program {
        pub fn system_program() -> Self {
            Self {
                pubkey: SYSTEM_PROGRAM_ID.clone(),
                package: Box::new(SystemProgramPackage {}),
                owner: Box::new(bpf_loader::ID.clone()),
                proxy_lamports: Box::new(0x1337),
                proxy_data: Box::new(Vec::new()),
            }
        }

        pub fn token_program() -> Self {
            Self::inline(spl_token::entrypoint::entrypoint, SPL_PROGRAM_ID.clone())
        }

        pub fn associated_token_program() -> Self {
            Self::inline(spl_associated_token_account::entrypoint::entrypoint, ASSOCIATED_PROGRAM_ID.clone())
        }

        pub fn sysvar_program() -> Self {
            Self { //TODO research methods
                pubkey: SYSVAR_PROGRAM_ID.clone(),
                package: Box::new(SystemProgramPackage {}),
                owner: Box::new(bpf_loader::ID.clone()),
                proxy_lamports: Box::new(0x1337),
                proxy_data: Box::new(Vec::new()),
            }
        }
    }
}
