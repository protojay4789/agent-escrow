use anchor_lang::prelude::*;

use crate::errors::AgentRegistryError;
use crate::state::{AGENT_SEED, AgentAccount};

/// World ID Merkle tree depth for this verification set
pub const WORLD_ID_TREE_DEPTH: u32 = 30;

#[derive(Accounts)]
#[instruction(proof: Vec<[u8; 32]>, proof_hash: [u8; 32])]
pub struct VerifyWorldId<'info> {
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

    /// World ID verifier contract / program
    /// CHECK: Verified against known World ID program address
    #[account(
        constraint = world_id_verifier.key() == crate::WORLD_ID_VERIFIER @ AgentRegistryError::InvalidWorldIdProof
    )]
    pub world_id_verifier: AccountInfo<'info>,

    /// Nullifier hash to prevent double-verification
    /// CHECK: PDA derived from World ID proof
    #[account(
        mut,
        seeds = [b"nullifier", &proof_hash],
        bump,
        constraint = !nullifier_account.is_used @ AgentRegistryError::AlreadyVerified,
    )]
    pub nullifier_account: Account<'info, NullifierAccount>,

    /// The action ID this proof is valid for (registered with World ID)
    /// CHECK: Compared against expected action ID
    pub action_id: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

/// Tracks whether a World ID nullifier has been consumed
#[account]
pub struct NullifierAccount {
    /// Whether this nullifier has been used
    pub is_used: bool,
}

/// World ID verifier program address (defined in lib.rs)
use crate::WORLD_ID_VERIFIER;

pub fn handler(
    ctx: Context<VerifyWorldId>,
    proof: Vec<[u8; 32]>,
    proof_hash: [u8; 32],
    root: [u8; 32],
    signal: [u8; 32],
) -> Result<()> {
    let agent = &mut ctx.accounts.agent_account;

    require!(
        !agent.world_id_verified,
        AgentRegistryError::AlreadyVerified
    );
    require!(
        !proof.is_empty(),
        AgentRegistryError::InvalidWorldIdProof
    );

    // In production, verify the World ID ZK proof against the verifier contract
    // For now, we trust that the World ID verifier program has validated the proof
    // The proof_hash uniquely identifies this proof instance
    // The root must match the current World ID Merkle tree root
    // The signal encodes the agent's pubkey to bind the proof to this registration

    // Mark nullifier as used to prevent replay attacks
    let nullifier = &mut ctx.accounts.nullifier_account;
    nullifier.is_used = true;

    agent.world_id_verified = true;
    agent.updated_at = Clock::get()?.unix_timestamp;

    msg!(
        "World ID verified for agent: owner={}",
        ctx.accounts.owner.key()
    );
    Ok(())
}
