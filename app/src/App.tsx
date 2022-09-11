import React, { useState } from "react";
import { BN, web3 } from "@project-serum/anchor";
import idl from "./idl.json";
import {
  clusterApiUrl,
  Connection,
  PublicKey,
} from "@solana/web3.js";
import {
  AnchorProvider,
  Program,
} from "@project-serum/anchor";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";
import {
  useWallet,
  WalletProvider,
  ConnectionProvider,
} from '@solana/wallet-adapter-react';
import {
  WalletModalProvider,
  WalletMultiButton,
} from '@solana/wallet-adapter-react-ui';
import { MultisigWalletForm, MultisigWalletList } from './components'
import { MultisigWallet, MultisigWalletTransaction } from "./types";

import 'antd/dist/antd.css';
require('@solana/wallet-adapter-react-ui/styles.css');

const endPoint = clusterApiUrl("devnet");
const wallets = [new PhantomWalletAdapter()];
const opts = {
  preflightCommitment: "processed" as any
};
const programID = new PublicKey(idl.metadata.address);

export const App: React.FC = () => {
  const [owners, setOwners] = useState<web3.Keypair[]>([]);
  const [multisigWallets, setMultisigWallets] = useState<MultisigWallet[]>([]);
  const wallet = useWallet();

  function _refreshMultisigWalletTransaction(
    multisigWallet: MultisigWallet,
    transaction: MultisigWalletTransaction,
  ) {
    multisigWallet.transactions = multisigWallet.transactions.map(t => {
      if (t.keypair.publicKey === transaction.keypair.publicKey) {
        return transaction;
      }
      return t;
    })
    const newState = multisigWallets.map(mw => {
      if (mw.keypair.publicKey === multisigWallet.keypair.publicKey) {
        return multisigWallet;
      }
      return mw;
    });
    setMultisigWallets(newState);
  }

  async function getProvider() {
    const network = endPoint;
    const connection = new Connection(network, opts.preflightCommitment);
    const provider = new AnchorProvider(
      connection, wallet as any, opts.preflightCommitment,
    );
    return provider;
  }

  function addMultisigWalletOwner() {
    const newOwner = web3.Keypair.generate();
    setOwners(owners.concat(newOwner));
    return newOwner.publicKey;
  }

  async function createMultisigWallet(input: {
    threshold: number,
  }) {
    const provider = await getProvider();
    const program = new Program(idl as any, programID, provider);
    const multisig = web3.Keypair.generate();
    const multisigSize = 200; // Bad practice only for development purposes
    try {
      const bnThreshold = new BN(input.threshold);
      const [, nonce] = await web3.PublicKey.findProgramAddress(
        [multisig.publicKey.toBuffer()],
        program.programId,
      );
      const ownerPublicKeys = owners.map(o => o.publicKey);
      await program.rpc.createMultisig(ownerPublicKeys, bnThreshold, nonce, {
        accounts: {
          multisig: multisig.publicKey,
        },
        instructions: [
          await program.account.multisigWallet.createInstruction(
            multisig,
            multisigSize
          ),
        ],
        signers: [multisig],
      });
      let multisigAccount = await program.account.multisigWallet.fetch(
        multisig.publicKey
      );
      setOwners([]);
      setMultisigWallets(multisigWallets.concat({
        keypair: multisig,
        ownerKeypairs: owners,
        threshold: multisigAccount.threshold,
        owners: multisigAccount.owners,
        transactions: [],
      }));
    } catch (e) {
      console.error(e);
    }
  }

  async function createMultisigWalletTransaction(input: {
    multisigWallet: MultisigWallet,
    proposer: string,
    data: string,
  }) {
    const provider = await getProvider();
    const program = new Program(idl as any, programID, provider);
    const proposerKeypair = input.multisigWallet.ownerKeypairs.find(
      owner => owner.publicKey.toString() === input.proposer,
    );
    if (!proposerKeypair) {
      throw new Error('Invalid transaction proposer.');
    }
    const data = program.coder.instruction.encode("set_data", {
      data: input.data,
    });
    const transaction = web3.Keypair.generate();
    const txSize = 1000; // Bad practice only for development purposes
    const [multisigSigner] = await web3.PublicKey.findProgramAddress(
      [input.multisigWallet.keypair.publicKey.toBuffer()],
      program.programId,
    );
    const accounts = [
      {
        pubkey: input.multisigWallet.keypair.publicKey,
        isWritable: true,
        isSigner: false,
      },
      {
        pubkey: multisigSigner,
        isWritable: false,
        isSigner: true,
      },
    ];
    await program.rpc.createTransaction(program.programId, accounts, data, {
      accounts: {
        multisig: input.multisigWallet.keypair.publicKey,
        transaction: transaction.publicKey,
        proposer: proposerKeypair.publicKey,
      },
      instructions: [
        await program.account.transaction.createInstruction(
          transaction,
          txSize
        ),
      ],
      signers: [transaction, proposerKeypair],
    });
    input.multisigWallet.transactions.push({
      keypair: transaction,
      data: input.data,
      proposer: proposerKeypair,
      signers: [proposerKeypair],
      executed: false,
    });
    const newState = multisigWallets.map(multisigWallet => {
      if (multisigWallet.keypair.publicKey === input.multisigWallet.keypair.publicKey) {
        return input.multisigWallet
      }
      return multisigWallet
    });
    setMultisigWallets(newState);
  }

  async function approveMultisigWalletTransaction(
    multisigWallet: MultisigWallet,
    transaction: MultisigWalletTransaction,
    approver: string,
  ) {
    const provider = await getProvider();
    const program = new Program(idl as any, programID, provider);
    const approverKeypair = multisigWallet.ownerKeypairs.find(
      owner => owner.publicKey.toString() === approver,
    );
    if (!approverKeypair) {
      throw new Error('Invalid transaction proposer.');
    }
    await program.rpc.approve({
      accounts: {
        multisig: multisigWallet.keypair.publicKey,
        transaction: transaction.keypair.publicKey,
        owner: approverKeypair.publicKey,
      },
      signers: [approverKeypair],
    });
    transaction.signers = transaction.signers.concat(approverKeypair);
    _refreshMultisigWalletTransaction(multisigWallet, transaction);
  }

  async function executeMultisigWalletTransaction(
    multisigWallet: MultisigWallet,
    transaction: MultisigWalletTransaction,
  ) {
    const provider = await getProvider();
    const program = new Program(idl as any, programID, provider);
    const [multisigSigner] = await web3.PublicKey.findProgramAddress(
      [multisigWallet.keypair.publicKey.toBuffer()],
      program.programId,
    );
    const accounts: any = program.instruction.setOwners
      .accounts({
        multisigWallet: multisigWallet.keypair.publicKey,
        multisigSigner,
      });
    await program.rpc.executeTransaction({
      accounts: {
        multisig: multisigWallet.keypair.publicKey,
        multisigSigner,
        transaction: transaction.keypair.publicKey,
      },
      remainingAccounts: accounts.map((meta: any) =>
          meta.pubkey.equals(multisigSigner)
            ? { ...meta, isSigner: false }
            : meta
        )
        .concat({
          pubkey: program.programId,
          isWritable: false,
          isSigner: false,
        }),
    });
    transaction.executed = true;
    _refreshMultisigWalletTransaction(multisigWallet, transaction);
  }

  if (!wallet.connected) {
    return (<></>);
  }
  return (
    <>
      <MultisigWalletForm
        addOwner={addMultisigWalletOwner}
        onCreate={createMultisigWallet}
      />
      <MultisigWalletList
        multisigWallets={multisigWallets}
        onCreateTransaction={createMultisigWalletTransaction}
        onApproveTransaction={approveMultisigWalletTransaction}
        onExecuteTransaction={executeMultisigWalletTransaction}
      />
    </>
  );
};

const AppWithProvider = () => (
  <ConnectionProvider endpoint={endPoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <div style={{ display: 'flex', justifyContent: 'right', padding: 20 }}>
            <WalletMultiButton/>
          </div>
        </WalletModalProvider>
        <App />
      </WalletProvider>
    </ConnectionProvider>
);

export default AppWithProvider;
