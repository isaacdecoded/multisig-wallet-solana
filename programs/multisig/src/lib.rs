//! An example of a multisig to execute arbitrary Solana transactions.
//!
//! This program can be used to allow a multisig to govern anything a regular
//! Pubkey can govern. One can use the multisig as a BPF program upgrade
//! authority, a mint authority, etc.
//!
//! To use, one must first create a `Multisig` account, specifying two important
//! parameters:
//!
//! 1. Owners - the set of addresses that sign transactions for the multisig.
//! 2. Threshold - the number of signers required to execute a transaction.
//!
//! Once the `Multisig` account is created, one can create a `Transaction`
//! account, specifying the parameters for a normal solana transaction.
//!
//! To sign, owners should invoke the `approve` instruction, and finally,
//! the `execute_transaction`, once enough (i.e. `threshold`) of the owners have
//! signed.

mod entities;
mod utils;

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use std::convert::Into;
use std::ops::Deref;

use self::entities::{
	Auth,
	MultisigWallet,
	Transaction,
	TransactionAccount,
};
use self::utils::{
	ErrorCode,
	assert_unique_owners,
};

use crate::entities::__client_accounts_auth;

declare_id!("4NGvJq94MGwTah7VcaMJPfNG94iqxtRPqWNEibTr5qCe");

#[program]
pub mod multisig_wallet {
	use super::*;

	// Initializes a new multisig account with a set of owners and a threshold.
	pub fn create_multisig(
		ctx: Context<CreateMultisig>,
		owners: Vec<Pubkey>,
		threshold: u64,
		nonce: u8,
	) -> Result<()> {
		assert_unique_owners(&owners)?;
		require!(
			threshold > 0 && threshold <= owners.len() as u64,
			InvalidThreshold
		);
		require!(!owners.is_empty(), InvalidOwnersLen);

		let multisig = &mut ctx.accounts.multisig;
		multisig.owners = owners;
		multisig.threshold = threshold;
		multisig.nonce = nonce;
		multisig.owner_set_seqno = 0;
		Ok(())
	}

	// Creates a new transaction account, automatically signed by the creator,
	// which must be one of the owners of the multisig.
	pub fn create_transaction(
		ctx: Context<CreateTransaction>,
		pid: Pubkey,
		accs: Vec<TransactionAccount>,
		data: Vec<u8>,
	) -> Result<()> {
		let owner_index = ctx
			.accounts
			.multisig
			.owners
			.iter()
			.position(|a| a == ctx.accounts.proposer.key)
			.ok_or(ErrorCode::InvalidOwner)?;

		let mut signers = Vec::new();
		signers.resize(ctx.accounts.multisig.owners.len(), false);
		signers[owner_index] = true;

		let tx = &mut ctx.accounts.transaction;
		tx.program_id = pid;
		tx.accounts = accs;
		tx.data = data;
		tx.signers = signers;
		tx.multisig = ctx.accounts.multisig.key();
		tx.did_execute = false;
		tx.owner_set_seqno = ctx.accounts.multisig.owner_set_seqno;

		Ok(())
	}

	// Approves a transaction on behalf of an owner of the multisig.
	pub fn approve(ctx: Context<Approve>) -> Result<()> {
		let owner_index = ctx
			.accounts
			.multisig
			.owners
			.iter()
			.position(|a| a == ctx.accounts.owner.key)
			.ok_or(ErrorCode::InvalidOwner)?;

		ctx.accounts.transaction.signers[owner_index] = true;

		Ok(())
	}

