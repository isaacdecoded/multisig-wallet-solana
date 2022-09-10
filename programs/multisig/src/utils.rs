use anchor_lang::prelude::*;

pub fn assert_unique_owners(owners: &[Pubkey]) -> Result<()> {
	for (i, owner) in owners.iter().enumerate() {
		require!(
			!owners.iter().skip(i + 1).any(|item| item == owner),
			UniqueOwners
		)
	}
	Ok(())
}

#[error_code]
pub enum ErrorCode {
	#[msg("The given owner is not part of this multisig.")]
	InvalidOwner,
	#[msg("Owners length must be non zero.")]
	InvalidOwnersLen,
	#[msg("Not enough owners signed this transaction.")]
	NotEnoughSigners,
	#[msg("Cannot delete a transaction that has been signed by an owner.")]
	TransactionAlreadySigned,
	#[msg("Overflow when adding.")]
	Overflow,
	#[msg("Cannot delete a transaction the owner did not create.")]
	UnableToDelete,
	#[msg("The given transaction has already been executed.")]
	AlreadyExecuted,
	#[msg("Threshold must be less than or equal to the number of owners.")]
	InvalidThreshold,
	#[msg("Owners must be unique")]
	UniqueOwners,
}