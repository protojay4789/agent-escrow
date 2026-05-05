/**
 * Wallet integration for AgentEscrow
 *
 * Supports Phantom wallet adapter and Swig programmable wallets
 * for both human users and AI agents.
 */

import {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

/**
 * Swig wallet configuration for AI agents
 * Swig provides programmable wallets that can be controlled by on-chain programs
 */
export interface SwigWalletConfig {
  /** The agent that owns this Swig wallet */
  agentOwner: PublicKey;
  /** Authority that can sign transactions for this wallet */
  authority: PublicKey;
  /** Ruleset for the Swig wallet */
  ruleset: PublicKey;
}

/**
 * Initialize a Swig wallet for an AI agent
 *
 * In production, this would call the Swig program to create a
 * programmable wallet that can sign transactions on behalf of the agent.
 *
 * @param connection - Solana connection
 * @param agentOwner - The public key of the agent owner
 * @returns The Swig wallet public key
 */
export async function initializeSwigWallet(
  connection: Connection,
  agentOwner: PublicKey
): Promise<PublicKey> {
  // Derive the Swig wallet PDA
  const [swigWallet] = PublicKey.findProgramAddressSync(
    [Buffer.from("swig"), agentOwner.toBuffer()],
    new PublicKey("SwigWalletProgramIdHere") // Replace with actual Swig program ID
  );

  // In production, this would create the Swig wallet via CPI
  // For now, return the derived PDA
  return swigWallet;
}

/**
 * Fund an escrow wallet with SOL
 */
export async function fundEscrow(
  connection: Connection,
  fromWallet: PublicKey,
  escrowVault: PublicKey,
  amountLamports: number
): Promise<Transaction> {
  const transaction = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: fromWallet,
      toPubkey: escrowVault,
      lamports: amountLamports,
    })
  );

  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = fromWallet;

  return transaction;
}

/**
 * Format lamports to SOL with proper decimal places
 */
export function lamportsToSol(lamports: number): string {
  return (lamports / LAMPORTS_PER_SOL).toFixed(4);
}

/**
 * Format SOL to lamports
 */
export function solToLamports(sol: number): number {
  return Math.floor(sol * LAMPORTS_PER_SOL);
}
