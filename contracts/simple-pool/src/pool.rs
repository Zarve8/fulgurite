use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{borsh0_9::try_from_slice_unchecked, program_error::ProgramError, account_info::AccountInfo, system_instruction, pubkey::Pubkey, program::{invoke_signed}, rent::Rent, sysvar::Sysvar, entrypoint::ProgramResult, instruction::Instruction, msg};
use crate::error::PoolError;


#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub owner: Pubkey,
    pub mint_left: Pubkey,
    pub mint_right: Pubkey,
    pub vault_left: Pubkey,
    pub vault_right: Pubkey,
    pub amount_left: u64,
    pub amount_right: u64,
    pub bump: u8,
}

impl Pool {
    pub fn new(owner: Pubkey, mint_left: Pubkey, mint_right: Pubkey, vault_left: Pubkey, vault_right: Pubkey) -> Self {
        Self {
            owner,
            mint_left,
            mint_right,
            vault_left,
            vault_right,
            amount_left: 0,
            amount_right: 0,
            bump: 0,
        }
    }

    pub fn load(ai: &AccountInfo) -> Result<Self, ProgramError> {
        try_from_slice_unchecked::<Self>(&ai.data.borrow())
            .map_err(|_| ProgramError::BorshIoError(String::from("Cannot load Pool Account")))
    }

    pub fn save(&self, ai: &AccountInfo) -> Result<(), ProgramError> {
        self.serialize(&mut *ai.data.borrow_mut())
            .map_err(|_| ProgramError::BorshIoError(String::from("Cannot save Pool Account")))
    }

    pub fn display(&self) {
        println!("==== Pool ====");
        println!("\towner {}", self.owner.to_string());
        println!("\tmint left {}", self.mint_left.to_string());
        println!("\tmint right {}", self.mint_right.to_string());
        println!("\tvault left {}", self.vault_left.to_string());
        println!("\tvault right {}", self.vault_right.to_string());
        println!("\tamount left {}", self.amount_left);
        println!("\tamount right {}", self.amount_right);
    }

    pub fn size(&self) -> usize {
        32 + // owner: Pubkey
            32 + // mint_left: Pubkey
            32 + // mint_right: Pubkey
            32 + // vault_right: Pubkey
            32 + // value_left: Pubkey
            8 + // amount_left: u64
            8 +  // amount_right: u64
            1 // bump: u8
    }

    pub fn find_address(program_id: &Pubkey, owner: &Pubkey, mint_left: &Pubkey, mint_right: &Pubkey) -> Pubkey {
        let (key, _bump) = Pubkey::find_program_address(&[
            "pool".as_bytes(),
            &owner.to_bytes(),
            &mint_left.to_bytes(),
            &mint_right.to_bytes(),
            &program_id.to_bytes(),
        ], program_id);

        key
    }

    pub fn find_bump(program_id: &Pubkey, owner: &Pubkey, mint_left: &Pubkey, mint_right: &Pubkey) -> u8 {
        let (_key, bump) = Pubkey::find_program_address(&[
            "pool".as_bytes(),
            &owner.to_bytes(),
            &mint_left.to_bytes(),
            &mint_right.to_bytes(),
            &program_id.to_bytes(),
        ], program_id);

        bump
    }

    pub(crate) fn create_pda<'a>(&mut self,
                                 program_id: &Pubkey,
                                 payer: &'a AccountInfo<'a>,
                                 ai: &'a AccountInfo<'a>,
                                 system_program: &'a AccountInfo<'a>,
    ) -> ProgramResult {
        let (key, bump) = Pubkey::find_program_address(&[
            "pool".as_bytes(),
            &payer.key.to_bytes(),
            &self.mint_left.to_bytes(),
            &self.mint_right.to_bytes(),
            &program_id.to_bytes(),
        ], program_id);

        if !key.eq(ai.key) {
            return Err(PoolError::InvalidPoolAccountPDA.into());
        }
        self.bump = bump;

        let seeds: &[&[&[u8]]] = &[&[
            "pool".as_bytes(),
            &payer.key.to_bytes(),
            &self.mint_left.to_bytes(),
            &self.mint_right.to_bytes(),
            &program_id.to_bytes(),
            &[bump]]];

        let idx = system_instruction::create_account(
            payer.key,
            ai.key,
            Rent::get()?.minimum_balance(self.size()),
            self.size() as u64,
            program_id,
        );

        invoke_signed(
            &idx,
            &[payer.clone(), ai.clone(), system_program.clone()],
            seeds,
        )
    }

    pub(crate) fn invoke_signed<'a>(&self, pool_ai: &'a AccountInfo<'a>, idx: &Instruction, infos: &[AccountInfo<'a>]) -> ProgramResult {
        let seeds: &[&[&[u8]]] = &[&[
            "pool".as_bytes(),
            &self.owner.to_bytes(),
            &self.mint_left.to_bytes(),
            &self.mint_right.to_bytes(),
            &pool_ai.owner.to_bytes(),
            &[self.bump]]];

        invoke_signed(idx, infos, seeds)
    }

    pub(crate) fn validate<'a>(&self, mint_left: &'a AccountInfo<'a>, mint_right: &'a AccountInfo<'a>, vault_left: &'a AccountInfo<'a>, vault_right: &'a AccountInfo<'a>) -> ProgramResult {
        if !self.mint_left.eq(mint_left.key) {
            return Err(PoolError::InvalidMint.into());
        }
        if !self.mint_right.eq(mint_right.key) {
            return Err(PoolError::InvalidMint.into());
        }
        if !self.vault_left.eq(vault_left.key) {
            return Err(PoolError::InvalidVault.into());
        }
        if !self.vault_right.eq(vault_right.key) {
            return Err(PoolError::InvalidVault.into());
        }
        Ok(())
    }
}



