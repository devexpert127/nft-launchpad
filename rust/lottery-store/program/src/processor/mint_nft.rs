use crate::{
  errors::StoreError,
  processor::{
    NFTMeta, StoreData, MintNFTArgs, MAX_NFTMETA_LEN, MAX_NAME_LENGTH, MAX_SYMBOL_LENGTH, MAX_URI_LENGTH
  },
  utils::{create_or_allocate_account_raw, assert_owned_by, assert_token_program_matches_package, is_zero_account},
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
  authority: &'a AccountInfo<'b>,
  store_id: &'a AccountInfo<'b>,
  token_mint: &'a AccountInfo<'b>,
  token_pool: &'a AccountInfo<'b>,
  token_program: &'a AccountInfo<'b>,
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
    authority: next_account_info(account_iter)?,
    store_id: next_account_info(account_iter)?,
    token_mint: next_account_info(account_iter)?,
    token_pool: next_account_info(account_iter)?,
    token_program: next_account_info(account_iter)?,
    rent: next_account_info(account_iter)?,
    system: next_account_info(account_iter)?,
  };

  assert_owned_by(accounts.store_id, program_id)?;
  assert_token_program_matches_package(accounts.token_program)?;

  if *accounts.token_program.key != spl_token::id() {
    return Err(StoreError::InvalidTokenProgram.into());
  }

  // check if rent sysvar program id is correct
  if *accounts.rent.key != Pubkey::from_str(RENT_SYSVAR_ID).map_err(|_| StoreError::InvalidPubkey)? {
    return Err(StoreError::InvalidRentSysvarId.into());
  }

  // check if system program id is correct
  if *accounts.system.key != Pubkey::from_str(SYSTEM_PROGRAM_ID).map_err(|_| StoreError::InvalidPubkey)? {
    return Err(StoreError::InvalidSystemProgramId.into());
  }

  // check if token_mint is signer
  if !accounts.payer.is_signer {
    return Err(StoreError::SignatureMissing.into());
  }

  // check if nftmeta is signer
  if !accounts.nftmeta.is_signer {
    return Err(StoreError::SignatureMissing.into());
  }

  // check if given store is initialized
  if is_zero_account(accounts.store_id) {
    return Err(StoreError::NotInitializedProgramData.into());
  }

  Ok(accounts)
}

pub fn mint_nft(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  args: MintNFTArgs,
) -> ProgramResult {
  let accounts = parse_accounts(program_id, accounts)?;

  create_or_allocate_account_raw(
    *program_id,
    accounts.nftmeta,
    accounts.rent,
    accounts.system,
    accounts.payer,
    MAX_NFTMETA_LEN,
    &[
        &(*accounts.nftmeta.key).to_bytes(),
        &[args.bump],
    ],
  )?;

  // Load the store and verify this bid is valid.
  let mut store = StoreData::from_account_info(accounts.store_id)?;

  store.nft_amount += 1;
  store.serialize(&mut *accounts.store_id.data.borrow_mut())?;

  let mut array_of_zeroes_uri = vec![];
  while array_of_zeroes_uri.len() < MAX_URI_LENGTH - args.uri.len() {
    array_of_zeroes_uri.push(0u8);
  }

  let mut array_of_zeroes_name = vec![];
  while array_of_zeroes_name.len() < MAX_NAME_LENGTH - args.name.len() {
    array_of_zeroes_name.push(0u8);
  }

  let mut array_of_zeroes_symbol = vec![];
  while array_of_zeroes_symbol.len() < MAX_SYMBOL_LENGTH - args.symbol.len() {
    array_of_zeroes_symbol.push(0u8);
  }

  // Configure Store.
  NFTMeta {
    store_id: *accounts.store_id.key,
    nft_number: store.nft_amount,
    name: args.name.clone() + std::str::from_utf8(&array_of_zeroes_name).unwrap(),
    symbol: args.symbol.clone() + std::str::from_utf8(&array_of_zeroes_symbol).unwrap(),
    uri: args.uri.clone() + std::str::from_utf8(&array_of_zeroes_uri).unwrap(),
    mint: *accounts.token_mint.key,
    token_pool: *accounts.token_pool.key,
    authority: *accounts.authority.key,
    exist_nft: 1,
    bump: args.bump,
  }
  .serialize(&mut * accounts.nftmeta.data.borrow_mut())?;

  Ok(())
}