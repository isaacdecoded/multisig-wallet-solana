import { web3 } from "@project-serum/anchor";
import { MultisigWalletTransaction } from "./MultisigWalletTransaction";

export interface MultisigWallet {
  keypair: web3.Keypair
  ownerKeypairs: web3.Keypair[],
  threshold: number
  owners: string[]
  transactions: MultisigWalletTransaction[]
};
