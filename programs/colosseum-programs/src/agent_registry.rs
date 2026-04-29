use anchor_lang::prelude::*;

use crate::errors::ColosseumError;

#[derive(InitSpace, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AgentTier {
    Scout = 0,
    Rookie = 1,
    Pro = 2,
    Legend = 3,
}

#[derive(InitSpace)]
#[account]
pub struct Agent {
    pub authority: Pubkey,            // The wallet that owns this agent
    pub name: [u8; 32],              // Agent name (fixed size)
    pub name_len: u8,                // Actual length of name
    #[max_len(10)]
    pub capabilities: Vec<[u8; 32]>, // What the agent can do
    pub capability_count: u8,
    pub stake: u64,                  // SOL staked as bond
    pub reputation_score: u32,       // Cumulative score
    pub reputation_count: u16,       // Number of ratings
    pub jobs_completed: u32,
    pub jobs_failed: u16,
    pub active: bool,
    pub tier: u8,                    // 0=Scout, 1=Rookie, 2=Pro, 3=Legend
    pub reputation_nft_mint: Pubkey, // NFT mint address (Pubkey::default() if not minted)
    pub registered_at: i64,
    pub bump: u8,
    // ── World ID / Swig fields ─────────────────────────────────
    pub world_id_hash: [u8; 32],     // Poseidon hash of World ID nullifier
    pub world_id_verified: bool,     // Whether World ID has been verified
    pub swig_wallet: Pubkey,         // Swig wallet for World ID auth flows
}

// ── register_agent ──────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(name: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Agent::INIT_SPACE,
        seeds = [b"agent", authority.key().as_ref()],
        bump,
    )]
    pub agent: Account<'info, Agent>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn register_agent(
    ctx: Context<RegisterAgent>,
    name: String,
    capabilities: Vec<String>,
    stake_amount: u64,
) -> Result<()> {
    require!(name.len() <= 32, ColosseumError::NameTooLong);
    require!(capabilities.len() <= 10, ColosseumError::TooManyCapabilities);
    require!(stake_amount >= 10_000_000, ColosseumError::StakeTooLow); // 0.01 SOL

    let agent = &mut ctx.accounts.agent;
    agent.authority = ctx.accounts.authority.key();

    let name_bytes = name.as_bytes();
    agent.name[..name_bytes.len()].copy_from_slice(name_bytes);
    agent.name_len = name_bytes.len() as u8;

    for cap in capabilities.iter() {
        let cap_bytes = cap.as_bytes();
        let mut buf = [0u8; 32];
        buf[..cap_bytes.len()].copy_from_slice(cap_bytes);
        agent.capabilities.push(buf);
    }
    agent.capability_count = capabilities.len() as u8;

    agent.stake = stake_amount;
    agent.active = true;
    agent.tier = 0; // Scout
    agent.reputation_nft_mint = Pubkey::default();
    agent.registered_at = Clock::get()?.unix_timestamp;
    agent.bump = ctx.bumps.agent;

    // Transfer stake SOL to agent PDA
    agent.world_id_hash = [0u8; 32];
    agent.world_id_verified = false;
    agent.swig_wallet = Pubkey::default(); // System program sentinel
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.authority.key(),
        &ctx.accounts.agent.key(),
        stake_amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.agent.to_account_info(),
        ],
    )?;

    msg!(
        "Agent '{}' registered with stake {} lamports",
        name,
        stake_amount
    );
    Ok(())
}

// ── update_agent ────────────────────────────────────────────────

#[derive(Accounts)]
pub struct UpdateAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", authority.key().as_ref()],
        bump = agent.bump,
        has_one = authority,
    )]
    pub agent: Account<'info, Agent>,
    pub authority: Signer<'info>,
}

pub fn update_agent(
    ctx: Context<UpdateAgent>,
    capabilities: Option<Vec<String>>,
    active: Option<bool>,
) -> Result<()> {
    let agent = &mut ctx.accounts.agent;

    if let Some(caps) = capabilities {
        agent.capabilities.clear();
        for cap in caps.iter() {
            let cap_bytes = cap.as_bytes();
            let mut buf = [0u8; 32];
            buf[..cap_bytes.len()].copy_from_slice(cap_bytes);
            agent.capabilities.push(buf);
        }
        agent.capability_count = agent.capabilities.len() as u8;
    }

    if let Some(is_active) = active {
        agent.active = is_active;
    }

    Ok(())
}

// ── verify_agent (World ID + Swig) ─────────────────────────────

#[derive(Accounts)]
pub struct VerifyAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", authority.key().as_ref()],
        bump = agent.bump,
        has_one = authority,
    )]
    pub agent: Account<'info, Agent>,
    pub authority: Signer<'info>,
}

pub fn verify_agent(
    ctx: Context<VerifyAgent>,
    world_id_hash: [u8; 32],
    swig_wallet: Pubkey,
) -> Result<()> {
    let agent = &mut ctx.accounts.agent;

    // Only the authority (owner) may trigger verification
    require!(
        ctx.accounts.authority.key() == agent.authority,
        ColosseumError::Unauthorized
    );

    // Verify only once
    require!(
        !agent.world_id_verified,
        ColosseumError::AlreadyVerified
    );

    agent.world_id_hash = world_id_hash;
    agent.swig_wallet = swig_wallet;
    agent.world_id_verified = true;

    msg!(
        "Agent '{}' verified — World ID hash set, Swig wallet assigned",
        String::from_utf8_lossy(&agent.name[..agent.name_len as usize])
    );
    Ok(())
}
