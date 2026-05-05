use anchor_lang::prelude::*;

use crate::errors::AgentRegistryError;
use crate::state::{AGENT_SEED, AgentAccount};

#[derive(Accounts)]
pub struct DeactivateAgent<'info> {
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

pub fn handler(ctx: Context<DeactivateAgent>) -> Result<()> {
    let agent = &mut ctx.accounts.agent_account;

    require!(
        agent.is_active,
        AgentRegistryError::AlreadyDeactivated
    );

    agent.is_active = false;
    agent.updated_at = Clock::get()?.unix_timestamp;

    msg!("Agent deactivated: owner={}", ctx.accounts.owner.key());
    Ok(())
}
