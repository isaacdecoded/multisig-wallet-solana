mod entities;
mod use_cases;
mod utils;

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use std::convert::Into;
use std::ops::Deref;

use crate::entities::{
	TransactionAccount,
};
use crate::use_cases::{
	Auth,
	CreateMultisig,
	CreateTransaction,
	Approve,
	ExecuteTransaction,
	create_multisig_wallet,
	create_multisig_wallet_transaction,
	set_multisig_wallet_owners,
	execute_multisig_wallet_transaction,
};
use crate::utils::ErrorCode;
use crate::use_cases::__client_accounts_auth;
use crate::use_cases::__client_accounts_approve;
use crate::use_cases::__client_accounts_create_transaction;
use crate::use_cases::__client_accounts_execute_transaction;
use crate::use_cases::__client_accounts_create_multisig;
declare_id!("4NGvJq94MGwTah7VcaMJPfNG94iqxtRPqWNEibTr5qCe");

#[program]
pub mod multisig_wallet {
	use super::*;

	// Initializes a new multisig wallet account with a set of owners and a threshold.
	pub fn create_multisig(
		ctx: Context<CreateMultisig>,
		owners: Vec<Pubkey>,
		threshold: u64,
		nonce: u8,
	) -> Result<()> {
		let result = create_multisig_wallet(
			owners,
			threshold,
			nonce,
		);
		match result {
			Ok(multisig_wallet) => {
				let multisig = &mut ctx.accounts.multisig;
				multisig.owners = multisig_wallet.owners;
				multisig.threshold = multisig_wallet.threshold;
				multisig.nonce = multisig_wallet.nonce;
				multisig.owner_set_seqno = multisig_wallet.owner_set_seqno;
				return Ok(())
			},
			Err(error) => return Err(error),
		}
	}

	// Creates a new transaction account, automatically signed by the creator,
	// which must be one of the owners of the multisig.
	pub fn create_transaction(
		ctx: Context<CreateTransaction>,
		pid: Pubkey,
		accs: Vec<TransactionAccount>,
		data: Vec<u8>,
	) -> Result<()> {
		let result = create_multisig_wallet_transaction(
			&ctx.accounts.multisig,
			ctx.accounts.proposer.key,
			pid,
			accs,
			data,
		);
		match result {
			Ok(transaction) => {
				let tx = &mut ctx.accounts.transaction;
				tx.program_id = transaction.program_id;
				tx.accounts = transaction.accounts;
				tx.data = transaction.data;
				tx.signers = transaction.signers;
				tx.multisig = transaction.multisig;
				tx.did_execute = transaction.did_execute;
				tx.owner_set_seqno = transaction.owner_set_seqno;
				return Ok(())
			},
			Err(error) => return Err(error),
		}
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
		let multisig = &mut ctx.accounts.multisig_wallet;
		let result = set_multisig_wallet_owners(multisig, owners);
		match result {
			Ok(()) => return Ok(()),
			Err(error) => return Err(error),
		}
	}

	pub fn set_data(ctx: Context<Auth>, data: Option<String>) -> Result<()> {
		let multisig = &mut ctx.accounts.multisig_wallet;
		multisig.data = data;
		Ok(())
	}

	// Changes the execution threshold of the multisig wallet. The only way this can be
	// invoked is via a recursive call from execute_transaction -> change_threshold.
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
		let result = execute_multisig_wallet_transaction(
			&ctx.accounts.multisig,
			&mut ctx.accounts.transaction,
		);
		match result {
			Ok(()) => {
				// Execute the transaction on Solana signed by the multisig.
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
				return Ok(())
			},
			Err(error) => return Err(error),
		}
	}
}
