use solana_program::pubkey::Pubkey;
use crate::runtime::*;
use descriptor_contract::{
    instruction::DescriptorInstruction,
    counter::Counter,
};
use viewer_contract::{
    instruction::ViewerInstruction
};


#[test]
fn test_log() {
    let system_program = Program::system_program();
    let viewer = Program::inline(viewer_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());

    let receipt = viewer.invoke_with_borsh(
        &ViewerInstruction::Log,
        vec![
            payer_ai.meta(true, true),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    receipt.expect_log("Simple log");
}

#[test]
fn test_log_data() {
    let system_program = Program::system_program();
    let viewer = Program::inline(viewer_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());

    let receipt = viewer.invoke_with_borsh(
        &ViewerInstruction::LogData,
        vec![
            payer_ai.meta(true, true),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    receipt.expect_data(&viewer.pubkey, &[&[1, 2, 3, 4]]);
}

#[test]
fn test_account_read() {
    let system_program = Program::system_program();
    let descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut counter_ai = Account::new(Pubkey::new_rand(), 1000, &descriptor.pubkey, Vec::from(u64::to_le_bytes(17)));

    let receipt = descriptor.invoke_with_borsh(
        &DescriptorInstruction::ReadToLog,
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(false, false),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    receipt.expect_data(&descriptor.pubkey, &[&u64::to_le_bytes(17)]);
}

#[test]
fn test_account_write() {
    let system_program = Program::system_program();
    let descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut counter_ai = Account::new(Pubkey::new_rand(), 1000, &descriptor.pubkey, Vec::from(u64::to_le_bytes(17)));

    let receipt = descriptor.invoke_with_borsh(
        &DescriptorInstruction::WriteData { value: 999 },
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(false, true),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    let counter = Counter::from_bytes(&mut counter_ai.data);
    assert_eq!(counter.value, 999);
}

#[test]
fn test_account_create() {
    let mut system_program = Program::system_program();
    let descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut counter_ai = Account::new(Pubkey::new_rand(), 0, &system_program.pubkey, Vec::new());

    let receipt = descriptor.invoke_with_borsh(
        &DescriptorInstruction::CreateAccount,
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(true, true),
            system_program.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
}

#[test]
fn test_call_and_read() {
    let mut system_program = Program::system_program();
    let mut descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let viewer = Program::inline(viewer_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut counter_ai = Account::new(Pubkey::new_rand(), 1000, &descriptor.pubkey, Vec::from(u64::to_le_bytes(17)));


    let receipt = viewer.invoke_with_borsh(
        &ViewerInstruction::CallAndRead,
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(false, true),
            descriptor.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    let counter = Counter::from_bytes(&mut counter_ai.data);
    assert_eq!(counter.value, 99);
}

#[test]
fn test_transfer_sol() {
    let mut system_program = Program::system_program();
    let descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut counter_ai = Account::new(Pubkey::new_rand(), 1000, &system_program.pubkey, Vec::new());


    let receipt = descriptor.invoke_with_borsh(
        &DescriptorInstruction::TransferSol { amount: 12345678 },
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(false, true),
            system_program.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
}

#[test]
fn test_create_account_pda() {
    let mut system_program = Program::system_program();
    let descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());

    let (counter_key, bump) = Pubkey::find_program_address(&[
        "counter".as_bytes(),
        &payer_ai.pubkey.to_bytes(),
        &descriptor.pubkey.to_bytes(),
    ], &descriptor.pubkey);

    let mut counter_ai = Account::new(counter_key, 0, &system_program.pubkey, Vec::new());

    let receipt = descriptor.invoke_with_borsh(
        &DescriptorInstruction::CreateAccountPDA,
        vec![
            payer_ai.meta(true, true),
            counter_ai.meta(false, true),
            system_program.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
}

#[test]
fn test_account_singed_pda() {
    let mut system_program = Program::system_program();
    let mut descriptor = Program::inline(descriptor_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let viewer = Program::inline(viewer_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());

    let (pda_key, _bump) = Pubkey::find_program_address(&[
        "viewer".as_bytes(),
        &payer_ai.pubkey.to_bytes(),
        &viewer.pubkey.to_bytes(),
    ], &viewer.pubkey);

    let mut pda_ai = Account::new(pda_key, 0, &system_program.pubkey, Vec::new());

    let receipt = viewer.invoke_with_borsh(
        &ViewerInstruction::PDASignature,
        vec![
            payer_ai.meta(true, true),
            pda_ai.meta(false, false),
            descriptor.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
}

#[test]
fn test_account_realloc() {
    let mut system_program = Program::system_program();
    let viewer = Program::inline(viewer_contract::entrypoint::entrypoint, Pubkey::new_rand());
    let mut payer_ai = Account::new(Pubkey::new_rand(), 1000000000, &system_program.pubkey, Vec::new());
    let mut account_ai = Account::new(Pubkey::new_rand(), 0, &system_program.pubkey, vec![1, 2, 3, 4]);

    let receipt = viewer.invoke_with_borsh(
        &ViewerInstruction::ReallocAccount { new_size: 8 },
        vec![
            payer_ai.meta(true, true),
            account_ai.meta(false, true),
            system_program.meta(),
        ],
    );

    println!("{:?}", receipt);
    receipt.expect_ok();
    account_ai.expect_bytes(&[1, 2, 3, 4, 0, 0, 0, 0]);
}

