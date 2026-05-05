use anchor_lang::prelude::*;

use crate::errors::ReputationError;
use crate::state::{ReputationNft, ReputationTier, REPUTATION_NFT_SEED};

#[derive(Accounts)]
pub struct UpdateReputation<'info> {
    #[account(
        mut,
        seeds = [REPUTATION_NFT_SEED, agent.key().as_ref()],
        bump = reputation_nft.bump,
    )]
    pub reputation_nft: Account<'info, ReputationNft>,

    /// The agent whose reputation is being updated
    /// CHECK: We only use the key for PDA derivation
    pub agent: AccountInfo<'info>,
}

pub fn handler(ctx: Context<UpdateReputation>) -> Result<()> {
    let rep_nft = &mut ctx.accounts.reputation_nft;

    require!(
        rep_nft.total_ratings > 0,
        ReputationError::AgentNotFound
    );

    // Recalculate average and determine new tier
    let average = rep_nft.total_score_sum / rep_nft.total_ratings as u64;
    let new_tier = ReputationTier::from_average(average);
    let old_tier = rep_nft.tier;

    rep_nft.tier = new_tier;
    rep_nft.updated_at = Clock::get()?.unix_timestamp;

    if new_tier != old_tier {
        msg!(
            "Reputation tier updated: {:?} -> {:?} for agent {}",
            old_tier,
            new_tier,
            ctx.accounts.agent.key()
        );
    } else {
        msg!(
            "Reputation updated: tier={:?}, avg={}",
            new_tier,
            average
        );
    }

    Ok(())
}
