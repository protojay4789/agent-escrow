/**
 * Reputation client for interacting with the Reputation program
 */

import {
  Connection,
  PublicKey,
  Transaction,
} from "@solana/web3.js";
import {
  REPUTATION_PROGRAM_ID,
  RATING_SEED,
  REPUTATION_NFT_SEED,
  ReputationTier,
  derivePda,
} from "./index";

/**
 * Rating account data as stored on-chain
 */
export interface RatingData {
  jobId: number;
  agent: PublicKey;
  rater: PublicKey;
  score: number;
  review: string;
  createdAt: number;
}

/**
 * Reputation NFT data
 */
export interface ReputationNftData {
  agent: PublicKey;
  tier: ReputationTier;
  totalRatings: number;
  totalScoreSum: number;
  assetId: PublicKey;
  mintedAt: number;
  updatedAt: number;
}

/**
 * Derive the rating PDA for a job-agent pair
 */
export function deriveRatingPda(jobId: number, agent: PublicKey): PublicKey {
  const jobIdBytes = Buffer.alloc(8);
  jobIdBytes.writeBigUInt64LE(BigInt(jobId));
  const [pda] = derivePda(
    [RATING_SEED, jobIdBytes, agent.toBuffer()],
    REPUTATION_PROGRAM_ID
  );
  return pda;
}

/**
 * Derive the reputation NFT PDA for an agent
 */
export function deriveReputationNftPda(agent: PublicKey): PublicKey {
  const [pda] = derivePda(
    [REPUTATION_NFT_SEED, agent.toBuffer()],
    REPUTATION_PROGRAM_ID
  );
  return pda;
}

/**
 * Create a rate_agent instruction
 */
export async function createRateAgentInstruction(
  rater: PublicKey,
  jobId: number,
  agent: PublicKey,
  score: number,
  review: string
): Promise<Transaction> {
  const ratingPda = deriveRatingPda(jobId, agent);
  const reputationNftPda = deriveReputationNftPda(agent);

  const reviewBytes = Buffer.alloc(256);
  reviewBytes.write(review);

  const transaction = new Transaction();

  console.log(`Rating agent for job #${jobId}`);
  console.log(`  Rater: ${rater.toBase58()}`);
  console.log(`  Agent: ${agent.toBase58()}`);
  console.log(`  Score: ${(score / 100).toFixed(2)}`);
  console.log(`  Rating PDA: ${ratingPda.toBase58()}`);
  console.log(`  Reputation NFT PDA: ${reputationNftPda.toBase58()}`);

  return transaction;
}

/**
 * Create an update_reputation instruction
 */
export async function createUpdateReputationInstruction(
  agent: PublicKey
): Promise<Transaction> {
  const reputationNftPda = deriveReputationNftPda(agent);
  const transaction = new Transaction();

  console.log(`Updating reputation for agent: ${agent.toBase58()}`);
  console.log(`  Reputation NFT PDA: ${reputationNftPda.toBase58()}`);

  return transaction;
}

/**
 * Create a mint_reputation_nft instruction
 */
export async function createMintNftInstruction(
  payer: PublicKey,
  agent: PublicKey
): Promise<Transaction> {
  const reputationNftPda = deriveReputationNftPda(agent);
  const transaction = new Transaction();

  console.log(`Minting soulbound reputation NFT for agent: ${agent.toBase58()}`);
  console.log(`  Payer: ${payer.toBase58()}`);
  console.log(`  Reputation NFT PDA: ${reputationNftPda.toBase58()}`);

  return transaction;
}

/**
 * Fetch reputation NFT data from on-chain account
 */
export async function fetchReputationNftData(
  connection: Connection,
  agent: PublicKey
): Promise<ReputationNftData | null> {
  const nftPda = deriveReputationNftPda(agent);
  const accountInfo = await connection.getAccountInfo(nftPda);

  if (!accountInfo) {
    return null;
  }

  // Placeholder - in production, deserialize with Anchor
  return {
    agent,
    tier: ReputationTier.Scout,
    totalRatings: 0,
    totalScoreSum: 0,
    assetId: PublicKey.default,
    mintedAt: 0,
    updatedAt: 0,
  };
}

/**
 * Calculate average reputation score from total score sum and count
 */
export function calculateAverageReputation(
  totalScoreSum: number,
  totalRatings: number
): number {
  if (totalRatings === 0) return 0;
  return totalScoreSum / totalRatings / 100; // Convert from scaled (100-500) to decimal (1.00-5.00)
}

/**
 * Get tier from average score
 */
export function getTierFromAverage(average: number): ReputationTier {
  if (average >= 4.5) return ReputationTier.Legend;
  if (average >= 3.5) return ReputationTier.Pro;
  if (average >= 2.0) return ReputationTier.Rookie;
  return ReputationTier.Scout;
}
