use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{
    EscrowVault, JobAccount, JobStatus, ESCROW_SEED, JOB_SEED,
};

#[derive(Accounts)]
pub struct RefundExpired<'info> {
    #[account(mut)]
    pub poster: Signer<'info>,

    #[account(
        mut,
        seeds = [JOB_SEED, &job_account.job_id.to_le_bytes()],
        bump = job_account.bump,
        constraint = poster.key() == job_account.poster @ JobEscrowError::NotPoster,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, &job_account.job_id.to_le_bytes()],
        bump = escrow_vault.bump,
        constraint = escrow_vault.job == job_account.key(),
    )]
    pub escrow_vault: Account<'info, EscrowVault>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RefundExpired>) -> Result<()> {
    let clock = Clock::get()?;

    // Allow refund if: job is Open/Accepted/Submitted and deadline has passed,
    // OR if status is explicitly Expired
    let can_refund = match ctx.accounts.job_account.status {
        JobStatus::Expired => true,
        JobStatus::Open | JobStatus::Accepted | JobStatus::Submitted => {
            clock.unix_timestamp > ctx.accounts.job_account.deadline
        }
        _ => false,
    };

    require!(can_refund, JobEscrowError::DeadlineNotPassed);
    require!(!ctx.accounts.escrow_vault.claimed, JobEscrowError::AlreadyClaimed);

    // Refund payment back to poster
    let refund_amount = ctx.accounts.escrow_vault.amount;
    let poster_info = ctx.accounts.poster.to_account_info();
    let vault_info = ctx.accounts.escrow_vault.to_account_info();
    **poster_info.try_borrow_mut_lamports()? += refund_amount;
    **vault_info.try_borrow_mut_lamports()? -= refund_amount;

    ctx.accounts.escrow_vault.claimed = true;
    ctx.accounts.escrow_vault.amount = 0;
    ctx.accounts.job_account.status = JobStatus::Refunded;
    ctx.accounts.job_account.updated_at = clock.unix_timestamp;

    msg!(
        "Job #{} expired refund: {} lamports returned to poster",
        ctx.accounts.job_account.job_id,
        refund_amount
    );
    Ok(())
}
