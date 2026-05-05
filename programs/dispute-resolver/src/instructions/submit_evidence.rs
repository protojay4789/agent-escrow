use anchor_lang::prelude::*;

use crate::errors::DisputeResolverError;
use crate::state::{
    DisputeAccount, DisputeStatus, EvidenceAccount, DISPUTE_SEED, EVIDENCE_SEED,
    MAX_EVIDENCE_COUNT, MAX_EVIDENCE_LENGTH, MAX_EVIDENCE_URL_LENGTH,
};

#[derive(Accounts)]
#[instruction(
    dispute_id: u64,
    content: [u8; MAX_EVIDENCE_LENGTH],
    content_len: u16,
)]
pub struct SubmitEvidence<'info> {
    #[account(mut)]
    pub submitter: Signer<'info>,

    #[account(
        mut,
        seeds = [DISPUTE_SEED, dispute_account.job.as_ref()],
        bump = dispute_account.bump,
        constraint = dispute_account.status == DisputeStatus::Open
            || dispute_account.status == DisputeStatus::UnderReview @ DisputeResolverError::InvalidDisputeStatus,
    )]
    pub dispute_account: Account<'info, DisputeAccount>,

    #[account(
        init,
        payer = submitter,
        space = 8 + EvidenceAccount::INIT_SPACE,
        seeds = [
            EVIDENCE_SEED,
            dispute_account.job.as_ref(),
            &dispute_account.bump.to_le_bytes(),
            &dispute_account.created_at.to_le_bytes(),
        ],
        bump,
    )]
    pub evidence_account: Account<'info, EvidenceAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<SubmitEvidence>,
    _dispute_id: u64,
    content: [u8; MAX_EVIDENCE_LENGTH],
    content_len: u16,
    url: [u8; MAX_EVIDENCE_URL_LENGTH],
    url_len: u16,
) -> Result<()> {
    require!(
        content_len <= MAX_EVIDENCE_LENGTH as u16,
        DisputeResolverError::EvidenceContentTooLong
    );
    require!(
        url_len <= MAX_EVIDENCE_URL_LENGTH as u16,
        DisputeResolverError::EvidenceUrlTooLong
    );

    let dispute = &mut ctx.accounts.dispute_account;
    let submitter_key = ctx.accounts.submitter.key();

    // Only poster or worker can submit evidence
    require!(
        submitter_key == dispute.poster || submitter_key == dispute.worker,
        DisputeResolverError::NotParticipant
    );
    require!(
        !dispute.resolved,
        DisputeResolverError::AlreadyResolved
    );

    let evidence = &mut ctx.accounts.evidence_account;
    evidence.dispute = dispute.key();
    evidence.submitter = submitter_key;
    evidence.content = content;
    evidence.content_len = content_len;
    evidence.url = url;
    evidence.url_len = url_len;
    evidence.created_at = Clock::get()?.unix_timestamp;
    evidence.bump = ctx.bumps.evidence_account;

    // Move to under review once evidence is submitted
    if dispute.status == DisputeStatus::Open {
        dispute.status = DisputeStatus::UnderReview;
    }

    msg!(
        "Evidence submitted for dispute {:?} by {}",
        dispute.job,
        submitter_key
    );
    Ok(())
}
