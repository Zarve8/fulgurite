extern crate self as solana_program;

pub mod account_info;
pub mod alt_bn128;
pub(crate) mod atomic_u64;
pub mod big_mod_exp;
pub mod blake3;
pub mod borsh;
pub mod borsh0_10;
pub mod borsh0_9;
pub mod bpf_loader;
pub mod debug_account_data;
pub mod decode_error;
pub mod entrypoint;
pub mod hash;
pub mod instruction;
pub mod keccak;
pub mod lamports;
pub mod log;
pub mod native_token;
pub mod program;
pub mod program_error;
pub mod program_option;
pub mod pubkey;
pub mod sanitize;
pub mod secp256k1_recover;
pub mod serde_varint;
pub mod serialize_utils;
pub mod short_vec;
pub mod syscalls;
pub mod system_instruction;
pub mod system_program;
pub mod clock;
pub mod sysvar;
pub mod rent;
pub mod program_utils;
pub mod program_memory;
pub mod program_pack;
pub mod incinerator;


pub use solana_sdk_macro::program_declare_deprecated_id as declare_deprecated_id;

pub use solana_sdk_macro::program_declare_id as declare_id;

pub use solana_sdk_macro::program_pubkey as pubkey;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate solana_frozen_abi_macro;

#[macro_export]
macro_rules! unchecked_div_by_const {
    ($num:expr, $den:expr) => {{
        // Ensure the denominator is compile-time constant
        let _ = [(); ($den - $den) as usize];
        // Compile-time constant integer div-by-zero passes for some reason
        // when invoked from a compilation unit other than that where this
        // macro is defined. Do an explicit zero-check for now. Sorry about the
        // ugly error messages!
        // https://users.rust-lang.org/t/unexpected-behavior-of-compile-time-integer-div-by-zero-check-in-declarative-macro/56718
        let _ = [(); ($den as usize) - 1];
        #[allow(clippy::integer_arithmetic)]
        let quotient = $num / $den;
        quotient
    }};
}
