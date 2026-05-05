use anchor_lang::prelude::*;

use crate::errors::AgentRegistryError;
use crate::state::{AGENT_SEED, AgentAccount};

#[derive(Accounts)]
pub struct LinkMetaplexIdentity<'info> {
    #[account(
        mut,
        constraint = agent_account.owner == owner.key() @ AgentRegistryError::Unauthorized,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump = agent_account.bump,
    )]
    pub agent_account: Account<'info, AgentAccount>,

    /// The Metaplex Core asset to link as the agent's identity
    /// CHECK: Stored by reference only; client validates it's a real Metaplex Core asset
    pub core_asset: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<LinkMetaplexIdentity>,
    agent_registration_uri: String,
) -> Result<()> {
    let agent = &mut ctx.accounts.agent_account;

    // Ensure no identity is already linked
    require!(
        agent.core_asset == Pubkey::default(),
        AgentRegistryError::CoreIdentityAlreadyLinked
    );

    // Validate that the provided core_asset is not the zero address
    require!(
        ctx.accounts.core_asset.key() != Pubkey::default(),
        AgentRegistryError::InvalidCoreAsset
    );

    // Store the Metaplex Core asset pubkey
    agent.core_asset = ctx.accounts.core_asset.key();
    agent.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "Metaplex identity linked: owner={}, asset={}",
        ctx.accounts.owner.key(),
        ctx.accounts.core_asset.key()
    );
    msg!("Agent registration URI: {}", agent_registration_uri);

    Ok(())
}
