use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("9CDr7iJrrKvzdxFMmK156EFEHKdXR2jiDDCVqLSX79rb");

#[program]
pub mod reputation {
    use super::*;

    /// Rate an agent after completing a job (score 100-500 = 1.00-5.00)
    pub fn rate_agent(
        ctx: Context<RateAgent>,
        job_id: u64,
        agent: Pubkey,
        score: u64,
        review: [u8; 256],
        review_len: u16,
    ) -> Result<()> {
        instructions::rate_agent::handler(ctx, job_id, agent, score, review, review_len)
    }

    /// Recalculate and update an agent's reputation tier
    pub fn update_reputation(ctx: Context<UpdateReputation>) -> Result<()> {
        instructions::update_reputation::handler(ctx)
    }

    /// Mint a soulbound Metaplex Core reputation NFT for an agent
    pub fn mint_reputation_nft(ctx: Context<MintNft>) -> Result<()> {
        instructions::mint_nft::handler(ctx)
    }
}
