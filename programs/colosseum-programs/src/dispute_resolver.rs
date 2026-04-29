use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct DisputeAccount {
    pub job: Pubkey,               // The job being disputed
    pub initiator: Pubkey,         // Who raised the dispute
    pub reason: [u8; 256],         // Reason for dispute
    pub reason_len: u16,
    pub resolution: [u8; 256],     // Resolution text
    pub resolution_len: u16,
    pub resolved: bool,            // Whether the dispute has been resolved
    pub ruling: u8,                // 0=pending, 1=favor_poster, 2=favor_worker, 3=split
    pub created_at: i64,
    pub bump: u8,
}

// ── RaiseDispute ────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(reason: String)]
pub struct RaiseDispute<'info> {
    #[account(
        init,
        payer = caller,
        space = 8 + DisputeAccount::INIT_SPACE,
        seeds = [b"dispute", job.key().as_ref()],
        bump,
    )]
    pub dispute: Account<'info, DisputeAccount>,
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = job.status == 4 @ crate::ColosseumError::JobNotOpen,
        constraint = (job.poster == caller.key() || job.worker == caller.key())
            @ crate::ColosseumError::NotDisputeParty,
    )]
    pub job: Account<'info, crate::Job>,
    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn raise_dispute(
    ctx: Context<RaiseDispute>,
    reason: String,
) -> Result<()> {
    require!(reason.len() <= 256, crate::ColosseumError::NameTooLong);

    let dispute = &mut ctx.accounts.dispute;
    dispute.job = ctx.accounts.job.key();
    dispute.initiator = ctx.accounts.caller.key();

    let reason_bytes = reason.as_bytes();
    dispute.reason[..reason_bytes.len()].copy_from_slice(reason_bytes);
    dispute.reason_len = reason_bytes.len() as u16;

    dispute.resolved = false;
    dispute.ruling = 0; // pending
    dispute.created_at = Clock::get()?.unix_timestamp;
    dispute.bump = ctx.bumps.dispute;

    msg!("Dispute raised for job");
    Ok(())
}

// ── ResolveDispute ──────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(resolution: String, ruling: u8)]
pub struct ResolveDispute<'info> {
    #[account(
        mut,
        seeds = [b"dispute", dispute.job.as_ref()],
        bump = dispute.bump,
        constraint = !dispute.resolved @ crate::ColosseumError::DisputeAlreadyResolved,
    )]
    pub dispute: Account<'info, DisputeAccount>,
    #[account(
        mut,
        seeds = [b"job", job.job_id[..job.job_id_len as usize].as_ref()],
        bump = job.bump,
        constraint = job.poster == caller.key() @ crate::ColosseumError::NotJobPoster,
        constraint = dispute.job == job.key() @ crate::ColosseumError::JobNotOpen,
    )]
    pub job: Account<'info, crate::Job>,
    /// CHECK: Poster wallet to receive payment if favor_poster or split
    #[account(mut)]
    pub poster: UncheckedAccount<'info>,
    /// CHECK: Worker wallet to receive payment if favor_worker or split
    #[account(mut)]
    pub worker: UncheckedAccount<'info>,
    #[account(mut)]
    pub caller: Signer<'info>,
}

pub fn resolve_dispute(
    ctx: Context<ResolveDispute>,
    resolution: String,
    ruling: u8,
) -> Result<()> {
    require!(resolution.len() <= 256, crate::ColosseumError::NameTooLong);
    require!(
        ruling >= 1 && ruling <= 3,
        crate::ColosseumError::InvalidDisputeRuling
    );

    // Store resolution details
    let dispute = &mut ctx.accounts.dispute;
    let resolution_bytes = resolution.as_bytes();
    dispute.resolution[..resolution_bytes.len()].copy_from_slice(resolution_bytes);
    dispute.resolution_len = resolution_bytes.len() as u16;
    dispute.ruling = ruling;
    dispute.resolved = true;

    // Snapshot values before mutable borrows
    let payment = ctx.accounts.job.payment;
    let job_info = ctx.accounts.job.to_account_info();
    let poster_info = ctx.accounts.poster.to_account_info();
    let worker_info = ctx.accounts.worker.to_account_info();

    // Transfer funds based on ruling and update job status
    match ruling {
        1 => {
            // Favor poster — return entire escrow
            **poster_info.try_borrow_mut_lamports()? += payment;
            **job_info.try_borrow_mut_lamports()? -= payment;
            ctx.accounts.job.status = 5; // Cancelled
            msg!(
                "Dispute resolved: favor poster. {} lamports returned",
                payment
            );
        }
        2 => {
            // Favor worker — release entire escrow to worker
            **worker_info.try_borrow_mut_lamports()? += payment;
            **job_info.try_borrow_mut_lamports()? -= payment;
            ctx.accounts.job.status = 3; // Completed
            msg!(
                "Dispute resolved: favor worker. {} lamports sent to worker",
                payment
            );
        }
        3 => {
            // Split — half to poster, half to worker
            let half = payment / 2;
            let worker_share = payment - half;
            **poster_info.try_borrow_mut_lamports()? += half;
            **worker_info.try_borrow_mut_lamports()? += worker_share;
            **job_info.try_borrow_mut_lamports()? -= payment;
            ctx.accounts.job.status = 5; // Cancelled (partial completion via split)
            msg!(
                "Dispute resolved: split. Poster gets {}, worker gets {}",
                half,
                worker_share
            );
        }
        _ => unreachable!(),
    }

    Ok(())
}
