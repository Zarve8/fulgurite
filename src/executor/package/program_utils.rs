use solana_program::instruction::InstructionError;
use solana_program::program_error::ProgramError;


pub const PACKET_DATA_SIZE: usize = 1280 - 40 - 8;

pub fn limited_deserialize<T>(instruction_data: &[u8]) -> Result<T, InstructionError>
    where
        T: serde::de::DeserializeOwned,
{
    solana_program::program_utils::limited_deserialize(
        instruction_data,
        PACKET_DATA_SIZE as u64,
    )
}

pub fn convert_instruction_error(err: InstructionError) -> ProgramError {
    println!("System Instruction Error {:?}", err);
    match ProgramError::try_from(err) {
        Ok(err) => err,
        Err(_) => ProgramError::Custom(0x0)
    }
}