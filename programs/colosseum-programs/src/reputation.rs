use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct ReputationRecord {
    pub agent: Pubkey,
    pub rater: Pubkey,
    pub job_ref: [u8; 32],
    pub job_ref_len: u8,
    pub score: u8,          // 1-5
    pub review: [u8; 128],
    pub review_len: u8,
    pub timestamp: i64,
    pub bump: u8,
}

/// Simulated Metaplex Core NFT representing an agent's reputation tier.
/// For hackathon purposes, this replaces the full Metaplex dependency.
#[derive(InitSpace)]
#[account]
pub struct ReputationNft {
    pub agent: Pubkey,       // The agent this NFT belongs to
    pub mint: Pubkey,        // The NFT mint address
    pub tier_at_mint: u8,    // Tier when NFT was minted (0=Scout,1=Rookie,2=Pro,3=Legend)
    pub jobs_at_mint: u32,   // Jobs completed when NFT was minted
    pub created_at: i64,     // When the NFT was created
    pub bump: u8,
}

/// Recalculate the agent tier based on jobs completed and average rating.
///
/// Tier thresholds:
///   0 (Scout):  default — no requirements
///   1 (Rookie): jobs_completed >= 3  AND avg_rating >= 3.0
///   2 (Pro):    jobs_completed >= 10 AND avg_rating >= 4.0
///   3 (Legend): jobs_completed >= 25 AND avg_rating >= 4.5
///
/// Uses integer math to avoid floating-point determinism concerns:
///   avg >= 3.0 ⟺ score >= count * 3
///   avg >= 4.0 ⟺ score >= count * 4
///   avg >= 4.5 ⟺ score * 2 >= count * 9
pub fn recalculate_tier(jobs_completed: u32, reputation_score: u32, reputation_count: u16) -> u8 {
    if reputation_count == 0 {
        return 0; // Scout — no ratings yet
    }

    // Legend: jobs_completed >= 25 AND avg_rating >= 4.5
    if jobs_completed >= 25
        && (reputation_score as u64 * 2 >= reputation_count as u64 * 9)
    {
        return 3;
    }

    // Pro: jobs_completed >= 10 AND avg_rating >= 4.0
    if jobs_completed >= 10
        && (reputation_score as u64 >= reputation_count as u64 * 4)
    {
        return 2;
    }

    // Rookie: jobs_completed >= 3 AND avg_rating >= 3.0
    if jobs_completed >= 3
        && (reputation_score as u64 >= reputation_count as u64 * 3)
    {
        return 1;
    }

    0 // Scout
}

