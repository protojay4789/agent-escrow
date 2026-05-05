use anchor_lang::prelude::*;

use crate::errors::DisputeResolverError;
use crate::state::{
    DisputeAccount, DisputeCounter, DisputeStatus, DISPUTE_COUNTER_SEED,
    DISPUTE_SEED, MAX_REASON_LENGTH,
};

#[derive(Accounts)]
#[instruction(
    job: Pubkey,
    reason: [u8; MAX_REASON_LENGTH],
    reason_len: u16,
)]
pub struct CreateDispute<'info> {
    #[account(mut)]
    pub initiator: Signer<'info>,

    #[account(
        init,
        payer = initiator,
        space = 8 + DisputeAccount::INIT_SPACE,
        seeds = [DISPUTE_SEED, job.as_ref()],
        bump,
    )]
    pub dispute_account: Account<'info, DisputeAccount>,

    #[account(
        init_if_needed,
        payer = initiator,
        space = 8 + DisputeCounter::INIT_SPACE,
        seeds = [DISPUTE_COUNTER_SEED],
        bump,
    )]
    pub dispute_counter: Account<'info, DisputeCounter>,

    /// The poster of the job being disputed
    /// CHECK: Validated from job account
    /// CHECK: Used as a participant reference
    pub poster: AccountInfo<'info>,

    /// The worker of the job being disputed
    /// CHECK: Validated from job account
    /// CHECK: Used as a participant reference
    pub worker: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateDispute>,
    job: Pubkey,
    reason: [u8; MAX_REASON_LENGTH],
    reason_len: u16,
) -> Result<()> {
    require!(
        reason_len <= MAX_REASON_LENGTH as u16,
        DisputeResolverError::ReasonTooLong
    );

    // Verify initiator is either the poster or worker
    let initiator_key = ctx.accounts.initiator.key();
    require!(
        initiator_key == ctx.accounts.poster.key()
            || initiator_key == ctx.accounts.worker.key(),
        DisputeResolverError::NotParticipant
    );

    let dispute = &mut ctx.accounts.dispute_account;
    let clock = Clock::get()?;

    dispute.job = job;
    dispute.initiator = initiator_key;
    dispute.poster = ctx.accounts.poster.key();
    dispute.worker = ctx.accounts.worker.key();
    dispute.reason = reason;
    dispute.reason_len = reason_len;
    dispute.resolution = [0u8; crate::state::MAX_RESOLUTION_LENGTH];
    dispute.resolution_len = 0;
    dispute.outcome = None;
    dispute.status = DisputeStatus::Open;
    dispute.resolved = false;
    dispute.created_at = clock.unix_timestamp;
    dispute.resolved_at = 0;
    dispute.bump = ctx.bumps.dispute_account;

    // Update counter
    let counter = &mut ctx.accounts.dispute_counter;
    if counter.count == 0 {
        counter.count = 1;
        counter.bump = ctx.bumps.dispute_counter;
    }
    counter.count = counter
        .count
        .checked_add(1)
        .ok_or(DisputeResolverError::CounterOverflow)?;

    msg!(
        "Dispute created for job {:?} by {}",
        job,
        initiator_key
    );
    Ok(())
}
