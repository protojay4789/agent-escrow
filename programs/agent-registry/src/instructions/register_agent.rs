use anchor_lang::prelude::*;

use crate::errors::AgentRegistryError;
use crate::state::{AGENT_SEED, AgentAccount, MAX_CAPABILITIES_LENGTH, MAX_NAME_LENGTH};

/// Minimum stake required to register an agent (0.1 SOL)
pub const MIN_STAKE_LAMPORTS: u64 = 100_000_000;

#[derive(Accounts)]
#[instruction(name: [u8; MAX_NAME_LENGTH], name_len: u16)]
pub struct RegisterAgent<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + AgentAccount::INIT_SPACE,
        seeds = [AGENT_SEED, owner.key().as_ref()],
        bump,
    )]
    pub agent_account: Account<'info, AgentAccount>,

    /// The Swig programmable wallet to assign to this agent
    /// CHECK: Validated as a system account or PDA
    #[account(mut)]
    pub swig_wallet: AccountInfo<'info>,

    /// World ID verification nullifier (0 if not verified yet)
    /// CHECK: Validated in verify_world_id instruction
    pub world_id_nullifier: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterAgent>,
    name: [u8; MAX_NAME_LENGTH],
    name_len: u16,
    capabilities: [u8; MAX_CAPABILITIES_LENGTH],
    capabilities_len: u16,
    stake_lamports: u64,
) -> Result<()> {
    require!(
        name_len <= MAX_NAME_LENGTH as u16,
        AgentRegistryError::NameTooLong
    );
    require!(
        capabilities_len <= MAX_CAPABILITIES_LENGTH as u16,
        AgentRegistryError::CapabilitiesTooLong
    );
    require!(
        stake_lamports >= MIN_STAKE_LAMPORTS,
        AgentRegistryError::InsufficientStake
    );

    let agent = &mut ctx.accounts.agent_account;
    let clock = Clock::get()?;

    agent.owner = ctx.accounts.owner.key();
    agent.name = name;
    agent.name_len = name_len;
    agent.capabilities = capabilities;
    agent.capabilities_len = capabilities_len;
    agent.stake_lamports = stake_lamports;
    agent.jobs_completed = 0;
    agent.reputation_sum = 0;
    agent.world_id_verified = false;
    agent.swig_wallet = ctx.accounts.swig_wallet.key();
    agent.is_active = true;
    agent.created_at = clock.unix_timestamp;
    agent.updated_at = clock.unix_timestamp;
    agent.bump = ctx.bumps.agent_account;

    // Transfer stake lamports from owner to agent PDA (locked as bond)
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.owner.key(),
        &ctx.accounts.agent_account.key(),
        stake_lamports,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.agent_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    msg!("Agent registered: {} with stake {} lamports", name_len, stake_lamports);
    Ok(())
}
