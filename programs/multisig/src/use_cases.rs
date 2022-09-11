use anchor_lang::prelude::*;
use crate::entities::{
	MultisigWallet,
  Transaction,
  TransactionAccount,
};
use crate::utils::ErrorCode;

fn assert_unique_owners(owners: &[Pubkey]) -> Result<()> {
	for (i, owner) in owners.iter().enumerate() {
		require!(
			!owners.iter().skip(i + 1).any(|item| item == owner),
			UniqueOwners
		)
	}
	Ok(())
}

pub fn create_multisig_wallet (
  owners: Vec<Pubkey>,
  threshold: u64,
  nonce: u8,
) -> Result<MultisigWallet> {
  assert_unique_owners(&owners)?;
  require!(
    threshold > 0 && threshold <= owners.len() as u64,
    ErrorCode::InvalidThreshold,
  );
  require!(!owners.is_empty(), ErrorCode::InvalidOwnersLen);
  let multisig_wallet = MultisigWallet{
    owners,
    threshold,
    nonce,
    owner_set_seqno: 0,
    data: None,
  };
  Ok(multisig_wallet)
}

pub fn set_multisig_wallet_owners (
  multisig_wallet: &mut Box<Account<MultisigWallet>>,
  owners: Vec<Pubkey>,
) -> Result<()> {
  assert_unique_owners(&owners)?;
  require!(!owners.is_empty(), InvalidOwnersLen);
  if (owners.len() as u64) < multisig_wallet.threshold {
    multisig_wallet.threshold = owners.len() as u64;
  }
  multisig_wallet.owners = owners;
  multisig_wallet.owner_set_seqno += 1;
  Ok(())
}

pub fn create_multisig_wallet_transaction (
  multisig_wallet: &Box<Account<MultisigWallet>>,
  proposer: &Pubkey,
  program_id: Pubkey,
  accounts: Vec<TransactionAccount>,
  data: Vec<u8>,
) -> Result<Transaction> {
  let owner_index = multisig_wallet
    .owners
    .iter()
    .position(|a| a == proposer)
    .ok_or(ErrorCode::InvalidOwner)?;
  let mut signers = Vec::new();
  signers.resize(multisig_wallet.owners.len(), false);
  signers[owner_index] = true;
  let transaction = Transaction{
    program_id,
    accounts,
    data,
    did_execute: false,
    multisig: multisig_wallet.key(),
    signers,
    owner_set_seqno: multisig_wallet.owner_set_seqno,
  };
  Ok(transaction)
}

pub fn execute_multisig_wallet_transaction (
  multisig_wallet: &Box<Account<MultisigWallet>>,
  transaction: &mut Box<Account<Transaction>>,
) -> Result<()> {
  if transaction.did_execute {
    return Err(ErrorCode::AlreadyExecuted.into());
  }
  let sig_count = transaction
    .signers
    .iter()
    .filter(|&did_sign| *did_sign)
    .count() as u64;
  if sig_count < multisig_wallet.threshold {
    return Err(ErrorCode::NotEnoughSigners.into());
  }
  transaction.did_execute = true;
  Ok(())
}

// Input Port/DAO definitions
#[derive(Accounts)]
pub struct Auth<'info> {
	#[account(mut)]
	pub multisig_wallet: Box<Account<'info, MultisigWallet>>,
	#[account(
    seeds = [multisig_wallet.key().as_ref()],
    bump = multisig_wallet.nonce,
	)]
	multisig_signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct CreateMultisig<'info> {
	#[account(zero, signer)]
	pub multisig: Box<Account<'info, MultisigWallet>>,
}

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
	pub multisig: Box<Account<'info, MultisigWallet>>,
	#[account(zero, signer)]
	pub transaction: Box<Account<'info, Transaction>>,
	// One of the owners. Checked in the handler.
	pub proposer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
	#[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
	pub multisig: Box<Account<'info, MultisigWallet>>,
	#[account(mut, has_one = multisig)]
	pub transaction: Box<Account<'info, Transaction>>,
	// One of the multisig owners. Checked in the handler.
	pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
	#[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
	pub multisig: Box<Account<'info, MultisigWallet>>,
  /// CHECK: multisig_signer is a PDA program signer. Data is never read or written to
	#[account(
		seeds = [multisig.key().as_ref()],
		bump = multisig.nonce,
	)]
	pub multisig_signer: UncheckedAccount<'info>,
	#[account(mut, has_one = multisig)]
	pub transaction: Box<Account<'info, Transaction>>,
}