	// Set owners and threshold at once.
	pub fn set_owners_and_change_threshold<'info>(
		ctx: Context<'_, '_, '_, 'info, Auth<'info>>,
		owners: Vec<Pubkey>,
		threshold: u64,
	) -> Result<()> {
		set_owners(
			Context::new(
				ctx.program_id,
				ctx.accounts,
				ctx.remaining_accounts,
				ctx.bumps.clone(),
			),
			owners,
		)?;
		change_threshold(ctx, threshold)
	}

	// Sets the owners field on the multisig. The only way this can be invoked
	// is via a recursive call from execute_transaction -> set_owners.
	pub fn set_owners(ctx: Context<Auth>, owners: Vec<Pubkey>) -> Result<()> {
		assert_unique_owners(&owners)?;
		require!(!owners.is_empty(), InvalidOwnersLen);

		let multisig = &mut ctx.accounts.multisig_wallet;

		if (owners.len() as u64) < multisig.threshold {
			multisig.threshold = owners.len() as u64;
		}

		multisig.owners = owners;
		multisig.owner_set_seqno += 1;

		Ok(())
	}

	pub fn set_data(ctx: Context<Auth>, data: String) -> Result<()> {
		let multisig = &mut ctx.accounts.multisig_wallet;
		multisig.data = data;
		Ok(())
	}

	// Changes the execution threshold of the multisig. The only way this can be
	// invoked is via a recursive call from execute_transaction ->
	// change_threshold.
	pub fn change_threshold(ctx: Context<Auth>, threshold: u64) -> Result<()> {
		require!(threshold > 0, InvalidThreshold);
		if threshold > ctx.accounts.multisig_wallet.owners.len() as u64 {
			return Err(ErrorCode::InvalidThreshold.into());
		}
		let multisig = &mut ctx.accounts.multisig_wallet;
		multisig.threshold = threshold;
		Ok(())
	}

	// Executes the given transaction if threshold owners have signed it.
	pub fn execute_transaction(ctx: Context<ExecuteTransaction>) -> Result<()> {
		// Has this been executed already?
		if ctx.accounts.transaction.did_execute {
			return Err(ErrorCode::AlreadyExecuted.into());
		}

		// Do we have enough signers.
		let sig_count = ctx
			.accounts
			.transaction
			.signers
			.iter()
			.filter(|&did_sign| *did_sign)
			.count() as u64;
		if sig_count < ctx.accounts.multisig.threshold {
			return Err(ErrorCode::NotEnoughSigners.into());
		}

		// Execute the transaction signed by the multisig.
		let mut ix: Instruction = (*ctx.accounts.transaction).deref().into();
		ix.accounts = ix
			.accounts
			.iter()
			.map(|acc| {
				let mut acc = acc.clone();
				if &acc.pubkey == ctx.accounts.multisig_signer.key {
						acc.is_signer = true;
				}
				acc
			})
			.collect();
		let multisig_key = ctx.accounts.multisig.key();
		let seeds = &[multisig_key.as_ref(), &[ctx.accounts.multisig.nonce]];
		let signer = &[&seeds[..]];
		let accounts = ctx.remaining_accounts;
		solana_program::program::invoke_signed(&ix, accounts, signer)?;

		// Burn the transaction to ensure one time use.
		ctx.accounts.transaction.did_execute = true;

		Ok(())
	}
}

#[derive(Accounts)]
pub struct CreateMultisig<'info> {
	#[account(zero, signer)]
	multisig: Box<Account<'info, MultisigWallet>>,
}

#[derive(Accounts)]
pub struct CreateTransaction<'info> {
	multisig: Box<Account<'info, MultisigWallet>>,
	#[account(zero, signer)]
	transaction: Box<Account<'info, Transaction>>,
	// One of the owners. Checked in the handler.
	proposer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
	#[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
	multisig: Box<Account<'info, MultisigWallet>>,
	#[account(mut, has_one = multisig)]
	transaction: Box<Account<'info, Transaction>>,
	// One of the multisig owners. Checked in the handler.
	owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteTransaction<'info> {
	#[account(constraint = multisig.owner_set_seqno == transaction.owner_set_seqno)]
	multisig: Box<Account<'info, MultisigWallet>>,
	/// CHECK: multisig_signer is a PDA program signer. Data is never read or written to
	#[account(
		seeds = [multisig.key().as_ref()],
		bump = multisig.nonce,
	)]
	multisig_signer: UncheckedAccount<'info>,
	#[account(mut, has_one = multisig)]
	transaction: Box<Account<'info, Transaction>>,
}
