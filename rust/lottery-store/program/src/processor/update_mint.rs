use crate::{
  errors::StoreError,
  processor::{
      NFTMeta, MintNFTArgs, MAX_URI_LENGTH
  },
  utils::{assert_owned_by, is_zero_account},
  constant::*,
};

use {
  borsh::{BorshSerialize},
  solana_program::{
      account_info::{next_account_info, AccountInfo},
      entrypoint::ProgramResult,
      program_error::ProgramError,
      pubkey::Pubkey,
  },
};

use std::str::FromStr;

struct Accounts<'a, 'b: 'a> {
  payer: &'a AccountInfo<'b>,
  nftmeta: &'a AccountInfo<'b>,
  rent: &'a AccountInfo<'b>,
  system: &'a AccountInfo<'b>,
}

fn parse_accounts<'a, 'b: 'a>(
  program_id: &Pubkey,
  accounts: &'a [AccountInfo<'b>],
) -> Result<Accounts<'a, 'b>, ProgramError> {
  let account_iter = &mut accounts.iter();
  let accounts = Accounts {
    payer: next_account_info(account_iter)?,
    nftmeta: next_account_info(account_iter)?,
    rent: next_account_info(account_iter)?,
    system: next_account_info(account_iter)?,
  };

  assert_owned_by(accounts.nftmeta, program_id)?;

  // check if rent sysvar program id is correct
  if *accounts.rent.key != Pubkey::from_str(RENT_SYSVAR_ID).map_err(|_| StoreError::InvalidPubkey)? {
    return Err(StoreError::InvalidRentSysvarId.into());
  }

  // check if system program id is correct
  if *accounts.system.key != Pubkey::from_str(SYSTEM_PROGRAM_ID).map_err(|_| StoreError::InvalidPubkey)? {
    return Err(StoreError::InvalidSystemProgramId.into());
  }

  // check if given store is initialized
  if is_zero_account(accounts.nftmeta) {
    return Err(StoreError::NotInitializedProgramData.into());
  }

  Ok(accounts)
}

pub fn update_mint(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
args: MintNFTArgs,
) -> ProgramResult {
  let accounts = parse_accounts(program_id, accounts)?;

  // Load the store and verify this bid is valid.
  let mut nft = NFTMeta::from_account_info(accounts.nftmeta)?;
  let mut array_of_zeroes = vec![];
  while array_of_zeroes.len() < MAX_URI_LENGTH - args.uri.len() {
    array_of_zeroes.push(0u8);
  }
  nft.uri = args.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();

  nft.serialize(&mut *accounts.nftmeta.data.borrow_mut())?;

  Ok(())
}