use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum ViewerInstruction {
    Log,
    LogData,
    CallAndRead,
    PDASignature,
    ReallocAccount { new_size: usize }
}