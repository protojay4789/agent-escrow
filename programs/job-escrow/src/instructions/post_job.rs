use anchor_lang::prelude::*;

use crate::errors::JobEscrowError;
use crate::state::{
    EscrowVault, JobAccount, JobCounter, JobStatus, ESCROW_SEED, JOB_COUNTER_SEED,
    JOB_SEED, MAX_DELIVERABLE_LENGTH, MAX_DESCRIPTION_LENGTH, MAX_REQUIREMENTS_LENGTH,
};

#[derive(Accounts)]
#[instruction(
    job_id: u64,
    description: [u8; MAX_DESCRIPTION_LENGTH],
    description_len: u16,
    requirements: [u8; MAX_REQUIREMENTS_LENGTH],
    requirements_len: u16,
    payment_lamports: u64,
    deadline: i64,
)]
pub struct PostJob<'info> {
    #[account(mut)]
    pub poster: Signer<'info>,

    #[account(
        init,
        payer = poster,
        space = 8 + JobAccount::INIT_SPACE,
        seeds = [JOB_SEED, &job_id.to_le_bytes()],
        bump,
    )]
    pub job_account: Account<'info, JobAccount>,

    #[account(
        init_if_needed,
        payer = poster,
        space = 8 + JobCounter::INIT_SPACE,
        seeds = [JOB_COUNTER_SEED],
        bump,
    )]
    pub job_counter: Account<'info, JobCounter>,

    #[account(
        init,
        payer = poster,
        space = 8 + EscrowVault::INIT_SPACE,
        seeds = [ESCROW_SEED, &job_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, EscrowVault>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<PostJob>,
    job_id: u64,
    description: [u8; MAX_DESCRIPTION_LENGTH],
    description_len: u16,
    requirements: [u8; MAX_REQUIREMENTS_LENGTH],
    requirements_len: u16,
    payment_lamports: u64,
    deadline: i64,
) -> Result<()> {
    require!(
        description_len <= MAX_DESCRIPTION_LENGTH as u16,
        JobEscrowError::DescriptionTooLong
    );
    require!(
        requirements_len <= MAX_REQUIREMENTS_LENGTH as u16,
        JobEscrowError::RequirementsTooLong
    );
    require!(payment_lamports > 0, JobEscrowError::ZeroPayment);
    require!(
        deadline > Clock::get()?.unix_timestamp,
        JobEscrowError::InvalidDeadline
    );

    let clock = Clock::get()?;

    // Initialize the job account
    let job = &mut ctx.accounts.job_account;
    job.job_id = job_id;
    job.poster = ctx.accounts.poster.key();
    job.worker = Pubkey::default(); // unassigned
    job.description = description;
    job.description_len = description_len;
    job.requirements = requirements;
    job.requirements_len = requirements_len;
    job.payment_lamports = payment_lamports;
    job.deadline = deadline;
    job.status = JobStatus::Open;
    job.deliverable = [0u8; MAX_DELIVERABLE_LENGTH];
    job.deliverable_len = 0;
    job.created_at = clock.unix_timestamp;
    job.updated_at = clock.unix_timestamp;
    job.bump = ctx.bumps.job_account;

    // Initialize escrow vault
    let vault = &mut ctx.accounts.escrow_vault;
    vault.job = ctx.accounts.job_account.key();
    vault.amount = payment_lamports;
    vault.claimed = false;
    vault.bump = ctx.bumps.escrow_vault;

    // Update job counter
    let counter = &mut ctx.accounts.job_counter;
    if counter.count == 0 {
        counter.count = 1;
        counter.bump = ctx.bumps.job_counter;
    }
    // Ensure the provided job_id matches the counter
    require!(
        job_id == counter.count,
        JobEscrowError::InvalidJobStatus
    );
    counter.count = counter.count.checked_add(1).ok_or(JobEscrowError::CounterOverflow)?;

    // Transfer payment from poster to the escrow vault PDA
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.poster.key(),
        &ctx.accounts.escrow_vault.key(),
        payment_lamports,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.poster.to_account_info(),
            ctx.accounts.escrow_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    msg!(
        "Job #{} posted with {} lamports payment, deadline: {}",
        job_id,
        payment_lamports,
        deadline
    );
    Ok(())
}
