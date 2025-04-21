use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum DescriptorInstruction {
    ReadToLog,
    WriteData {value: u64},
    CreateAccount,
    CreateAccountPDA,
    TransferSol {amount: u64},
    VerifySigner
}