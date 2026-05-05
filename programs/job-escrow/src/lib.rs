use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("HqpwrZDaoLtNkhsdM8XqX3UPDDozrWLnNF2q6FrD95vH");

#[program]
pub mod job_escrow {
    use super::*;

    /// Post a new job with payment into PDA-locked escrow
    pub fn post_job(
        ctx: Context<PostJob>,
        job_id: u64,
        description: [u8; state::MAX_DESCRIPTION_LENGTH],
        description_len: u16,
        requirements: [u8; state::MAX_REQUIREMENTS_LENGTH],
        requirements_len: u16,
        payment_lamports: u64,
        deadline: i64,
    ) -> Result<()> {
        instructions::post_job::handler(
            ctx,
            job_id,
            description,
            description_len,
            requirements,
            requirements_len,
            payment_lamports,
            deadline,
        )
    }

    /// Worker accepts an open job
    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        instructions::accept_job::handler(ctx)
    }

    /// Worker submits deliverables for an accepted job
    pub fn submit_work(
        ctx: Context<SubmitWork>,
        deliverable: [u8; state::MAX_DELIVERABLE_LENGTH],
        deliverable_len: u16,
    ) -> Result<()> {
        instructions::submit_work::handler(ctx, deliverable, deliverable_len)
    }

    /// Poster approves submitted work and releases funds to worker
    pub fn approve_work(ctx: Context<ApproveWork>) -> Result<()> {
        instructions::approve_work::handler(ctx)
    }

    /// Either party can dispute a submitted or accepted job
    pub fn dispute_job(ctx: Context<DisputeJob>) -> Result<()> {
        instructions::dispute_job::handler(ctx)
    }

    /// Poster cancels an open or accepted job and gets refunded
    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        instructions::cancel_job::handler(ctx)
    }

    /// Refund poster if job has expired (deadline passed)
    pub fn refund_expired(ctx: Context<RefundExpired>) -> Result<()> {
        instructions::refund_expired::handler(ctx)
    }
}
