use anchor_lang::prelude::*;

/// Centralized error codes for the Colosseum protocol.
#[error_code]
pub enum ColosseumError {
    // ── Agent Registry ──────────────────────────────────────────
    #[msg("Agent name too long (max 32 chars)")]
    NameTooLong,

    #[msg("Too many capabilities (max 10)")]
    TooManyCapabilities,

    #[msg("Stake amount too low (min 0.01 SOL)")]
    StakeTooLow,

    #[msg("Agent is not active")]
    AgentNotActive,

    #[msg("Agent is already World ID verified")]
    AlreadyVerified,

    #[msg("World ID verification is required")]
    VerificationRequired,

    #[msg("World ID hash mismatch")]
    WorldIdMismatch,

    #[msg("Invalid agent tier")]
    InvalidTier,

    // ── Job Escrow ──────────────────────────────────────────────
    #[msg("Job payment too low")]
    PaymentTooLow,

    #[msg("Job is not in the required state")]
    JobNotOpen,

    #[msg("Only the assigned worker can submit work")]
    NotAssignedWorker,

    #[msg("Only the job poster can approve work")]
    NotJobPoster,

    #[msg("Job deadline has passed")]
    DeadlinePassed,

    #[msg("Job deadline has not passed yet")]
    DeadlineNotPassed,

    #[msg("Cannot cancel a job after it has been accepted")]
    CannotCancelAfterAccept,

    #[msg("Insufficient funds in escrow")]
    InsufficientEscrow,

    #[msg("Work has not been submitted for this job")]
    JobNotSubmitted,

    // ── Dispute Resolution ──────────────────────────────────────
    #[msg("Dispute has already been resolved")]
    DisputeAlreadyResolved,

    #[msg("Job is not currently disputed")]
    JobNotDisputed,

    #[msg("Invalid dispute resolution")]
    InvalidResolution,

    #[msg("Invalid dispute ruling (1=favor_poster, 2=favor_worker, 3=split)")]
    InvalidDisputeRuling,

    #[msg("Caller is not the job poster or assigned worker")]
    NotDisputeParty,

    // ── Reputation ──────────────────────────────────────────────
    #[msg("Invalid reputation score (1-5)")]
    InvalidScore,

    #[msg("Cannot rate yourself")]
    SelfRating,

    #[msg("Agent has no completed jobs to rate")]
    NoCompletedJobs,

    #[msg("Already rated this agent for this job")]
    AlreadyRated,

    #[msg("Agent is not eligible for a reputation NFT")]
    AgentNotEligibleForNft,

    #[msg("Reputation NFT has already been minted for this agent")]
    NftAlreadyMinted,

    // ── Authorization ───────────────────────────────────────────
    #[msg("Unauthorized: signer is not the authority")]
    Unauthorized,
}
