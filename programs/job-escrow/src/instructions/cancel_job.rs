use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{
    EscrowVault, JobAccount, JobStatus, ESCROW_SEED, JOB_SEED,
};

#[derive(Accounts)]
pub struct CancelJob<'info> {
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

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelJob>) -> Result<()> {
    let clock = Clock::get()?;

    require!(ctx.accounts.job_account.status.can_cancel(), JobEscrowError::NotOpenForCancellation);
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
        "Job #{} cancelled, {} lamports refunded to poster",
        ctx.accounts.job_account.job_id,
        refund_amount
    );
    Ok(())
}
