# Multi-Signature Wallet

Rust On-Chain based program implementing Anchor for Multisig Wallet on Solana.

This is a basic example of a Multisig Wallet program that executes arbitrary Solana transactions.
A multisig is a smart contract that acts as a wallet and have following functionalities:
- create wallet
- set owners and threshold (threshold: how many owners needed to approve a transaction)
- create transaction (first approver)
- confirm transaction (rest of approvers)
- reject transaction

## Usage

To start using, first create a `Multisig` account, specifying two important
parameters:

1. Owners - the set of addresses that sign transactions for the multisig.
2. Threshold - the number of signers required to execute a transaction.

Once the `Multisig` account is created, it is ready to create `Transaction`
account, specifying the parameters for a normal solana transaction.

To sign, owners should invoke the `approve` instruction, and finally,
the `execute_transaction`, once enough (i.e. `threshold`) of the owners have
signed.

## Prerequisites:

- NodeJS ([nvm](https://github.com/nvm-sh/nvm) recommended)
- [Rust](https://www.rust-lang.org/tools/install)
- [Solana Tool Suite](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://book.anchor-lang.com/getting_started/installation.html)

## Running instructions:

1) Build Rust program
```bash
$ yarn build
```
or
```bash
$ anchor build
```

2) Deploy built program on Solana devnet
```bash
$ yarn deploy-devnet
```
or
```bash
$ anchor deploy --provider.cluster devnet
```

3) Go to /app directory and run react app
```bash
$ cd app && yarn start
```

:warning: **there is an incompatibility with package @jnwng/walletconnect-solana.** To fix this, you can modify its "package.json" from "node_modules" by deleting the { "type": "module" } attribute.
