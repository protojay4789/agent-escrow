use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Job {
    pub job_id: [u8; 32],           // Unique job identifier
    pub job_id_len: u8,
    pub poster: Pubkey,             // Who posted the job
    pub worker: Pubkey,             // Who accepted (0 if unassigned)
    pub worker_assigned: bool,
    pub description: [u8; 128],     // Job description
    pub description_len: u8,
    #[max_len(10)]
    pub requirements: Vec<[u8; 32]>, // Required capabilities
    pub requirement_count: u8,
    pub payment: u64,               // SOL in escrow
    pub deadline: i64,              // Unix timestamp
    pub status: u8,                 // 0=open, 1=assigned, 2=submitted, 3=completed, 4=disputed, 5=cancelled, 6=expired
    pub deliverable: [u8; 256],     // Work submission URL/hash
    pub deliverable_len: u16,
    pub created_at: i64,
    pub bump: u8,
}

// ── PostJob ─────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(job_id: String, description: String)]
pub struct PostJob<'info> {
    #[account(
        init,
        payer = poster,
        space = 8 + Job::INIT_SPACE,
        seeds = [b"job", job_id.as_bytes()],
        bump,
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub poster: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn post_job(
    ctx: Context<PostJob>,
    job_id: String,
    description: String,
    requirements: Vec<String>,
    payment: u64,
    deadline: i64,
) -> Result<()> {
    require!(job_id.len() <= 32, crate::ColosseumError::NameTooLong);
    require!(description.len() <= 128, crate::ColosseumError::NameTooLong);
    require!(payment >= 1_000_000, crate::ColosseumError::PaymentTooLow); // 0.001 SOL min

    let job = &mut ctx.accounts.job;

    let id_bytes = job_id.as_bytes();
    job.job_id[..id_bytes.len()].copy_from_slice(id_bytes);
    job.job_id_len = id_bytes.len() as u8;

    job.poster = ctx.accounts.poster.key();

    let desc_bytes = description.as_bytes();
    job.description[..desc_bytes.len()].copy_from_slice(desc_bytes);
    job.description_len = desc_bytes.len() as u8;

    for req in requirements.iter() {
        let req_bytes = req.as_bytes();
        let mut buf = [0u8; 32];
        buf[..req_bytes.len()].copy_from_slice(req_bytes);
        job.requirements.push(buf);
    }
    job.requirement_count = requirements.len() as u8;

    job.payment = payment;
    job.deadline = deadline;
    job.status = 0; // Open
    job.created_at = Clock::get()?.unix_timestamp;
    job.bump = ctx.bumps.job;

    // Transfer payment to job PDA (escrow)
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.poster.key(),
        &ctx.accounts.job.key(),
        payment,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.poster.to_account_info(),
            ctx.accounts.job.to_account_info(),
        ],
    )?;

    msg!("Job '{}' posted with {} lamports payment", job_id, payment);
    Ok(())
}

// ── AcceptJob ───────────────────────────────────────────────────

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
    )]
    pub job: Account<'info, Job>,
    #[account(
        seeds = [b"agent", worker.key().as_ref()],
        bump = agent.bump,
        constraint = agent.active @ crate::ColosseumError::AgentNotActive,
    )]
    pub agent: Account<'info, crate::Agent>,
    #[account(mut)]
    pub worker: Signer<'info>,
}

pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == 0, crate::ColosseumError::JobNotOpen);

    let now = Clock::get()?.unix_timestamp;
    require!(now < job.deadline, crate::ColosseumError::DeadlinePassed);

    job.worker = ctx.accounts.worker.key();
    job.worker_assigned = true;
    job.status = 1; // Assigned

    msg!("Job accepted by agent");
    Ok(())
}

// ── SubmitWork ──────────────────────────────────────────────────

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = job.worker == worker.key() @ crate::ColosseumError::NotAssignedWorker,
    )]
    pub job: Account<'info, Job>,
    pub worker: Signer<'info>,
}

