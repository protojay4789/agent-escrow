use anchor_lang::prelude::*;

use crate::errors::AgentRegistryError;
use crate::state::{
    AGENT_SEED, AgentAccount, MAX_CAPABILITIES_LENGTH, MAX_NAME_LENGTH,
};

#[derive(Accounts)]
pub struct UpdateAgent<'info> {
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
}

pub fn handler(
    ctx: Context<UpdateAgent>,
    name: [u8; MAX_NAME_LENGTH],
    name_len: u16,
    capabilities: [u8; MAX_CAPABILITIES_LENGTH],
    capabilities_len: u16,
) -> Result<()> {
    require!(
        name_len <= MAX_NAME_LENGTH as u16,
        AgentRegistryError::NameTooLong
    );
    require!(
        capabilities_len <= MAX_CAPABILITIES_LENGTH as u16,
        AgentRegistryError::CapabilitiesTooLong
    );

    let agent = &mut ctx.accounts.agent_account;
    let clock = Clock::get()?;

    agent.name = name;
    agent.name_len = name_len;
    agent.capabilities = capabilities;
    agent.capabilities_len = capabilities_len;
    agent.updated_at = clock.unix_timestamp;

    msg!("Agent updated: name_len={}, capabilities_len={}", name_len, capabilities_len);
    Ok(())
}
