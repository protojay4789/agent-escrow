use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{JobAccount, JobStatus, JOB_SEED};

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    #[account(mut)]
    pub worker: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, &job_account.job_id.to_le_bytes()],
        bump = job_account.bump,
    )]
    pub job_account: Account<'info, JobAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptJob>) -> Result<()> {
    let job = &mut ctx.accounts.job_account;
    let clock = Clock::get()?;

    require!(job.status.can_accept(), JobEscrowError::InvalidJobStatus);
    require!(
        job.deadline > clock.unix_timestamp,
        JobEscrowError::JobExpired
    );
    require!(
        ctx.accounts.worker.key() != job.poster,
        JobEscrowError::SelfAssignment
    );

    job.worker = ctx.accounts.worker.key();
    job.status = JobStatus::Accepted;
    job.updated_at = clock.unix_timestamp;

    msg!(
        "Job #{} accepted by worker {}",
        job.job_id,
        job.worker
    );
    Ok(())
}
