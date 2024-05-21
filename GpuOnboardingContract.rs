use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program::invoke;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh0_10::try_from_slice_unchecked,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use std::convert::TryInto;
use thiserror::Error;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}

#[derive(Debug)]
pub enum ProgramInstruction {
    InitStorage {},
    CreateEntry { model: String },
}

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ProgramInstruction::unpack(instruction_data).unwrap();
        match instruction {
            ProgramInstruction::InitStorage {} => {
                msg!("Instruction: InitStorage");
                Self::process_init_storage(accounts, program_id)
            }
            ProgramInstruction::CreateEntry { model } => {
                msg!("Instruction: CreateEntry");
                Self::process_Entry(accounts, model, program_id)
            }
        }
    }

    //push entry
    fn process_Entry(
        accounts: &[AccountInfo],
        model: String,
        program_id: &Pubkey,
    ) -> ProgramResult {
        if model.len() <= 0 {
            return Err(PError::InvalidGpuData.into());
        }
        let account_info_iter = &mut accounts.iter();
        let owner_account = next_account_info(account_info_iter)?;
        let storage_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        if !owner_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (storage_pda, _storage_bump) = Pubkey::find_program_address(
            &[b"storagePool".as_ref(), owner_account.key.as_ref()],
            program_id,
        );
        if storage_pda != *storage_account.key
            || !storage_account.is_writable
            || storage_account.data_is_empty()
        {
            return Err(PError::InvalidStorageAccount.into());
        }
        let entry = Entry {
            owner: *owner_account.key,
            storage: *storage_account.key,
            model: model.clone(),
        };
        let mut storage_state =
            try_from_slice_unchecked::<Storage>(&storage_account.data.borrow_mut())?;
        let vec_length = storage_state.gpustorage.len() + 1;
        let new_size = Storage::calculate_len(vec_length, model.clone());
        let rent = Rent::get()?;
        let new_minimum_balance = rent.minimum_balance(new_size);
        let lamports_diff = new_minimum_balance.saturating_sub(storage_account.lamports());
        invoke(
            &system_instruction::transfer(owner_account.key, storage_account.key, lamports_diff),
            &[
                owner_account.clone(),
                storage_account.clone(),
                system_program.clone(),
            ],
        )?;
        storage_account.realloc(new_size, false)?;
        storage_state.gpustorage.push(entry);
        storage_state.serialize(&mut &mut storage_account.data.borrow_mut()[..])?;
        msg!("entry data is {:#?}", storage_state.gpustorage[14]);
        Ok(())
    }

    // init storage done
    fn process_init_storage(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let authority_account = next_account_info(account_info_iter)?;
        let storage_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let (storage_pda, storage_bump) = Pubkey::find_program_address(
            &[b"storagePool".as_ref(), authority_account.key.as_ref()],
            program_id,
        );
        if storage_pda != *storage_account.key {
            return Err(PError::InvalidStorageAccount.into());
        }
        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(Storage::BASE_LEN);
        let create_storage_pda_ix = &system_instruction::create_account(
            authority_account.key,
            storage_account.key,
            rent_lamports,
            Storage::BASE_LEN.try_into().unwrap(),
            program_id,
        );
        msg!("Creating Storage account!");
        invoke_signed(
            create_storage_pda_ix,
            &[
                authority_account.clone(),
                storage_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"storagePool".as_ref(),
                authority_account.key.as_ref(),
                &[storage_bump],
            ]],
        )?;
        {
            let mut storage_account_data =
                try_from_slice_unchecked::<Storage>(&storage_account.data.borrow_mut()).unwrap();
            let earray: Vec<Entry> = Vec::new();
            storage_account_data.authority = *authority_account.key;
            storage_account_data.bump = storage_bump;
            storage_account_data.gpustorage = earray;
            storage_account_data.serialize(&mut &mut storage_account.data.borrow_mut()[..])?;
        }
        Ok(())
    }
}

#[derive(BorshDeserialize, Debug)]
struct GPUIxPayload {
    model: String,
}

impl ProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (variant, rest) = input.split_first().ok_or(PError::InvalidInstruction)?;
        let payload = GPUIxPayload::try_from_slice(rest).unwrap();

        Ok(match variant {
            0 => Self::InitStorage {},
            1 => Self::CreateEntry {
                model: payload.model,
            },
            _ => return Err(PError::InvalidInstruction.into()),
        })
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Storage {
    pub authority: Pubkey,
    pub bump: u8,
    pub gpustorage: Vec<Entry>,
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Entry {
    pub storage: Pubkey,
    pub owner: Pubkey,
    pub model: String,
}
impl Entry {
    pub fn calculate_len(model: &str) -> usize {
        32 + 32 + 1 + 4 + model.len()
    }
}
// length of storage
impl Storage {
    pub const BASE_LEN: usize = 32 + 1 + 4;
    pub const ENTRY_LEN: usize = 32 + 32 + 1 + 4;

    pub fn calculate_len(entries: usize, model: String) -> usize {
        Self::BASE_LEN + entries * (Self::ENTRY_LEN + model.len())
    }
}

// Errors
#[derive(Error, Debug, Copy, Clone)]
pub enum PError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Invalid Storage Account")]
    InvalidStorageAccount,
    #[error("Invalid EntryData")]
    InvalidGpuData,
    #[error("Account not Writable")]
    AccountNotWritable,
    #[error("Serialization failed")]
    SerializationFailed,
}

impl From<PError> for ProgramError {
    fn from(e: PError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}
