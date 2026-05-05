use anchor_lang::prelude::*;

use crate::errors::ReputationError;
use crate::state::{ReputationNft, ReputationTier, REPUTATION_NFT_SEED};

/// Metaplex Core program address (mainnet/devnet)
pub const METAPLEX_CORE_PROGRAM: Pubkey = solana_program::pubkey!(
    "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
);

/// Default NFT URI pattern (to be updated with actual metadata URI)
pub const DEFAULT_NFT_URI: &str = "https://api.agent-escrow.sol/metadata/{agent}";

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [REPUTATION_NFT_SEED, agent.key().as_ref()],
        bump = reputation_nft.bump,
        constraint = !reputation_nft.asset_id.eq(&Pubkey::default()) @ ReputationError::AlreadyMinted,
    )]
    pub reputation_nft: Account<'info, ReputationNft>,

    /// The agent this NFT belongs to
    /// CHECK: Used for PDA derivation
    pub agent: AccountInfo<'info>,

    /// Metaplex Core collection mint
    /// CHECK: Validated against known collection address
    #[account(
        constraint = collection.key() == METAPLEX_CORE_PROGRAM @ ReputationError::MetaplexError,
    )]
    pub collection: AccountInfo<'info>,

    /// Metaplex Core program
    /// CHECK: Validated against known program address
    #[account(
        constraint = metaplex_program.key() == METAPLEX_CORE_PROGRAM @ ReputationError::MetaplexError,
    )]
    pub metaplex_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<MintNft>) -> Result<()> {
    let rep_nft = &mut ctx.accounts.reputation_nft;
    let clock = Clock::get()?;

    // Ensure reputation has been initialized
    require!(
        rep_nft.total_ratings > 0,
        ReputationError::AgentNotFound
    );

    // Calculate current tier
    let average = rep_nft.total_score_sum / rep_nft.total_ratings as u64;
    let tier = ReputationTier::from_average(average);
    rep_nft.tier = tier;

    // In production, this would call Metaplex Core via CPI to mint a soulbound NFT
    // For now, we store the asset reference and update the tier
    // The actual CPI call would look like:
    //
    // let cpi_accounts =mpl_core::CreateV1 {
    //     asset: ctx.accounts.asset.to_account_info(),
    //     collection: Some(ctx.accounts.collection.to_account_info()),
    //     authority: Some(ctx.accounts.payer.to_account_info()),
    //     payer: ctx.accounts.payer.to_account_info(),
    //     system_program: ctx.accounts.system_program.to_account_info(),
    //     // ... other fields
    // };
    // let cpi_program = ctx.accounts.metaplex_program.to_account_info();
    // mpl_core::cpi::create_v1(cpi_program, cpi_accounts)?;

    // For scaffolding, use the agent's key as a placeholder asset ID
    // In production this would be the actual Metaplex Core asset ID
    rep_nft.asset_id = ctx.accounts.agent.key();
    rep_nft.minted_at = clock.unix_timestamp;
    rep_nft.updated_at = clock.unix_timestamp;

    msg!(
        "Soulbound reputation NFT minted for agent {}: tier={:?}",
        ctx.accounts.agent.key(),
        tier
    );
    Ok(())
}
