use anchor_lang::prelude::*;

/// PDA seed prefixes
pub const RATING_SEED: &[u8] = b"rating";
pub const REPUTATION_NFT_SEED: &[u8] = b"rep_nft";

/// Reputation tier levels
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy, Debug, InitSpace)]
pub enum ReputationTier {
    Scout,   // 0.00 – 1.99 average rating
    Rookie,  // 2.00 – 3.49
    Pro,     // 3.50 – 4.49
    Legend,  // 4.50 – 5.00
}

impl ReputationTier {
    pub fn from_average(avg: u64) -> Self {
        match avg {
            0..=199 => ReputationTier::Scout,
            200..=349 => ReputationTier::Rookie,
            350..=449 => ReputationTier::Pro,
            _ => ReputationTier::Legend,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            ReputationTier::Scout => "Scout",
            ReputationTier::Rookie => "Rookie",
            ReputationTier::Pro => "Pro",
            ReputationTier::Legend => "Legend",
        }
    }

    /// Metaplex Core asset name for the soulbound NFT
    pub fn nft_name(&self) -> String {
        format!("Agent Reputation: {}", self.to_string())
    }
}

/// On-chain account tracking a single rating
#[account]
#[derive(InitSpace)]
pub struct RatingAccount {
    /// The job this rating is for
    pub job_id: u64,
    /// The agent being rated
    pub agent: Pubkey,
    /// The rater (poster who rated the worker, or worker who rated the poster)
    pub rater: Pubkey,
    /// Rating score (100-500, representing 1.00-5.00 with 2 decimal precision)
    pub score: u64,
    /// Optional review text (max 256 bytes)
    pub review: [u8; 256],
    pub review_len: u16,
    /// Timestamp when the rating was created
    pub created_at: i64,
    /// PDA bump
    pub bump: u8,
}

/// Reputation NFT metadata for the soulbound Metaplex Core asset
#[account]
#[derive(InitSpace)]
pub struct ReputationNft {
    /// The agent this NFT belongs to
    pub agent: Pubkey,
    /// Current tier
    pub tier: ReputationTier,
    /// Total number of ratings
    pub total_ratings: u32,
    /// Sum of all rating scores
    pub total_score_sum: u64,
    /// The Metaplex Core asset ID
    pub asset_id: Pubkey,
    /// Timestamp when the NFT was first minted
    pub minted_at: i64,
    /// Timestamp of last tier update
    pub updated_at: i64,
    /// PDA bump
    pub bump: u8,
}
