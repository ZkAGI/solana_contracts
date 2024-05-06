use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GpuRegistry {
    owner: Pubkey,
    model: String,
}

impl GpuRegistry {
    pub fn new(owner: Pubkey, model: String) -> Self {
        Self { owner, model }
    }
}

#[derive(BorshDeserialize, Debug)]
struct Payload {
    arg1: String,
}

pub mod Gpuinstruction;
use crate::Gpuinstruction::GpuRegistryInstruction;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    let instruction = Payload::try_from_slice(instruction_data).unwrap();

    // Match the instruction and perform the necessary actions
    // match instruction {
    //     GpuRegistryInstruction::RegisterGpu { model } => {
    //         register_gpu(program_id, accounts, model)?;
    //     }
    // }
    msg!("the instruction is {:#?}", instruction);

    Ok(())
}

// Register a GPU
fn register_gpu(_program_id: &Pubkey, accounts: &[AccountInfo], model: String) -> ProgramResult {
    // Extract accounts
    let accounts_iter = &mut accounts.iter();
    let gpu_registry_account = next_account_info(accounts_iter)?;

    // Update the GPU registry account data with the new owner and model
    let updated_gpu_registry = GpuRegistry::new(*gpu_registry_account.key, model);
    let mut gpu_registry_data = gpu_registry_account.try_borrow_mut_data()?;
    updated_gpu_registry.serialize(&mut gpu_registry_data.as_mut())?;

    Ok(())
}
