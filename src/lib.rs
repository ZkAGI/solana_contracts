use arrayref::{array_ref, array_refs};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};
use std::{io::BufWriter, mem};

pub const DATA_VERSION: u8 = 0;
/// Account allocated size
pub const ACCOUNT_ALLOCATION_SIZE: usize = 1024;
/// Initialized flag is 1st byte of data block
const IS_INITIALIZED: usize = 1;
/// Data version (current) is 2nd byte of data block
const DATA_VERSION_ID: usize = 1;
/// Previous content data size (before changing this is equal to current)
pub const PREVIOUS_VERSION_DATA_SIZE: usize = mem::size_of::<GpuRegistryData>();
/// Total space occupied by previous account data
pub const PREVIOUS_ACCOUNT_SPACE: usize =
    IS_INITIALIZED + DATA_VERSION_ID + PREVIOUS_VERSION_DATA_SIZE;
/// Current content data size
pub const CURRENT_VERSION_DATA_SIZE: usize = mem::size_of::<GpuRegistryData>();
/// Total usage for data only
pub const CURRENT_USED_SIZE: usize = IS_INITIALIZED + DATA_VERSION_ID + CURRENT_VERSION_DATA_SIZE;
/// How much of 1024 is used
pub const CURRENT_UNUSED_SIZE: usize = ACCOUNT_ALLOCATION_SIZE - CURRENT_USED_SIZE;
/// Current space used by header (initialized, data version and Content)
pub const ACCOUNT_STATE_SPACE: usize = CURRENT_USED_SIZE + CURRENT_UNUSED_SIZE;

//struct for GPU
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GpuRegistry {
    owner: Pubkey,
    model: String,
}
//impl to create a struct of type GPU Registery
impl GpuRegistry {
    pub fn new(owner: Pubkey, model: String) -> Self {
        Self { owner, model }
    }
}

#[derive(Debug, PartialEq)]
pub enum GpuRegistryInstruction {
    RegisterGpu { model: String },
}

//vector of GPU Regietry Data
#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct GpuRegistryData {
    pub gpu_registries: Vec<GpuRegistry>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Default)]
pub struct ProgramAccountState {
    is_initialized: bool,
    data_version: u8,
    account_data: GpuRegistryData,
}

impl ProgramAccountState {
    pub fn set_initialized(&mut self) {
        self.is_initialized = true;
    }
    pub fn initialized(&self) -> bool {
        self.is_initialized
    }
    pub fn version(&self) -> u8 {
        self.data_version
    }
    pub fn content(&self) -> &GpuRegistryData {
        &self.account_data
    }
    pub fn content_mut(&mut self) -> &mut GpuRegistryData {
        &mut self.account_data
    }
}
#[derive(Debug)]
pub enum ProgramInstruction {
    InitializeAccount,
    RegisterGpu { model: String },
    FailInstruction,
}
#[derive(BorshDeserialize, Debug)]
struct Payload {
    varient: u8,
    model: String,
}
impl ProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let payload = Payload::try_from_slice(input).unwrap();
        match payload.varient {
            0 => Ok(ProgramInstruction::InitializeAccount),
            1 => Ok(ProgramInstruction::RegisterGpu {
                model: (payload.model),
            }),
            _ => Err(ProgramError::InvalidInstructionData),
            // 1 => Ok(GpuRegistryInstruction::RegisterGpu {
            //     model: (payload.model),
            // }),
        }
    }
}
fn conversion_logic(src: &[u8]) -> Result<ProgramAccountState, ProgramError> {
    let past = array_ref![src, 0, PREVIOUS_ACCOUNT_SPACE];
    let (initialized, _, _account_space) = array_refs![
        past,
        IS_INITIALIZED,
        DATA_VERSION_ID,
        PREVIOUS_VERSION_DATA_SIZE
    ];
    Ok(ProgramAccountState {
        is_initialized: initialized[0] != 0u8,
        data_version: DATA_VERSION,
        account_data: GpuRegistryData::default(),
    })
}
impl Sealed for ProgramAccountState {}
impl IsInitialized for ProgramAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
//pack unpack for program account state
impl Pack for ProgramAccountState {
    const LEN: usize = ACCOUNT_STATE_SPACE;
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut bw = BufWriter::new(dst);
        self.serialize(&mut bw).unwrap();
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let initialized = src[0] != 0;
        if initialized {
            if src[1] == DATA_VERSION {
                msg!("Processing consistent data");
                Ok(
                    ProgramAccountState::try_from_slice(array_ref![src, 0, CURRENT_USED_SIZE])
                        .unwrap(),
                )
            } else {
                msg!("Processing backlevel data");
                conversion_logic(src)
            }
        } else {
            msg!("Processing pre-initialized data");
            Ok(ProgramAccountState {
                is_initialized: false,
                data_version: DATA_VERSION,
                account_data: GpuRegistryData::default(),
            })
        }
    }
}

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ProgramInstruction::unpack(instruction_data)?;
    msg!("instruction is {:#?}", instruction);
    match instruction {
        ProgramInstruction::InitializeAccount => initialize_account(accounts),
        ProgramInstruction::RegisterGpu { model } => register_gpu(program_id, accounts, model),
        // GpuRegistryInstruction::RegisterGpu { model } => register_gpu(program_id, accounts, model),
        _ => Ok({
            msg!("Received unknown instruction");
        }),
    };

    Ok(())
}

//account is initialised
fn initialize_account(accounts: &[AccountInfo]) -> ProgramResult {
    msg!("Initialize account");
    let account_info_iter = &mut accounts.iter();
    msg!("account_info_iter is {:#?}", account_info_iter);
    let program_account = next_account_info(account_info_iter)?;
    msg!("Program account is {:#?}", program_account);
    let mut account_data = program_account.data.borrow_mut();
    msg!("account data is {:#?}", &account_data);

    let mut account_state = match ProgramAccountState::unpack_unchecked(&account_data) {
        Ok(state) => state,
        Err(err) => return Err(err.into()), // Convert error to ProgramError
    };
    
    msg!("account state ");
    if account_state.is_initialized() {
        msg!("Account already Initialized");
    } else {
        account_state.set_initialized();
        account_state.content_mut().gpu_registries = Vec::new();
    }
    msg!("Account Initialized");
    ProgramAccountState::pack(account_state, &mut account_data)
}

// Register a GPU
fn register_gpu(program_id: &Pubkey, accounts: &[AccountInfo], model: String) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let program_account = next_account_info(account_info_iter)?;
    let gpuRegisteryAccount = next_account_info(account_info_iter)?;
    let mut account_data = program_account.data.borrow_mut();
    let mut account_state = ProgramAccountState::try_from_slice(&account_data)?;
    msg!("GPU account state is {:#?}", account_state);
    let new_gpu_registry = GpuRegistry::new(*gpuRegisteryAccount.key, model.clone());
    account_state
        .content_mut()
        .gpu_registries
        .push(new_gpu_registry);

    msg!("GPU array is {:#?}", account_state.content());

    ProgramAccountState::pack(account_state, &mut account_data);
    Ok(())
}
