//ZkAGI Solana Contract(test0)
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    rent::Rent,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
    program_pack::{IsInitialized, Pack, Sealed},
    program_pack::Pack
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::HashMap;



#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Provider {
    pub address: Pubkey,
    pub gpu_power: u64,
    pub price_per_hour: u64,
    pub is_active: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserRequest {
    pub address: Pubkey,
    pub provider: Pubkey,
    pub requested_hours: u64,
    pub amount_paid: u64,
    pub fulfilled: bool,
}



#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct IOState {
    pub is_initialized: bool,
    pub providers: HashMap<Pubkey, Provider>,
    pub requests: HashMap<Pubkey, UserRequest>,
}



entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    
    match instruction_data[0] {
        0 => register_provider(program_id, accounts, instruction_data),
        1 => request_gpu_service(program_id, accounts, instruction_data),
        2 => fulfill_request(program_id, accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

pub fn register_provider(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let owner = next_account_info(accounts_iter)?;
    let provider_account = next_account_info(accounts_iter)?;


    let provider_data = Provider::try_from_slice(&instruction_data[1..])?;

    let mut io_state = IOState::try_from_slice(&provider_account.data.borrow())?;
    io_state.providers.insert(owner.key.clone(), provider_data);
    io_state.serialize(&mut &mut provider_account.data.borrow_mut()[..])?;

    Ok(())
}
//some work to be done here
pub fn request_gpu_service(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user = next_account_info(accounts_iter)?;
    let provider_account = next_account_info(accounts_iter)?;


    let request_data = UserRequest::try_from_slice(&instruction_data[1..])?;


    let mut io_state = IOState::try_from_slice(&provider_account.data.borrow())?;
    io_state.requests.insert(user.key.clone(), request_data);
    io_state.serialize(&mut &mut provider_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn fulfill_request(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let provider = next_account_info(accounts_iter)?;
    let request_account = next_account_info(accounts_iter)?;

    
    let mut io_state = IOState::try_from_slice(&request_account.data.borrow())?;
    let request = io_state.requests.get_mut(provider.key).unwrap();
    request.fulfilled = true;
    io_state.serialize(&mut &mut request_account.data.borrow_mut()[..])?;

    Ok(())
}
