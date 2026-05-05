use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{JobAccount, JobStatus, JOB_SEED};

#[derive(Accounts)]
pub struct DisputeJob<'info> {
    /// Either poster or worker can initiate a dispute
    #[account(mut)]
    pub initiator: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, &job_account.job_id.to_le_bytes()],
        bump = job_account.bump,
        constraint = initiator.key() == job_account.poster
            || initiator.key() == job_account.worker @ JobEscrowError::Unauthorized,
    )]
    pub job_account: Account<'info, JobAccount>,
}

pub fn handler(ctx: Context<DisputeJob>) -> Result<()> {
    let job = &mut ctx.accounts.job_account;
    let clock = Clock::get()?;

    require!(job.status.can_dispute(), JobEscrowError::InvalidJobStatus);

    job.status = JobStatus::Disputed;
    job.updated_at = clock.unix_timestamp;

    msg!(
        "Job #{} disputed by {}",
        job.job_id,
        ctx.accounts.initiator.key()
    );
    Ok(())
}
