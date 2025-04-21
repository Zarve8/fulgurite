use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialOrd, PartialEq)]
pub enum PoolInstruction {
    Create,
    ProvideLiquidity {amount_left: u64, amount_right: u64},
    Exchange {amount_left: u64, amount_right: u64},
}