use anchor_lang::prelude::*;

use crate::errors::ReputationError;
use crate::state::{RatingAccount, ReputationNft, RATING_SEED, REPUTATION_NFT_SEED};

/// Minimum rating score (1.00 = 100)
pub const MIN_RATING: u64 = 100;
/// Maximum rating score (5.00 = 500)
pub const MAX_RATING: u64 = 500;

#[derive(Accounts)]
#[instruction(job_id: u64, agent: Pubkey)]
pub struct RateAgent<'info> {
    #[account(mut)]
    pub rater: Signer<'info>,

    #[account(
        init,
        payer = rater,
        space = 8 + RatingAccount::INIT_SPACE,
        seeds = [RATING_SEED, &job_id.to_le_bytes(), agent.as_ref()],
        bump,
    )]
    pub rating_account: Account<'info, RatingAccount>,

    #[account(
        mut,
        seeds = [REPUTATION_NFT_SEED, agent.as_ref()],
        bump = reputation_nft.bump,
    )]
    pub reputation_nft: Account<'info, ReputationNft>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RateAgent>,
    job_id: u64,
    agent: Pubkey,
    score: u64,
    review: [u8; 256],
    review_len: u16,
) -> Result<()> {
    require!(
        score >= MIN_RATING && score <= MAX_RATING,
        ReputationError::InvalidRatingScore
    );
    require!(
        review_len <= 256,
        ReputationError::ReviewTooLong
    );

    let rating = &mut ctx.accounts.rating_account;
    let rep_nft = &mut ctx.accounts.reputation_nft;
    let clock = Clock::get()?;

    // Initialize rating
    rating.job_id = job_id;
    rating.agent = agent;
    rating.rater = ctx.accounts.rater.key();
    rating.score = score;
    rating.review = review;
    rating.review_len = review_len;
    rating.created_at = clock.unix_timestamp;
    rating.bump = ctx.bumps.rating_account;

    // Update reputation totals
    rep_nft.total_ratings += 1;
    rep_nft.total_score_sum += score;
    rep_nft.updated_at = clock.unix_timestamp;

    msg!(
        "Rating recorded: agent={}, score={}, total_ratings={}",
        agent,
        score,
        rep_nft.total_ratings
    );
    Ok(())
}
