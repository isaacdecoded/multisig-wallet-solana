import { web3 } from "@project-serum/anchor";

export interface MultisigWalletTransaction {
  keypair: web3.Keypair
  proposer: web3.Keypair
  signers: web3.Keypair[]
  data: string
  executed: boolean
};
