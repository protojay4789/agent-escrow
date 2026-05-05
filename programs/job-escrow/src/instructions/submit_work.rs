use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{
    JobAccount, JobStatus, JOB_SEED, MAX_DELIVERABLE_LENGTH,
};

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(
        mut,
        constraint = worker.key() == job_account.worker @ JobEscrowError::NotWorker,
    )]
    pub worker: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, &job_account.job_id.to_le_bytes()],
        bump = job_account.bump,
    )]
    pub job_account: Account<'info, JobAccount>,
}

pub fn handler(
    ctx: Context<SubmitWork>,
    deliverable: [u8; MAX_DELIVERABLE_LENGTH],
    deliverable_len: u16,
) -> Result<()> {
    require!(
        deliverable_len <= MAX_DELIVERABLE_LENGTH as u16,
        JobEscrowError::DeliverableTooLong
    );

    let job = &mut ctx.accounts.job_account;
    let clock = Clock::get()?;

    require!(job.status.can_submit(), JobEscrowError::InvalidJobStatus);
    require!(
        job.deadline > clock.unix_timestamp,
        JobEscrowError::JobExpired
    );

    job.deliverable = deliverable;
    job.deliverable_len = deliverable_len;
    job.status = JobStatus::Submitted;
    job.updated_at = clock.unix_timestamp;

    msg!(
        "Job #{} work submitted by worker {}",
        job.job_id,
        ctx.accounts.worker.key()
    );
    Ok(())
}
