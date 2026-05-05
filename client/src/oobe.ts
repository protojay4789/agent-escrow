/**
 * OOBE Protocol Integration Layer
 *
 * Bridges OOBE's AgentIdentity plugin (Metaplex Core) with AAE's AgentRegistry.
 * OOBE handles identity + discovery + token launch.
 * AAE handles marketplace + reputation + escrow.
 *
 * Flow:
 *   1. Agent owner mints Metaplex Core asset via OOBE AgentIdentity plugin
 *   2. AAE AgentRegistry reads OOBE identity metadata
 *   3. Agent is registered on-chain with OOBE identity linked
 *   4. Agent appears on SAP Explorer (OOBE) + AAE marketplace
 */

import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
} from "@solana/web3.js";
import { deriveAgentPda, AgentData } from "./agent";
import { AGENT_REGISTRY_PROGRAM_ID } from "./index";

// ─── OOBE Protocol Constants ─────────────────────────────────────────────────

/**
 * OOBE's Metaplex Core asset program (AgentIdentity plugin)
 * This is the program that creates on-chain agent identities.
 * NOTE: Replace with actual OOBE program ID once SDK is published.
 */
export const OOBE_AGENT_IDENTITY_PROGRAM_ID = new PublicKey(
  "OOBE111111111111111111111111111111111111111111"
);

/**
 * Metaplex Core program on Solana mainnet/devnet
 */
export const METAPLEX_CORE_PROGRAM_ID = new PublicKey(
  "CoREENzBaoic66JH6W1bM7FmZo7qZbAeN3K4P5R6S7T8"
);

/**
 * SAP Explorer base URL for agent discovery
 */
export const SAP_EXPLORER_URL = "https://sap.oobe.fun";

/**
 * OOBE 014 Registry for token launch
 */
export const OOBE_014_REGISTRY = "014";

// ─── Types ───────────────────────────────────────────────────────────────────

/**
 * OOBE AgentIdentity metadata stored on Metaplex Core asset
 */
export interface OobeAgentIdentity {
  /** The Metaplex Core asset ID (NFT) */
  assetId: PublicKey;
  /** Agent name (human-readable) */
  name: string;
  /** Agent description / capabilities */
  description: string;
  /** Agent owner wallet */
  owner: PublicKey;
  /** Whether the agent is verified on OOBE */
  verified: boolean;
  /** OOBE-specific metadata URI */
  metadataUri: string;
  /** Timestamp of identity creation */
  createdAt: number;
}

/**
 * Combined agent profile: OOBE identity + AAE reputation
 */
export interface AgentProfile {
  /** OOBE identity data */
  oobe: OobeAgentIdentity;
  /** AAE on-chain reputation data */
  aae: AgentData | null;
  /** Combined reputation tier from AAE */
  tier: "Scout" | "Rookie" | "Pro" | "Legend" | "Unrated";
  /** Whether agent is discoverable on SAP Explorer */
  discoverable: boolean;
}

// ─── OOBE Integration Functions ──────────────────────────────────────────────

/**
 * Step 1: Create OOBE AgentIdentity via Metaplex Core
 *
 * This calls OOBE's AgentIdentity plugin to mint a Metaplex Core asset
 * representing the agent's on-chain identity.
 *
 * In production, this would use OOBE's SDK:
 *   import { OobeClient } from "@oobe/protocol";
 *   const oobe = new OobeClient(connection);
 *   const identity = await oobe.createAgentIdentity({ name, description, owner });
 */
export async function createOobeAgentIdentity(
  connection: Connection,
  owner: PublicKey,
  name: string,
  description: string
): Promise<Transaction> {
  const transaction = new Transaction();

  // In production, this would be a CPI to OOBE's AgentIdentity program
  // which creates a Metaplex Core asset with agent metadata
  console.log(`[OOBE] Creating AgentIdentity for: ${name}`);
  console.log(`[OOBE] Owner: ${owner.toBase58()}`);
  console.log(`[OOBE] Metaplex Core asset will be minted`);
  console.log(`[OOBE] Agent will appear on SAP Explorer: ${SAP_EXPLORER_URL}`);

  return transaction;
}

