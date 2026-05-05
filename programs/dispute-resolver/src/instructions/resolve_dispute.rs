use anchor_lang::prelude::*;

use crate::errors::DisputeResolverError;
use crate::state::{
    DisputeAccount, DisputeStatus, ResolutionOutcome, DISPUTE_SEED,
    MAX_RESOLUTION_LENGTH,
};

/// The resolver authority — this could be a DAO multisig or an authorized oracle
/// CHECK: Validated against known resolver address
pub const RESOLVER_AUTHORITY: Pubkey = solana_program::pubkey!(
    "4kX9b9hytCTrC6qikjVpnWYrvDK7NG97qCUDUTk9fMmn"
);

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    /// The authorized resolver (DAO multisig or oracle)
    #[account(
        constraint = resolver.key() == RESOLVER_AUTHORITY @ DisputeResolverError::NotResolver,
    )]
    pub resolver: Signer<'info>,

    #[account(
        mut,
        seeds = [DISPUTE_SEED, dispute_account.job.as_ref()],
        bump = dispute_account.bump,
    )]
    pub dispute_account: Account<'info, DisputeAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ResolveDispute>,
    resolution: [u8; MAX_RESOLUTION_LENGTH],
    resolution_len: u16,
    outcome: ResolutionOutcome,
) -> Result<()> {
    require!(
        resolution_len <= MAX_RESOLUTION_LENGTH as u16,
        DisputeResolverError::ResolutionTooLong
    );

    let dispute = &mut ctx.accounts.dispute_account;

    require!(
        dispute.status == DisputeStatus::UnderReview
            || dispute.status == DisputeStatus::Open,
        DisputeResolverError::InvalidDisputeStatus
    );
    require!(!dispute.resolved, DisputeResolverError::AlreadyResolved);

    // Validate split outcome if applicable
    if let ResolutionOutcome::Split { poster_share_bps } = &outcome {
        require!(
            *poster_share_bps <= 10000,
            DisputeResolverError::InvalidSplitPercentage
        );
    }

    let clock = Clock::get()?;

    dispute.resolution = resolution;
    dispute.resolution_len = resolution_len;
    dispute.outcome = Some(outcome.clone());
    dispute.status = DisputeStatus::Resolved;
    dispute.resolved = true;
    dispute.resolved_at = clock.unix_timestamp;

    msg!(
        "Dispute resolved: job={:?}, outcome={:?}",
        dispute.job,
        outcome
    );
    Ok(())
}
