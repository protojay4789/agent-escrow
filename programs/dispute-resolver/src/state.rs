use anchor_lang::prelude::*;

/// Maximum lengths
pub const MAX_REASON_LENGTH: usize = 512;
pub const MAX_RESOLUTION_LENGTH: usize = 1024;
pub const MAX_EVIDENCE_LENGTH: usize = 256;
pub const MAX_EVIDENCE_URL_LENGTH: usize = 256;
pub const MAX_EVIDENCE_COUNT: usize = 10;

/// PDA seed prefixes
pub const DISPUTE_SEED: &[u8] = b"dispute";
pub const DISPUTE_COUNTER_SEED: &[u8] = b"dispute_counter";
pub const EVIDENCE_SEED: &[u8] = b"evidence";

/// Dispute status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub enum DisputeStatus {
    /// Dispute has been created, evidence gathering phase
    Open,
    /// Evidence submitted by both parties, awaiting resolution
    UnderReview,
    /// Dispute resolved by the resolver/DAO
    Resolved,
}

/// Resolution outcome
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug, InitSpace)]
pub enum ResolutionOutcome {
    /// Poster wins, funds returned to poster
    PosterWins,
    /// Worker wins, funds released to worker
    WorkerWins,
    /// Split decision, funds divided between poster and worker
    Split { poster_share_bps: u16 }, // basis points (0-10000)
    /// No resolution, dispute dismissed
    Dismissed,
}

/// On-chain account representing a dispute
#[account]
#[derive(InitSpace)]
pub struct DisputeAccount {
    /// The job this dispute is about
    pub job: Pubkey,
    /// Who initiated the dispute
    pub initiator: Pubkey,
    /// The poster of the original job
    pub poster: Pubkey,
    /// The worker of the original job
    pub worker: Pubkey,
    /// Reason for the dispute
    pub reason: [u8; MAX_REASON_LENGTH],
    pub reason_len: u16,
    /// Resolution description (set when resolved)
    pub resolution: [u8; MAX_RESOLUTION_LENGTH],
    pub resolution_len: u16,
    /// Outcome of the resolution
    pub outcome: Option<ResolutionOutcome>,
    /// Current status
    pub status: DisputeStatus,
    /// Whether the dispute has been resolved
    pub resolved: bool,
    /// Timestamp when the dispute was created
    pub created_at: i64,
    /// Timestamp when the dispute was resolved
    pub resolved_at: i64,
    /// PDA bump
    pub bump: u8,
}

/// Global dispute counter
#[account]
#[derive(InitSpace)]
pub struct DisputeCounter {
    pub count: u64,
    pub bump: u8,
}

/// Individual evidence submission
#[account]
#[derive(InitSpace)]
pub struct EvidenceAccount {
    /// The dispute this evidence belongs to
    pub dispute: Pubkey,
    /// Who submitted the evidence
    pub submitter: Pubkey,
    /// Evidence description or hash
    pub content: [u8; MAX_EVIDENCE_LENGTH],
    pub content_len: u16,
    /// URL to supporting document/IPFS hash
    pub url: [u8; MAX_EVIDENCE_URL_LENGTH],
    pub url_len: u16,
    /// Timestamp
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}
