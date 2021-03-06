use crate::{
    errors::StoreError,
    processor::{
        StoreData, 
    },
    utils::{create_or_allocate_account_raw, assert_program_account},
    constant::*,
};

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};
use std::str::FromStr;

struct Accounts<'a, 'b: 'a> {
    payer: &'a AccountInfo<'b>,
    store_id: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
    rent: &'a AccountInfo<'b>,
    system: &'a AccountInfo<'b>,
}


#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct CreateStoreArgs {
    /// bump
    pub bump: u8,
}


fn parse_accounts<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
    let account_iter = &mut accounts.iter();
    let accounts = Accounts {
        payer: next_account_info(account_iter)?,
        store_id: next_account_info(account_iter)?,
        authority: next_account_info(account_iter)?,
        rent: next_account_info(account_iter)?,
        system: next_account_info(account_iter)?,
    };

    // check if rent sysvar program id is correct
    if *accounts.rent.key != Pubkey::from_str(RENT_SYSVAR_ID).map_err(|_| StoreError::InvalidPubkey)? {
        return Err(StoreError::InvalidRentSysvarId.into());
    }

    // check if system program id is correct
    if *accounts.system.key != Pubkey::from_str(SYSTEM_PROGRAM_ID).map_err(|_| StoreError::InvalidPubkey)? {
        return Err(StoreError::InvalidSystemProgramId.into());
    }

    // check if store id is signer
    if !accounts.store_id.is_signer {
        return Err(StoreError::SignatureMissing.into());
    }

    assert_program_account(accounts.store_id.key, program_id, accounts.authority.key)?;

    Ok(accounts)
}

pub fn create_store(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: CreateStoreArgs,
) -> ProgramResult {
    msg!("+ Processing CreateStore");
    let accounts = parse_accounts(program_id, accounts)?;

    // ************
    // bump from frontend
    // ************
    
    // Create store
    if accounts.store_id.data_is_empty() {
        create_or_allocate_account_raw(
            *program_id,
            accounts.store_id,
            accounts.rent,
            accounts.system,
            accounts.payer,
            std::mem::size_of::<StoreData>() ,
            &[
                &(*accounts.store_id.key).to_bytes(),
                &[args.bump],
            ],
        )?;

        // Configure Store.
        StoreData {
            owner: *accounts.payer.key,
            authority: *accounts.authority.key,
            nft_amount: 0,
            bump: args.bump,
        }
        .serialize(&mut *accounts.store_id.data.borrow_mut())?;
    }
    Ok(())
}
