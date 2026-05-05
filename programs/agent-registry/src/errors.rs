use anchor_lang::prelude::*;

#[error_code]
pub enum AgentRegistryError {
    #[msg("Agent name exceeds maximum length")]
    NameTooLong,
    #[msg("Capabilities description exceeds maximum length")]
    CapabilitiesTooLong,
    #[msg("Agent is already registered")]
    AgentAlreadyRegistered,
    #[msg("Agent not found")]
    AgentNotFound,
    #[msg("Unauthorized: only the agent owner can perform this action")]
    Unauthorized,
    #[msg("Agent is already deactivated")]
    AlreadyDeactivated,
    #[msg("Agent is already active")]
    AlreadyActive,
    #[msg("World ID verification already completed")]
    AlreadyVerified,
    #[msg("World ID proof is invalid")]
    InvalidWorldIdProof,
    #[msg("Stake amount is below minimum threshold")]
    InsufficientStake,
    #[msg("Invalid Swig wallet address")]
    InvalidSwigWallet,
    #[msg("Agent already has a Metaplex identity linked")]
    CoreIdentityAlreadyLinked,
    #[msg("Invalid Metaplex Core asset")]
    InvalidCoreAsset,
    #[msg("No Metaplex identity linked to this agent")]
    IdentityNotLinked,
}
