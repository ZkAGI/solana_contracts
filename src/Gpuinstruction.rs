use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
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

#[derive(Debug, PartialEq)]
pub enum GpuRegistryInstruction {
    RegisterGpu { model: String },
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GpuRegistryAccount {
    gpu_registries: Vec<GpuRegistry>,
}
impl GpuRegistryAccount {
    pub fn new() -> Self {
        Self {
            gpu_registries: Vec::new(),
        }
    }

    pub fn add_gpu_registry(&mut self, registry: GpuRegistry) {
        self.gpu_registries.push(registry);
    }

    pub fn get_gpu_registries(&self) -> &Vec<GpuRegistry> {
        &self.gpu_registries
    }
}

#[derive(BorshDeserialize, Debug)]
struct Payload {
    model: String,
}

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Parse instruction data
    let instruction = Payload::try_from_slice(instruction_data).unwrap();
    match instruction {
        Payload { model } => {
            register_gpu(program_id, accounts, model);
        }
    }

    Ok(())
}

// Register a GPU
fn register_gpu(program_id: &Pubkey, accounts: &[AccountInfo], model: String) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let gpu_registry_account = next_account_info(accounts_iter)?;

    let mut gpu_registry_data = gpu_registry_account.try_borrow_mut_data()?;
    msg!("gpu regietery data is {:#?}", gpu_registry_data);
    let mut gpu_registry: GpuRegistryAccount = if gpu_registry_data.len() > 0 {
        GpuRegistryAccount::try_from_slice(&gpu_registry_data)
            .map_err(|_| ProgramError::InvalidAccountData)?
    } else {
        GpuRegistryAccount::new()
    };

    let new_gpu_registry = GpuRegistry::new(*gpu_registry_account.key, model.clone());
    gpu_registry.add_gpu_registry(new_gpu_registry);
    msg!("{:#?}", gpu_registry);

    gpu_registry.serialize(&mut gpu_registry_data.as_mut())?;

    Ok(())
}
