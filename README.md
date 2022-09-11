# Multi-Signature Wallet

Rust On-Chain based program implementing Anchor for Multisig Wallet on Solana.

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

WARN: there is an incompatibility with package @jnwng/walletconnect-solana. To fix this, you can modify its "package.json" from "node_modules" by deleting the { "type": "module" } attribute.