use anchor_lang::prelude::*;

pub mod agent_registry;
pub mod dispute_resolver;
pub mod job_escrow;
pub mod reputation;

pub use agent_registry::*;
pub use dispute_resolver::*;
pub use job_escrow::*;
pub use reputation::*;

// Re-export errors under `crate::errors` path for modules that import via that path
pub mod errors {
    pub use crate::ColosseumError;
}

declare_id!("4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn");

#[program]
pub mod colosseum_programs {
    use super::*;

    // ── Agent Registry ──────────────────────────────────────────
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        name: String,
        capabilities: Vec<String>,
        stake_amount: u64,
    ) -> Result<()> {
        agent_registry::register_agent(ctx, name, capabilities, stake_amount)
    }

    pub fn update_agent(
        ctx: Context<UpdateAgent>,
        capabilities: Option<Vec<String>>,
        active: Option<bool>,
    ) -> Result<()> {
        agent_registry::update_agent(ctx, capabilities, active)
    }

    pub fn verify_agent(
        ctx: Context<VerifyAgent>,
        world_id_hash: [u8; 32],
        swig_wallet: Pubkey,
    ) -> Result<()> {
        agent_registry::verify_agent(ctx, world_id_hash, swig_wallet)
    }

    // ── Job Escrow ──────────────────────────────────────────────
    pub fn post_job(
        ctx: Context<PostJob>,
        job_id: String,
        description: String,
        requirements: Vec<String>,
        payment: u64,
        deadline: i64,
    ) -> Result<()> {
        job_escrow::post_job(ctx, job_id, description, requirements, payment, deadline)
    }

    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        job_escrow::accept_job(ctx)
    }

    pub fn submit_work(ctx: Context<SubmitWork>, deliverable: String) -> Result<()> {
        job_escrow::submit_work(ctx, deliverable)
    }

    pub fn approve_work(ctx: Context<ApproveWork>) -> Result<()> {
        job_escrow::approve_work(ctx)
    }

    pub fn dispute_job(ctx: Context<DisputeJob>, reason: String) -> Result<()> {
        job_escrow::dispute_job(ctx, reason)
    }

    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        job_escrow::cancel_job(ctx)
    }

    pub fn expire_job(ctx: Context<ExpireJob>) -> Result<()> {
        job_escrow::expire_job(ctx)
    }

    // ── Dispute Resolver ────────────────────────────────────────
    pub fn raise_dispute(ctx: Context<RaiseDispute>, reason: String) -> Result<()> {
        dispute_resolver::raise_dispute(ctx, reason)
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        resolution: String,
        ruling: u8,
    ) -> Result<()> {
        dispute_resolver::resolve_dispute(ctx, resolution, ruling)
    }

    // ── Reputation ──────────────────────────────────────────────
    pub fn rate_agent(
        ctx: Context<RateAgent>,
        agent_pubkey: Pubkey,
        job_ref: String,
        score: u8,
        review: String,
    ) -> Result<()> {
        reputation::rate_agent(ctx, agent_pubkey, job_ref, score, review)
    }

    pub fn get_reputation(ctx: Context<GetReputation>) -> Result<()> {
        reputation::get_reputation(ctx)
    }

    pub fn mint_reputation_nft(ctx: Context<MintReputationNft>) -> Result<()> {
        reputation::mint_reputation_nft(ctx)
    }
}

// ── Error Codes ─────────────────────────────────────────────────
#[error_code]
pub enum ColosseumError {
    #[msg("Agent name too long (max 32 chars)")]
    NameTooLong,
    #[msg("Too many capabilities (max 10)")]
    TooManyCapabilities,
    #[msg("Stake amount too low (min 0.01 SOL)")]
    StakeTooLow,
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
    #[msg("Invalid reputation score (1-5)")]
    InvalidScore,
    #[msg("Cannot rate yourself")]
    SelfRating,
    #[msg("Agent is not active")]
    AgentNotActive,
    #[msg("Job deadline has not passed yet")]
    DeadlineNotPassed,
    #[msg("Caller is not the job poster or assigned worker")]
    NotDisputeParty,
    #[msg("Dispute has already been resolved")]
    DisputeAlreadyResolved,
    #[msg("Invalid dispute ruling (1=favor_poster, 2=favor_worker, 3=split)")]
    InvalidDisputeRuling,
    #[msg("Agent not eligible for NFT (must have completed at least 1 job)")]
    AgentNotEligibleForNft,
    #[msg("NFT already minted for this agent")]
    NftAlreadyMinted,
    #[msg("Unauthorized action")]
    Unauthorized,
    #[msg("Agent already verified")]
    AlreadyVerified,
}
