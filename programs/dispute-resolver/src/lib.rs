use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("wtPxD8z3K2C515cZaB5CCN4BNmuX4ahhNpvYJn2HCo1");

#[program]
pub mod dispute_resolver {
    use super::*;

    /// Create a new dispute for a job
    pub fn create_dispute(
        ctx: Context<CreateDispute>,
        job: Pubkey,
        reason: [u8; state::MAX_REASON_LENGTH],
        reason_len: u16,
    ) -> Result<()> {
        instructions::create_dispute::handler(ctx, job, reason, reason_len)
    }

    /// Submit evidence for an ongoing dispute
    pub fn submit_evidence(
        ctx: Context<SubmitEvidence>,
        dispute_id: u64,
        content: [u8; state::MAX_EVIDENCE_LENGTH],
        content_len: u16,
        url: [u8; state::MAX_EVIDENCE_URL_LENGTH],
        url_len: u16,
    ) -> Result<()> {
        instructions::submit_evidence::handler(
            ctx,
            dispute_id,
            content,
            content_len,
            url,
            url_len,
        )
    }

    /// Resolve a dispute (authorized resolver only)
    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        resolution: [u8; state::MAX_RESOLUTION_LENGTH],
        resolution_len: u16,
        outcome: state::ResolutionOutcome,
    ) -> Result<()> {
        instructions::resolve_dispute::handler(ctx, resolution, resolution_len, outcome)
    }
}
