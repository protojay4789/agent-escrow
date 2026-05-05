use anchor_lang::prelude::*;

#[error_code]
pub enum ReputationError {
    #[msg("Rating score must be between 100 and 500 (1.00-5.00)")]
    InvalidRatingScore,
    #[msg("You cannot rate yourself")]
    SelfRating,
    #[msg("Rating for this job already exists")]
    AlreadyRated,
    #[msg("Only the poster or worker of a job can rate")]
    NotJobParticipant,
    #[msg("Review text exceeds maximum length")]
    ReviewTooLong,
    #[msg("Agent not found")]
    AgentNotFound,
    #[msg("NFT already minted for this agent")]
    AlreadyMinted,
    #[msg("Metaplex program error")]
    MetaplexError,
    #[msg("Invalid tier for NFT mint")]
    InvalidTier,
}
