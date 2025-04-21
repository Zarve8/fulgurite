use solana_program::rent::{
    DEFAULT_LAMPORTS_PER_BYTE_YEAR,
    DEFAULT_EXEMPTION_THRESHOLD
};

pub fn sol_to_lamports(sol: u64) -> u64 {
    sol * 1000000000
}

pub fn rent_exempt_for_size(size: usize) -> u64 {
    DEFAULT_LAMPORTS_PER_BYTE_YEAR * (DEFAULT_EXEMPTION_THRESHOLD as u64) * (size as u64)
}