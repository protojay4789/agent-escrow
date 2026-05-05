use anchor_lang::prelude::*;

#[error_code]
pub enum JobEscrowError {
    #[msg("Job description exceeds maximum length")]
    DescriptionTooLong,
    #[msg("Requirements exceed maximum length")]
    RequirementsTooLong,
    #[msg("Deliverable exceeds maximum length")]
    DeliverableTooLong,
    #[msg("Payment amount must be greater than zero")]
    ZeroPayment,
    #[msg("Deadline must be in the future")]
    InvalidDeadline,
    #[msg("Job is not in the correct state for this action")]
    InvalidJobStatus,
    #[msg("Unauthorized: you are not the poster of this job")]
    NotPoster,
    #[msg("Unauthorized: you are not the worker assigned to this job")]
    NotWorker,
    #[msg("Job has expired, use refund_expired instead")]
    JobExpired,
    #[msg("Escrow vault already claimed")]
    AlreadyClaimed,
    #[msg("Escrow vault not found")]
    VaultNotFound,
    #[msg("Insufficient funds in escrow")]
    InsufficientFunds,
    #[msg("Job is still within deadline, cannot refund")]
    DeadlineNotPassed,
    #[msg("Job is not open, cannot be cancelled")]
    NotOpenForCancellation,
    #[msg("Worker cannot be the same as the poster")]
    SelfAssignment,
    #[msg("Unauthorized: not poster or worker")]
    Unauthorized,
    #[msg("Job counter overflow")]
    CounterOverflow,
}
