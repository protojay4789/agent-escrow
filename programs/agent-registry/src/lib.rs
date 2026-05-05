use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6ZS1rNMKrkHLyFS878mU748BbWwbgWMkKFgBJQNKiAG7");

/// World ID verifier program address (Solana deployment)
pub const WORLD_ID_VERIFIER: Pubkey = anchor_lang::solana_program::pubkey!(
    "7EvtswQxbU4aSHAUe2M3iGppRR2jjroQM6DfH29LiVs3"
);

#[program]
pub mod agent_registry {
    use super::*;

    /// Register a new AI agent with a name, capabilities, and stake
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        name: [u8; state::MAX_NAME_LENGTH],
        name_len: u16,
        capabilities: [u8; state::MAX_CAPABILITIES_LENGTH],
        capabilities_len: u16,
        stake_lamports: u64,
    ) -> Result<()> {
        instructions::register_agent::handler(ctx, name, name_len, capabilities, capabilities_len, stake_lamports)
    }

    /// Update agent name and capabilities
    pub fn update_agent(
        ctx: Context<UpdateAgent>,
        name: [u8; state::MAX_NAME_LENGTH],
        name_len: u16,
        capabilities: [u8; state::MAX_CAPABILITIES_LENGTH],
        capabilities_len: u16,
    ) -> Result<()> {
        instructions::update_agent::handler(ctx, name, name_len, capabilities, capabilities_len)
    }

    /// Deactivate an agent (cannot accept new jobs after deactivation)
    pub fn deactivate_agent(ctx: Context<DeactivateAgent>) -> Result<()> {
        instructions::deactivate_agent::handler(ctx)
    }

    /// Verify World ID proof to confirm the agent owner is a unique human
    pub fn verify_world_id(
        ctx: Context<VerifyWorldId>,
        proof: Vec<[u8; 32]>,
        proof_hash: [u8; 32],
        root: [u8; 32],
        signal: [u8; 32],
    ) -> Result<()> {
        instructions::verify_world_id::handler(ctx, proof, proof_hash, root, signal)
    }

    /// Link a Metaplex Core asset as the agent's on-chain identity (OOBE Protocol)
    pub fn link_metaplex_identity(
        ctx: Context<LinkMetaplexIdentity>,
        agent_registration_uri: String,
    ) -> Result<()> {
        instructions::link_metaplex_identity::handler(ctx, agent_registration_uri)
    }
}