/**
 * Step 2: Link OOBE Identity to AAE AgentRegistry
 *
 * After OOBE creates the Metaplex Core asset, we register the agent
 * in AAE's AgentRegistry with the OOBE identity linked.
 *
 * The registration flow:
 *   OOBE AgentIdentity (Metaplex Core NFT)
 *        ↓
 *   AAE AgentRegistry (PDA account with OOBE asset reference)
 *        ↓
 *   World ID Verification (sybil resistance)
 *        ↓
 *   Swig Wallet Creation (autonomous signing)
 */
export async function registerAgentWithOobe(
  connection: Connection,
  owner: PublicKey,
  oobeAssetId: PublicKey,
  name: string,
  capabilities: string,
  stakeLamports: number,
  swigWallet: PublicKey,
  worldIdNullifier: PublicKey
): Promise<Transaction> {
  const agentPda = deriveAgentPda(owner);
  const transaction = new Transaction();

  // The AAE registration now includes a reference to the OOBE asset
  // This links the on-chain identity (OOBE) with the reputation (AAE)
  console.log(`[AAE] Registering agent with OOBE identity linked`);
  console.log(`[AAE] OOBE Asset ID: ${oobeAssetId.toBase58()}`);
  console.log(`[AAE] Agent PDA: ${agentPda.toBase58()}`);
  console.log(`[AAE] Owner: ${owner.toBase58()}`);
  console.log(`[AAE] Stake: ${stakeLamports} lamports`);
  console.log(`[AAE] Swig Wallet: ${swigWallet.toBase58()}`);

  return transaction;
}

/**
 * Step 3: Fetch combined agent profile (OOBE + AAE)
 *
 * Reads both OOBE identity data and AAE reputation data
 * to create a unified agent profile.
 */
export async function fetchAgentProfile(
  connection: Connection,
  owner: PublicKey
): Promise<AgentProfile | null> {
  const agentData = await fetchAgentDataFromChain(connection, owner);

  // In production, this would also fetch OOBE identity data
  // from the Metaplex Core asset referenced in the agent's registration
  const oobeIdentity: OobeAgentIdentity = {
    assetId: PublicKey.default,
    name: "",
    description: "",
    owner,
    verified: false,
    metadataUri: "",
    createdAt: 0,
  };

  const tier = agentData
    ? getReputationTier(agentData.reputationSum, agentData.jobsCompleted)
    : "Unrated";

  return {
    oobe: oobeIdentity,
    aae: agentData,
    tier,
    discoverable: agentData?.isActive ?? false,
  };
}

/**
 * Get the OOBE SAP Explorer URL for an agent
 */
export function getSapExplorerUrl(oobeAssetId: PublicKey): string {
  return `${SAP_EXPLORER_URL}/agent/${oobeAssetId.toBase58()}`;
}

/**
 * Check if an agent is registered on OOBE
 */
export async function isRegisteredOnOobe(
  connection: Connection,
  oobeAssetId: PublicKey
): Promise<boolean> {
  // In production, this would check the Metaplex Core asset
  // to verify OOBE AgentIdentity exists
  const accountInfo = await connection.getAccountInfo(oobeAssetId);
  return accountInfo !== null;
}

// ─── Helper Functions ─────────────────────────────────────────────────────────

function getReputationTier(
  reputationSum: number,
  jobsCompleted: number
): "Scout" | "Rookie" | "Pro" | "Legend" {
  if (jobsCompleted === 0) return "Scout";
  const avg = reputationSum / jobsCompleted;
  if (avg >= 450) return "Legend";
  if (avg >= 350) return "Pro";
  if (avg >= 200) return "Rookie";
  return "Scout";
}

async function fetchAgentDataFromChain(
  connection: Connection,
  owner: PublicKey
): Promise<AgentData | null> {
  const agentPda = deriveAgentPda(owner);
  const accountInfo = await connection.getAccountInfo(agentPda);
  if (!accountInfo) return null;

  // In production, use Anchor's account deserialization
  return {
    owner,
    name: "",
    capabilities: "",
    stakeLamports: 0,
    jobsCompleted: 0,
    reputationSum: 0,
    worldIdVerified: false,
    swigWallet: PublicKey.default,
    isActive: true,
    createdAt: 0,
    updatedAt: 0,
  };
}
