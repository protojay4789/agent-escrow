use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{EscrowVault, JobAccount, JobStatus, ESCROW_SEED, JOB_SEED};

#[derive(Accounts)]
pub struct ApproveWork<'info> {
    #[account(
        mut,
        constraint = poster.key() == job_account.poster @ JobEscrowError::NotPoster,
    )]
    pub poster: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, &job_account.job_id.to_le_bytes()],
        bump = job_account.bump,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, &job_account.job_id.to_le_bytes()],
        bump = escrow_vault.bump,
        constraint = escrow_vault.job == job_account.key(),
    )]
    pub escrow_vault: Account<'info, EscrowVault>,

    /// The worker receiving payment
    /// CHECK: Validated against job_account.worker
    #[account(
        mut,
        constraint = worker.key() == job_account.worker @ JobEscrowError::NotWorker,
    )]
    /// CHECK: We just need the pubkey for the transfer
    pub worker: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApproveWork>) -> Result<()> {
    let clock = Clock::get()?;

    require!(ctx.accounts.job_account.status.can_approve(), JobEscrowError::InvalidJobStatus);
    require!(!ctx.accounts.escrow_vault.claimed, JobEscrowError::AlreadyClaimed);

    // Transfer funds from escrow vault to worker
    let payment = ctx.accounts.escrow_vault.amount;
    let vault_info = ctx.accounts.escrow_vault.to_account_info();
    let worker_info = ctx.accounts.worker.to_account_info();

    **vault_info.try_borrow_mut_lamports()? -= payment;
    **worker_info.try_borrow_mut_lamports()? += payment;

    ctx.accounts.escrow_vault.claimed = true;
    ctx.accounts.escrow_vault.amount = 0;
    ctx.accounts.job_account.status = JobStatus::Completed;
    ctx.accounts.job_account.updated_at = clock.unix_timestamp;

    msg!(
        "Job #{} approved, {} lamports released to worker {}",
        ctx.accounts.job_account.job_id,
        payment,
        ctx.accounts.worker.key()
    );
    Ok(())
}
