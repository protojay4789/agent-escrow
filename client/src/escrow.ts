/**
 * Job Escrow client for interacting with the JobEscrow program
 */

import {
  Connection,
  PublicKey,
  Transaction,
  SystemProgram,
} from "@solana/web3.js";
import {
  JOB_ESCROW_PROGRAM_ID,
  JOB_SEED,
  JOB_COUNTER_SEED,
  ESCROW_SEED,
  JobStatus,
  derivePda,
} from "./index";

/**
 * Job account data as stored on-chain
 */
export interface JobData {
  jobId: number;
  poster: PublicKey;
  worker: PublicKey;
  description: string;
  requirements: string;
  paymentLamports: number;
  deadline: number;
  status: JobStatus;
  deliverable: string;
  createdAt: number;
  updatedAt: number;
}

/**
 * Derive the job PDA for a given job ID
 */
export function deriveJobPda(jobId: number): PublicKey {
  const jobIdBytes = Buffer.alloc(8);
  jobIdBytes.writeBigUInt64LE(BigInt(jobId));
  const [pda] = derivePda([JOB_SEED, jobIdBytes], JOB_ESCROW_PROGRAM_ID);
  return pda;
}

/**
 * Derive the job counter PDA
 */
export function deriveJobCounterPda(): PublicKey {
  const [pda] = derivePda([JOB_COUNTER_SEED], JOB_ESCROW_PROGRAM_ID);
  return pda;
}

/**
 * Derive the escrow vault PDA for a given job ID
 */
export function deriveEscrowVaultPda(jobId: number): PublicKey {
  const jobIdBytes = Buffer.alloc(8);
  jobIdBytes.writeBigUInt64LE(BigInt(jobId));
  const [pda] = derivePda([ESCROW_SEED, jobIdBytes], JOB_ESCROW_PROGRAM_ID);
  return pda;
}

/**
 * Create a post_job instruction
 */
export async function createPostJobInstruction(
  poster: PublicKey,
  jobId: number,
  description: string,
  requirements: string,
  paymentLamports: number,
  deadline: number
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const counterPda = deriveJobCounterPda();
  const escrowPda = deriveEscrowVaultPda(jobId);

  const descBytes = Buffer.alloc(512);
  descBytes.write(description);
  const reqBytes = Buffer.alloc(1024);
  reqBytes.write(requirements);

  const transaction = new Transaction();

  console.log(`Posting job #${jobId}`);
  console.log(`  Poster: ${poster.toBase58()}`);
  console.log(`  Job PDA: ${jobPda.toBase58()}`);
  console.log(`  Escrow PDA: ${escrowPda.toBase58()}`);
  console.log(`  Payment: ${paymentLamports} lamports`);
  console.log(`  Deadline: ${new Date(deadline * 1000).toISOString()}`);

  return transaction;
}

/**
 * Create an accept_job instruction
 */
export async function createAcceptJobInstruction(
  worker: PublicKey,
  jobId: number
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const transaction = new Transaction();

  console.log(`Accepting job #${jobId}`);
  console.log(`  Worker: ${worker.toBase58()}`);
  console.log(`  Job PDA: ${jobPda.toBase58()}`);

  return transaction;
}

/**
 * Create a submit_work instruction
 */
export async function createSubmitWorkInstruction(
  worker: PublicKey,
  jobId: number,
  deliverable: string
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const delivBytes = Buffer.alloc(256);
  delivBytes.write(deliverable);

  const transaction = new Transaction();

  console.log(`Submitting work for job #${jobId}`);
  console.log(`  Worker: ${worker.toBase58()}`);
  console.log(`  Deliverable: ${deliverable}`);

  return transaction;
}

/**
 * Create an approve_work instruction
 */
export async function createApproveWorkInstruction(
  poster: PublicKey,
  jobId: number,
  worker: PublicKey
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const escrowPda = deriveEscrowVaultPda(jobId);
  const transaction = new Transaction();

  console.log(`Approving work for job #${jobId}`);
  console.log(`  Poster: ${poster.toBase58()}`);
  console.log(`  Worker: ${worker.toBase58()}`);
  console.log(`  Escrow PDA: ${escrowPda.toBase58()}`);

  return transaction;
}

/**
 * Create a cancel_job instruction
 */
export async function createCancelJobInstruction(
  poster: PublicKey,
  jobId: number
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const escrowPda = deriveEscrowVaultPda(jobId);
  const transaction = new Transaction();

  console.log(`Cancelling job #${jobId}`);
  console.log(`  Poster: ${poster.toBase58()}`);
  console.log(`  Refunding from: ${escrowPda.toBase58()}`);

  return transaction;
}

/**
 * Create a refund_expired instruction
 */
export async function createRefundExpiredInstruction(
  poster: PublicKey,
  jobId: number
): Promise<Transaction> {
  const jobPda = deriveJobPda(jobId);
  const escrowPda = deriveEscrowVaultPda(jobId);
  const transaction = new Transaction();

  console.log(`Refunding expired job #${jobId}`);
  console.log(`  Poster: ${poster.toBase58()}`);
  console.log(`  Escrow PDA: ${escrowPda.toBase58()}`);

  return transaction;
}

/**
 * Fetch job data from on-chain account
 */
export async function fetchJobData(
  connection: Connection,
  jobId: number
): Promise<JobData | null> {
  const jobPda = deriveJobPda(jobId);
  const accountInfo = await connection.getAccountInfo(jobPda);

  if (!accountInfo) {
    return null;
  }

  // Placeholder - in production, deserialize with Anchor
  return {
    jobId,
    poster: PublicKey.default,
    worker: PublicKey.default,
    description: "",
    requirements: "",
    paymentLamports: 0,
    deadline: 0,
    status: JobStatus.Open,
    deliverable: "",
    createdAt: 0,
    updatedAt: 0,
  };
}
