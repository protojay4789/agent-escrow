use anchor_lang::prelude::*;

/// Maximum length for agent name (bytes)
pub const MAX_NAME_LENGTH: usize = 64;
/// Maximum length for capabilities list (bytes)
pub const MAX_CAPABILITIES_LENGTH: usize = 256;

/// PDA seed prefix for agent accounts
pub const AGENT_SEED: &[u8] = b"agent";

/// Agent registration status
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AgentStatus {
    Active,
    Deactivated,
    PendingVerification,
}

impl AgentStatus {
    pub fn is_active(&self) -> bool {
        *self == AgentStatus::Active
    }
}

/// On-chain account representing a registered AI agent
#[account]
#[derive(InitSpace)]
pub struct AgentAccount {
    /// The wallet that owns and controls this agent
    pub owner: Pubkey,
    /// Human-readable agent name
    pub name: [u8; MAX_NAME_LENGTH],
    /// Length of the name string
    pub name_len: u16,
    /// Capabilities / skills description
    pub capabilities: [u8; MAX_CAPABILITIES_LENGTH],
    /// Length of capabilities string
    pub capabilities_len: u16,
    /// Lamports staked by the agent (bond / security deposit)
    pub stake_lamports: u64,
    /// Total number of jobs completed
    pub jobs_completed: u32,
    /// Cumulative reputation score (sum of all ratings * 100 for precision)
    pub reputation_sum: u64,
    /// Whether World ID human verification has been passed
    pub world_id_verified: bool,
    /// Swig programmable wallet assigned to this agent
    pub swig_wallet: Pubkey,
    /// Metaplex Core asset pubkey holding the AgentIdentity plugin (OOBE Protocol)
    /// Pubkey::default() means no identity is linked
    pub core_asset: Pubkey,
    /// Whether an agent is currently in good standing
    pub is_active: bool,
    /// Timestamp when the agent was registered
    pub created_at: i64,
    /// Timestamp of last update
    pub updated_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl AgentAccount {
    /// Calculate average reputation score (scaled by 100, e.g. 450 = 4.50 stars)
    pub fn average_reputation(&self) -> u64 {
        if self.jobs_completed == 0 {
            0
        } else {
            self.reputation_sum / self.jobs_completed as u64
        }
    }

    /// Determine agent tier based on reputation
    pub fn tier(&self) -> AgentTier {
        let avg = self.average_reputation();
        match avg {
            0..=199 => AgentTier::Scout,
            200..=349 => AgentTier::Rookie,
            350..=449 => AgentTier::Pro,
            _ => AgentTier::Legend,
        }
    }
}

/// Agent reputation tier
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy)]
pub enum AgentTier {
    Scout,    // 0.00 – 1.99
    Rookie,   // 2.00 – 3.49
    Pro,      // 3.50 – 4.49
    Legend,   // 4.50 – 5.00
}
