/**
 * AgentEscrow Client - Solana
 *
 * Main entry point for the AgentEscrow client library.
 * Provides program IDL imports and connection helpers.
 *
 * Integrates with:
 *   - OOBE Protocol: AgentIdentity plugin + SAP Explorer + 014 token registry
 *   - Metaplex Core: NFT minting for identity + reputation
 *   - Swig: Programmable agent wallets
 *   - World: Sybil-resistant identity verification
 *   - x402: Agent-to-agent micropayments
 */

import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";

// Program IDs (devnet)
export const AGENT_REGISTRY_PROGRAM_ID = new PublicKey(
  "4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn"
);
export const JOB_ESCROW_PROGRAM_ID = new PublicKey(
  "4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn"
);
export const REPUTATION_PROGRAM_ID = new PublicKey(
  "4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn"
);
export const DISPUTE_RESOLVER_PROGRAM_ID = new PublicKey(
  "4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn"
);

// PDA Seeds
export const AGENT_SEED = Buffer.from("agent");
export const JOB_SEED = Buffer.from("job");
export const JOB_COUNTER_SEED = Buffer.from("job_counter");
export const ESCROW_SEED = Buffer.from("escrow");
export const RATING_SEED = Buffer.from("rating");
export const REPUTATION_NFT_SEED = Buffer.from("rep_nft");
export const DISPUTE_SEED = Buffer.from("dispute");
export const EVIDENCE_SEED = Buffer.from("evidence");

/**
 * Create a connection to Solana devnet
 */
export function getDevnetConnection(): Connection {
  return new Connection(clusterApiUrl("devnet"), "confirmed");
}

/**
 * Derive a PDA from seeds and program ID
 */
export function derivePda(
  seeds: (Buffer | Uint8Array)[],
  programId: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

/**
 * Job status enum values matching the on-chain enum
 */
export enum JobStatus {
  Open = 0,
  Accepted = 1,
  Submitted = 2,
  Approved = 3,
  Disputed = 4,
  Expired = 5,
  Refunded = 6,
  Completed = 7,
}

/**
 * Reputation tier enum values
 */
export enum ReputationTier {
  Scout = 0,
  Rookie = 1,
  Pro = 2,
  Legend = 3,
}

/**
 * Dispute status enum values
 */
export enum DisputeStatus {
  Open = 0,
  UnderReview = 1,
  Resolved = 2,
}
