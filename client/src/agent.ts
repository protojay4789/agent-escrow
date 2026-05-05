/**
 * Agent Registry client for interacting with the AgentRegistry program
 *
 * Integrates with OOBE Protocol's AgentIdentity plugin (Metaplex Core).
 * OOBE provides on-chain identity + SAP Explorer discovery.
 * AAE provides reputation + escrow + dispute resolution.
 */

import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
} from "@solana/web3.js";
import {
  AGENT_REGISTRY_PROGRAM_ID,
  AGENT_SEED,
  derivePda,
} from "./index";
// OOBE Protocol constants (AgentIdentity program)
// See: github.com/metaplex-foundation/mpl-agent (MIP-014)
export const OOBE_AGENT_IDENTITY_PROGRAM_ID = new PublicKey(
  "1DREGFgysWYxLnRnKQnwrxnJQeSMk2HmGaC6whw2B2p"
);

/** Metaplex Core program ID */
export const MPL_CORE_PROGRAM_ID = new PublicKey(
  "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
);

/**
 * Agent account data as stored on-chain
 */
export interface AgentData {
  owner: PublicKey;
  name: string;
  capabilities: string;
  stakeLamports: number;
  jobsCompleted: number;
  reputationSum: number;
  worldIdVerified: boolean;
  swigWallet: PublicKey;
  isActive: boolean;
  createdAt: number;
  updatedAt: number;
  /** OOBE AgentIdentity asset ID (Metaplex Core) — links to on-chain identity */
  oobeAssetId?: PublicKey;
  /** Metaplex Core asset pubkey holding the AgentIdentity plugin (OOBE Protocol) */
  coreAsset?: PublicKey;
}

/**
 * Derive the agent PDA for a given owner
 */
export function deriveAgentPda(owner: PublicKey): PublicKey {
  const [pda] = derivePda([AGENT_SEED, owner.toBuffer()], AGENT_REGISTRY_PROGRAM_ID);
  return pda;
}

/**
 * Create a register_agent instruction
 */
export async function createRegisterAgentInstruction(
  owner: PublicKey,
  name: string,
  capabilities: string,
  stakeLamports: number,
  swigWallet: PublicKey,
  worldIdNullifier: PublicKey
): Promise<Transaction> {
  const agentPda = deriveAgentPda(owner);

  // Pad name and capabilities to fixed-size arrays
  const nameBytes = Buffer.alloc(64);
  nameBytes.write(name);
  const capabilitiesBytes = Buffer.alloc(256);
  capabilitiesBytes.write(capabilities);

  // In production, this would build the Anchor instruction with proper accounts
  const transaction = new Transaction();

  // Add the register_agent instruction
  // This is a placeholder - actual implementation would use Anchor's instruction builder
  console.log(`Registering agent: ${name}`);
  console.log(`  Owner: ${owner.toBase58()}`);
  console.log(`  Agent PDA: ${agentPda.toBase58()}`);
  console.log(`  Stake: ${stakeLamports} lamports`);
  console.log(`  Swig Wallet: ${swigWallet.toBase58()}`);

  return transaction;
}

/**
 * Create an update_agent instruction
 */
export async function createUpdateAgentInstruction(
  owner: PublicKey,
  name: string,
  capabilities: string
): Promise<Transaction> {
  const agentPda = deriveAgentPda(owner);

  const nameBytes = Buffer.alloc(64);
  nameBytes.write(name);
  const capabilitiesBytes = Buffer.alloc(256);
  capabilitiesBytes.write(capabilities);

  const transaction = new Transaction();

  console.log(`Updating agent: ${name}`);
  console.log(`  Owner: ${owner.toBase58()}`);
  console.log(`  Agent PDA: ${agentPda.toBase58()}`);

  return transaction;
}

/**
 * Create a deactivate_agent instruction
 */
export async function createDeactivateAgentInstruction(
  owner: PublicKey
): Promise<Transaction> {
  const agentPda = deriveAgentPda(owner);
  const transaction = new Transaction();

  console.log(`Deactivating agent: owner=${owner.toBase58()}`);
  console.log(`  Agent PDA: ${agentPda.toBase58()}`);

  return transaction;
}

/**
 * Fetch agent data from on-chain account
 */
export async function fetchAgentData(
  connection: Connection,
  owner: PublicKey
): Promise<AgentData | null> {
  const agentPda = deriveAgentPda(owner);
  const accountInfo = await connection.getAccountInfo(agentPda);

  if (!accountInfo) {
    return null;
  }

  // In production, this would use Anchor's account deserialization
  // For now, return a mock structure
  return {
    owner,
    name: "",
    capabilities: "",
    stakeLamports: 0,
    jobsCompleted: 0,
    reputationSum: 0,
    worldIdVerified: false,
    swigWallet: PublicKey.default,
    coreAsset: PublicKey.default,
    isActive: true,
    createdAt: 0,
    updatedAt: 0,
  };
}

/**
 * Build instruction data for link_metaplex_identity
 * This creates a TransactionInstruction that calls the agent_registry program's
 * link_metaplex_identity instruction with the given core_asset and registration URI.
 */
export function createLinkMetaplexIdentityInstruction(
  owner: PublicKey,
  coreAsset: PublicKey,
  agentRegistrationUri: string
): TransactionInstruction {
  const agentPda = deriveAgentPda(owner);

  // Anchor instruction discriminator for "link_metaplex_identity"
  // SHA256("global:link_metaplex_identity")[..8]
  const instructionDiscriminator = Buffer.from([
    0x1b, 0x4a, 0x2c, 0x6b, 0xf3, 0x3f, 0x9e, 0x7d,
  ]);

  // Serialize agent_registration_uri as a Borsh String (4-byte length prefix + UTF-8 bytes)
  const uriBytes = Buffer.from(agentRegistrationUri, "utf-8");
  const uriLength = Buffer.alloc(4);
  uriLength.writeUInt32LE(uriBytes.length);

  const instructionData = Buffer.concat([
    instructionDiscriminator,
    uriLength,
    uriBytes,
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: agentPda, isSigner: false, isWritable: true },
      { pubkey: coreAsset, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: AGENT_REGISTRY_PROGRAM_ID,
    data: instructionData,
  });
}

/**
 * Register an agent and link a Metaplex Core identity in a single transaction
 */
export async function registerAndLinkIdentity(
  connection: Connection,
  owner: PublicKey,
  name: string,
  capabilities: string,
  stakeLamports: number,
  swigWallet: PublicKey,
  worldIdNullifier: PublicKey,
  coreAsset: PublicKey,
  agentRegistrationUri: string
): Promise<Transaction> {
  const registerTx = await createRegisterAgentInstruction(
    owner,
    name,
    capabilities,
    stakeLamports,
    swigWallet,
    worldIdNullifier
  );

  const linkIx = createLinkMetaplexIdentityInstruction(
    owner,
    coreAsset,
    agentRegistrationUri
  );

  const transaction = new Transaction();
  // Add all instructions from the registration transaction
  transaction.add(...registerTx.instructions);
  // Add the link identity instruction
  transaction.add(linkIx);

  // Set recent blockhash and fee payer
  const { blockhash } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = owner;

  console.log(
    `Registering agent and linking identity: owner=${owner.toBase58()}, coreAsset=${coreAsset.toBase58()}`
  );

  return transaction;
}
