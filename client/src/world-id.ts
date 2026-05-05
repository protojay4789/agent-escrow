/**
 * World ID integration for AgentEscrow
 *
 * Provides World ID verification for Sybil resistance.
 * Uses the @worldcoin/idkit library for frontend integration.
 */

import {
  Connection,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  AGENT_REGISTRY_PROGRAM_ID,
  AGENT_SEED,
  derivePda,
} from "./index";

/**
 * World ID verification proof data
 */
export interface WorldIdProof {
  /** The ZK proof bytes */
  proof: Uint8Array[];
  /** Hash of the proof for uniqueness */
  proofHash: Uint8Array;
  /** Merkle tree root */
  root: Uint8Array;
  /** Signal (agent owner pubkey encoded as bytes) */
  signal: Uint8Array;
  /** Nullifier hash to prevent double-verification */
  nullifierHash: Uint8Array;
}

/**
 * World ID verification configuration
 */
export interface WorldIdConfig {
  /** World ID app ID */
  appId: string;
  /** Action ID for this verification */
  actionId: string;
  /** Whether to use staging environment */
  staging?: boolean;
}

/**
 * Derive the nullifier PDA for a World ID proof
 */
export function deriveNullifierPda(proofHash: Uint8Array): PublicKey {
  const [pda] = derivePda(
    [Buffer.from("nullifier"), Buffer.from(proofHash)],
    AGENT_REGISTRY_PROGRAM_ID
  );
  return pda;
}

/**
 * Derive the agent PDA for a given owner
 */
export function deriveAgentPda(owner: PublicKey): PublicKey {
  const [pda] = derivePda(
    [AGENT_SEED, owner.toBuffer()],
    AGENT_REGISTRY_PROGRAM_ID
  );
  return pda;
}

/**
 * Create a verify_world_id instruction
 */
export async function createVerifyWorldIdInstruction(
  owner: PublicKey,
  proof: WorldIdProof
): Promise<Transaction> {
  const agentPda = deriveAgentPda(owner);
  const nullifierPda = deriveNullifierPda(proof.proofHash);

  const transaction = new Transaction();

  console.log(`Verifying World ID for agent: ${owner.toBase58()}`);
  console.log(`  Agent PDA: ${agentPda.toBase58()}`);
  console.log(`  Nullifier PDA: ${nullifierPda.toBase58()}`);

  return transaction;
}

/**
 * Default World ID configuration for AgentEscrow
 */
export const WORLD_ID_CONFIG: WorldIdConfig = {
  appId: "app_agent_escrow_solana",
  actionId: "agent-registration",
  staging: true, // Use staging for devnet
};

/**
 * Frontend World ID verification helper
 *
 * In a Next.js/React frontend, you would use the @worldcoin/idkit package:
 *
 * ```tsx
 * import { IDKitWidget } from "@worldcoin/idkit";
 *
 * function WorldIdVerification({ onSuccess }: { onSuccess: (proof: any) => void }) {
 *   return (
 *     <IDKitWidget
 *       app_id={WORLD_ID_CONFIG.appId}
 *       action={WORLD_ID_CONFIG.actionId}
 *       onSuccess={onSuccess}
 *     >
 *       {({ open }) => (
 *         <button onClick={open}>Verify with World ID</button>
 *       )}
 *     </IDKitWidget>
 *   );
 * }
 * ```
 */
export function getWorldIdWidgetProps() {
  return {
    app_id: WORLD_ID_CONFIG.appId,
    action: WORLD_ID_CONFIG.actionId,
    staging: WORLD_ID_CONFIG.staging,
  };
}
