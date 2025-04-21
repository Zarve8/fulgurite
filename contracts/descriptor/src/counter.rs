use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    borsh0_9::try_from_slice_unchecked,
    program_error::ProgramError,
    account_info::AccountInfo,
    system_instruction,
    log::sol_log_data,
    pubkey::Pubkey,
    program::{invoke, invoke_signed},
    rent::Rent,
    sysvar::Sysvar,
    entrypoint::ProgramResult,
    msg
};


#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Counter {
    pub value: u64
}

impl Counter {
    pub fn new() -> Self {
        Self {value: 1}
    }

    pub fn load(ai: &AccountInfo) -> Result<Self, ProgramError> {
        try_from_slice_unchecked::<Self>(&ai.data.borrow())
            .map_err(|_| ProgramError::BorshIoError(String::from("Cannot load Account")))
    }

    pub fn save(&self, ai: &AccountInfo) -> Result<(), ProgramError> {
        self.serialize(&mut *ai.data.borrow_mut())
            .map_err(|_| ProgramError::BorshIoError(String::from("Cannot save Account")))
    }

    pub fn log(&self) {
        msg!("Log Counter");
        sol_log_data(&[&self.value.to_le_bytes()]);
    }

    pub fn size(&self) -> usize {
        8 // value: u64
    }

    pub(crate) fn create<'a>(&self, program_id: &Pubkey, payer: &'a AccountInfo<'a>, ai: &'a AccountInfo<'a>, system_program: &'a AccountInfo<'a>) -> ProgramResult {
        let idx = system_instruction::create_account(
            payer.key,
            ai.key,
            Rent::get()?.minimum_balance(self.size() ),
            self.size() as u64,
            program_id
        );

        invoke(
            &idx,
            &[payer.clone(), ai.clone(), system_program.clone()]
        )
    }

    pub(crate) fn create_pda<'a>(&self, program_id: &Pubkey, payer: &'a AccountInfo<'a>, ai: &'a AccountInfo<'a>, system_program: &'a AccountInfo<'a>) -> ProgramResult {
        let (_key, bump) = Pubkey::find_program_address(&[
            "counter".as_bytes(),
            &payer.key.to_bytes(),
            &program_id.to_bytes(),
        ], program_id);


        let seeds: &[&[&[u8]]] = &[&[
            "counter".as_bytes(),
            &payer.key.to_bytes(),
            &program_id.to_bytes(),
            &[bump]]];

        let idx = system_instruction::create_account(
            payer.key,
            ai.key,
            Rent::get()?.minimum_balance(self.size()),
            self.size() as u64,
            program_id
        );

        invoke_signed(
            &idx,
            &[payer.clone(), ai.clone(), system_program.clone()],
            seeds
        )
    }

    pub(crate) fn realloc<'a>(&self, add_size: usize, payer: &'a AccountInfo<'a>, ai: &'a AccountInfo<'a>, system_program: &'a AccountInfo<'a>) -> Result<usize, ProgramError>{
        let idx = system_instruction::transfer(
            payer.key,
            ai.key,
            Rent::get()?.minimum_balance(add_size),
        );
        invoke(
            &idx,
            &[payer.clone(), ai.clone(), system_program.clone()])?;

        let new_len = ai.data_len() + add_size;
        ai.realloc(new_len, true);
        Ok(new_len)
    }

    pub fn from_bytes(bytes: &mut Vec<u8>) -> Self {
        Self::deserialize(&mut bytes.as_slice()).unwrap()
    }

    pub fn dump_bytes(&self, bytes: &mut Vec<u8>) {
        self.serialize(bytes).unwrap();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        self.dump_bytes(&mut bytes);
        bytes
    }
}