pub fn submit_work(ctx: Context<SubmitWork>, deliverable: String) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == 1, crate::ColosseumError::JobNotOpen);

    let del_bytes = deliverable.as_bytes();
    job.deliverable[..del_bytes.len()].copy_from_slice(del_bytes);
    job.deliverable_len = del_bytes.len() as u16;
    job.status = 2; // Submitted

    msg!("Work submitted for job");
    Ok(())
}

// ── ApproveWork ─────────────────────────────────────────────────

#[derive(Accounts)]
pub struct ApproveWork<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = job.poster == poster.key() @ crate::ColosseumError::NotJobPoster,
        constraint = job.worker == worker.key() @ crate::ColosseumError::NotAssignedWorker,
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub poster: Signer<'info>,
    /// CHECK: Worker wallet to receive payment
    #[account(mut)]
    pub worker: UncheckedAccount<'info>,
}

pub fn approve_work(ctx: Context<ApproveWork>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == 2, crate::ColosseumError::JobNotOpen);

    let payment = job.payment;
    job.status = 3; // Completed

    // Release escrow to worker
    **ctx.accounts.worker.to_account_info().try_borrow_mut_lamports()? += payment;
    **ctx.accounts.job.to_account_info().try_borrow_mut_lamports()? -= payment;

    msg!("Work approved! {} lamports released to worker", payment);
    Ok(())
}

// ── DisputeJob ──────────────────────────────────────────────────

#[derive(Accounts)]
pub struct DisputeJob<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = (job.poster == disputer.key() || job.worker == disputer.key())
            @ crate::ColosseumError::NotDisputeParty,
    )]
    pub job: Account<'info, Job>,
    pub disputer: Signer<'info>,
}

pub fn dispute_job(ctx: Context<DisputeJob>, reason: String) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(
        job.status == 1 || job.status == 2,
        crate::ColosseumError::JobNotOpen
    );

    job.status = 4; // Disputed
    msg!("Job disputed: {}", reason);
    Ok(())
}

// ── CancelJob ───────────────────────────────────────────────────

#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = job.poster == poster.key() @ crate::ColosseumError::NotJobPoster,
    )]
    pub job: Account<'info, Job>,
    #[account(mut)]
    pub poster: Signer<'info>,
}

pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
    let job = &mut ctx.accounts.job;
    require!(job.status == 0, crate::ColosseumError::JobNotOpen);

    let payment = job.payment;
    job.status = 5; // Cancelled

    // Return escrow funds to poster
    **ctx.accounts.poster.to_account_info().try_borrow_mut_lamports()? += payment;
    **ctx.accounts.job.to_account_info().try_borrow_mut_lamports()? -= payment;

    msg!("Job cancelled. {} lamports returned to poster", payment);
    Ok(())
}

// ── ExpireJob ───────────────────────────────────────────────────

#[derive(Accounts)]
pub struct ExpireJob<'info> {
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
    )]
    pub job: Account<'info, Job>,
    /// CHECK: Poster wallet to receive refunded payment
    #[account(mut)]
    pub poster: UncheckedAccount<'info>,
}

pub fn expire_job(ctx: Context<ExpireJob>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    {
        let job = &ctx.accounts.job;
        require!(
            job.status == 0 || job.status == 1,
            crate::ColosseumError::JobNotOpen
        );
        require!(now > job.deadline, crate::ColosseumError::DeadlineNotPassed);
    }

    let job = &mut ctx.accounts.job;
    let payment = job.payment;
    job.status = 6; // Expired

    // Return escrow funds to poster
    **ctx.accounts.poster.to_account_info().try_borrow_mut_lamports()? += payment;
    **ctx.accounts.job.to_account_info().try_borrow_mut_lamports()? -= payment;

    msg!("Job expired. {} lamports returned to poster", payment);
    Ok(())
}
