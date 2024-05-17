use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
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
use std::{io::BufWriter, mem};
use thiserror::Error;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}

pub enum ProgramInstruction {
    InitStorage {},
    CreateEntry { owner: Pubkey, model: String },
}

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = ProgramInstruction::unpack(instruction_data)?;

        match instruction {
            ProgramInstruction::InitStorage {} => {
                msg!("Instruction: InitStorage");
                Self::process_init_storage(accounts, program_id)
            }
            ProgramInstruction::CreateEntry { owner, model } => {
                msg!("Instruction: CreateEntry");
                Self::process_Entry(accounts, owner, model, program_id)
            }
        }
    }

    fn process_Entry(
        accounts: &[AccountInfo],
        owner: Pubkey,
        model: String,
        program_id: &Pubkey,
    ) -> ProgramResult {
        if model.len() <= 0 {
            return Err(BlogError::InvalidPostData.into());
        }

        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let storage_account = next_account_info(account_info_iter)?;
        let owner_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (storage_pda, _storage_bump) = Pubkey::find_program_address(
            &[b"Storage".as_ref(), authority_account.key.as_ref()],
            program_id,
        );
        if storage_pda != *storage_account.key
            || !storage_account.is_writable
            || storage_account.data_is_empty()
        {
            return Err(BlogError::InvalidBlogAccount.into());
        }

        let (entry_pda, entry_bump) = Pubkey::find_program_address(
            &[
                b"Entry".as_ref(),
                owner.as_ref(),
                authority_account.key.as_ref(),
            ],
            program_id,
        );
        if entry_pda != *owner_account.key {
            return Err(BlogError::InvalidPostAccount.into());
        }

        let entry_len: usize = 32 + 32 + 1 + mem::size_of::<Entry>();

        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(entry_len);

        let create_entry_pda_ix = &system_instruction::create_account(
            authority_account.key,
            owner_account.key,
            rent_lamports,
            entry_len.try_into().unwrap(),
            program_id,
        );
        msg!("Creating post account!");
        invoke_signed(
            create_entry_pda_ix,
            &[
                authority_account.clone(),
                owner_account.clone(),
                system_program.clone(),
            ],
            &[&[
                b"Entry".as_ref(),
                owner.as_ref(),
                authority_account.key.as_ref(),
                &[entry_bump],
            ]],
        )?;

        let mut entry_account_state =
            try_from_slice_unchecked::<Entry>(&owner_account.data.borrow()).unwrap();
        entry_account_state.owner = *authority_account.key;
        entry_account_state.storage = *storage_account.key;
        entry_account_state.bump = entry_bump;
        entry_account_state.model = model;
        // msg!("Serializing Post data");
        // entry_account_state.serialize(&mut &mut post_account.data.borrow_mut()[..])?;

        let mut storage_account_state = Storage::try_from_slice(&storage_account.data.borrow())?;
        storage_account_state.gpustorage.push(entry_account_state);

        msg!("Serializing Blog data");
        storage_account_state.serialize(&mut &mut storage_account.data.borrow_mut()[..])?;

        Ok(())
    }

    fn process_init_storage(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let storage_account = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;

        if !authority_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (storage_pda, storage_bump) = Pubkey::find_program_address(
            &[b"Storage".as_ref(), authority_account.key.as_ref()],
            program_id,
        );
        if storage_pda != *storage_account.key {
            return Err(BlogError::InvalidBlogAccount.into());
        }

        let rent = Rent::get()?;
        let rent_lamports = rent.minimum_balance(Storage::LEN);

        let create_storage_pda_ix = &system_instruction::create_account(
            authority_account.key,
            storage_account.key,
            rent_lamports,
            Storage::LEN.try_into().unwrap(),
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
                b"Storage".as_ref(),
                authority_account.key.as_ref(),
                &[storage_bump],
            ]],
        )?;

        let mut storage_account_state = Storage::try_from_slice(&storage_account.data.borrow())?;
        storage_account_state.authority = *authority_account.key;
        storage_account_state.bump = storage_bump;
        storage_account_state.gpustorage = Vec::new();
        storage_account_state.serialize(&mut &mut storage_account.data.borrow_mut()[..])?;

        Ok(())
    }
}

#[derive(BorshDeserialize, Debug)]
struct GPUIxPayload {
    owner: Pubkey,
    model: String,
}

impl ProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (variant, rest) = input.split_first().ok_or(BlogError::InvalidInstruction)?;
        let payload = GPUIxPayload::try_from_slice(rest).unwrap();

        Ok(match variant {
            0 => Self::InitStorage {},
            1 => Self::CreateEntry {
                owner: payload.owner,
                model: payload.model,
            },
            _ => return Err(BlogError::InvalidInstruction.into()),
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
    pub bump: u8,
    pub model: String,
}

impl Storage {
    pub const LEN: usize = 32 + 1 + mem::size_of::<Entry>();
}

#[derive(Error, Debug, Copy, Clone)]
pub enum BlogError {
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid Blog Account")]
    InvalidBlogAccount,

    #[error("Invalid Post Account")]
    InvalidPostAccount,

    #[error("Invalid Post Data")]
    InvalidPostData,

    #[error("Account not Writable")]
    AccountNotWritable,
}

impl From<BlogError> for ProgramError {
    fn from(e: BlogError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}