#[derive(Accounts)]
#[instruction(agent_pubkey: Pubkey, job_ref: String, score: u8, review: String)]
pub struct RateAgent<'info> {
    #[account(
        init,
        payer = rater,
        space = 8 + ReputationRecord::INIT_SPACE,
        seeds = [b"rep", agent_pubkey.as_ref(), job_ref.as_bytes()],
        bump,
    )]
    pub record: Account<'info, ReputationRecord>,
    #[account(
        mut,
        seeds = [b"agent", agent_pubkey.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, crate::Agent>,
    #[account(mut)]
    pub rater: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn rate_agent(
    ctx: Context<RateAgent>,
    agent_pubkey: Pubkey,
    job_ref: String,
    score: u8,
    review: String,
) -> Result<()> {
    require!(score >= 1 && score <= 5, crate::ColosseumError::InvalidScore);
    require!(
        ctx.accounts.rater.key() != ctx.accounts.agent.authority,
        crate::ColosseumError::SelfRating
    );
    require!(review.len() <= 128, crate::ColosseumError::NameTooLong);

    let record = &mut ctx.accounts.record;
    record.agent = ctx.accounts.agent.key();
    record.rater = ctx.accounts.rater.key();

    let job_bytes = ctx.accounts.agent.key().to_bytes();
    record.job_ref[..job_bytes.len().min(32)]
        .copy_from_slice(&job_bytes[..job_bytes.len().min(32)]);

    record.score = score;
    let rev_bytes = review.as_bytes();
    record.review[..rev_bytes.len()].copy_from_slice(rev_bytes);
    record.review_len = rev_bytes.len() as u8;
    record.timestamp = Clock::get()?.unix_timestamp;
    record.bump = ctx.bumps.record;

    // Update agent cumulative score
    let agent = &mut ctx.accounts.agent;
    agent.reputation_score += score as u32;
    agent.reputation_count += 1;

    // Recalculate tier based on updated stats
    let new_tier = recalculate_tier(
        agent.jobs_completed,
        agent.reputation_score,
        agent.reputation_count,
    );
    agent.tier = new_tier;

    let tier_name = match new_tier {
        1 => "Rookie",
        2 => "Pro",
        3 => "Legend",
        _ => "Scout",
    };

    msg!(
        "Agent rated {}/5 — total score: {} ({} ratings) | Tier updated: {} ({})",
        score,
        agent.reputation_score,
        agent.reputation_count,
        new_tier,
        tier_name,
    );
    Ok(())
}

#[derive(Accounts)]
pub struct GetReputation<'info> {
    #[account(
        seeds = [b"agent", agent.authority.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, crate::Agent>,
}

pub fn get_reputation(ctx: Context<GetReputation>) -> Result<()> {
    let agent = &ctx.accounts.agent;
    let avg = if agent.reputation_count > 0 {
        agent.reputation_score as f64 / agent.reputation_count as f64
    } else {
        0.0
    };

    let tier_name = match agent.tier {
        1 => "Rookie",
        2 => "Pro",
        3 => "Legend",
        _ => "Scout",
    };

    msg!(
        "Agent '{}': {:.1}/5.0 ({} ratings, {} jobs completed) | Tier: {} ({})",
        String::from_utf8_lossy(&agent.name[..agent.name_len as usize]),
        avg,
        agent.reputation_count,
        agent.jobs_completed,
        agent.tier,
        tier_name,
    );
    Ok(())
}

// ── Mint Reputation NFT ────────────────────────────────────────

#[derive(Accounts)]
pub struct MintReputationNft<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + ReputationNft::INIT_SPACE,
        seeds = [b"rep_nft", agent.authority.as_ref()],
        bump,
    )]
    pub rep_nft: Account<'info, ReputationNft>,
    #[account(
        mut,
        seeds = [b"agent", agent.authority.as_ref()],
        bump = agent.bump,
        constraint = agent.jobs_completed >= 1
            @ crate::ColosseumError::AgentNotEligibleForNft,
        constraint = agent.reputation_nft_mint == Pubkey::default()
            @ crate::ColosseumError::NftAlreadyMinted,
    )]
    pub agent: Account<'info, crate::Agent>,
    /// CHECK: The NFT mint keypair — used as the mint address for the simulated NFT.
    pub mint: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn mint_reputation_nft(ctx: Context<MintReputationNft>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;

    // Read agent fields before mutable borrow
    let agent_key = ctx.accounts.agent.key();
    let agent_tier = ctx.accounts.agent.tier;
    let agent_jobs = ctx.accounts.agent.jobs_completed;
    let mint_key = ctx.accounts.mint.key();

    // Populate the ReputationNft account
    let rep_nft = &mut ctx.accounts.rep_nft;
    rep_nft.agent = agent_key;
    rep_nft.mint = mint_key;
    rep_nft.tier_at_mint = agent_tier;
    rep_nft.jobs_at_mint = agent_jobs;
    rep_nft.created_at = now;
    rep_nft.bump = ctx.bumps.rep_nft;

    // Point the agent record to this NFT mint
    let agent_mut = &mut ctx.accounts.agent;
    agent_mut.reputation_nft_mint = mint_key;

    let tier_name = match agent_tier {
        1 => "Rookie",
        2 => "Pro",
        3 => "Legend",
        _ => "Scout",
    };

    msg!(
        "Reputation NFT minted — agent tier: {} ({}), mint: {}, jobs_at_mint: {}",
        agent_tier,
        tier_name,
        mint_key,
        agent_jobs,
    );
    Ok(())
}
