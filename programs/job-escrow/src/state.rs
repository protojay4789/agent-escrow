use anchor_lang::prelude::*;

/// Maximum length for job description (bytes)
pub const MAX_DESCRIPTION_LENGTH: usize = 512;
/// Maximum length for requirements (bytes)
pub const MAX_REQUIREMENTS_LENGTH: usize = 1024;
/// Maximum length for deliverable URL/hash (bytes)
pub const MAX_DELIVERABLE_LENGTH: usize = 256;

/// PDA seed prefixes
pub const JOB_SEED: &[u8] = b"job";
pub const JOB_COUNTER_SEED: &[u8] = b"job_counter";
pub const ESCROW_SEED: &[u8] = b"escrow";

/// Job lifecycle states (8 states)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub enum JobStatus {
    /// Job posted, waiting for a worker to accept
    Open,
    /// A worker has accepted the job, escrow is locked
    Accepted,
    /// Worker has submitted deliverables, waiting for review
    Submitted,
    /// Poster has approved the work, funds released to worker
    Approved,
    /// Either party has disputed the job
    Disputed,
    /// Job has expired (deadline passed), eligible for refund
    Expired,
    /// Funds refunded to the poster (after expiry or dispute resolution)
    Refunded,
    /// Job completed, funds transferred to worker
    Completed,
}

impl JobStatus {
    pub fn can_accept(&self) -> bool {
        *self == JobStatus::Open
    }

    pub fn can_submit(&self) -> bool {
        *self == JobStatus::Accepted
    }

    pub fn can_approve(&self) -> bool {
        *self == JobStatus::Submitted
    }

    pub fn can_dispute(&self) -> bool {
        matches!(
            *self,
            JobStatus::Submitted | JobStatus::Accepted
        )
    }

    pub fn can_refund(&self) -> bool {
        *self == JobStatus::Expired
    }

    pub fn can_cancel(&self) -> bool {
        matches!(*self, JobStatus::Open | JobStatus::Accepted)
    }
}

/// On-chain account representing a job escrow
#[account]
#[derive(InitSpace)]
pub struct JobAccount {
    /// Unique job identifier (monotonically increasing)
    pub job_id: u64,
    /// The wallet that posted and funded the job
    pub poster: Pubkey,
    /// The worker who accepted the job (zero key if unassigned)
    pub worker: Pubkey,
    /// Job description
    pub description: [u8; MAX_DESCRIPTION_LENGTH],
    pub description_len: u16,
    /// Requirements for the job
    pub requirements: [u8; MAX_REQUIREMENTS_LENGTH],
    pub requirements_len: u16,
    /// Payment amount in lamports
    pub payment_lamports: u64,
    /// Deadline (Unix timestamp) - auto-refund after this time
    pub deadline: i64,
    /// Current status of the job
    pub status: JobStatus,
    /// Deliverable URL or IPFS hash
    pub deliverable: [u8; MAX_DELIVERABLE_LENGTH],
    pub deliverable_len: u16,
    /// Timestamp when the job was created
    pub created_at: i64,
    /// Timestamp of last status update
    pub updated_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

/// Global job counter for unique IDs
#[account]
#[derive(InitSpace)]
pub struct JobCounter {
    pub count: u64,
    pub bump: u8,
}

/// Escrow vault PDA (holds the locked funds)
#[account]
#[derive(InitSpace)]
pub struct EscrowVault {
    /// The job this vault belongs to
    pub job: Pubkey,
    /// Amount of lamports held
    pub amount: u64,
    /// Whether the vault has been claimed
    pub claimed: bool,
    pub bump: u8,
}
