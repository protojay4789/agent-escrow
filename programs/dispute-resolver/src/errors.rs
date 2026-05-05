use anchor_lang::prelude::*;

#[error_code]
pub enum DisputeResolverError {
    #[msg("Dispute reason exceeds maximum length")]
    ReasonTooLong,
    #[msg("Resolution exceeds maximum length")]
    ResolutionTooLong,
    #[msg("Evidence content exceeds maximum length")]
    EvidenceContentTooLong,
    #[msg("Evidence URL exceeds maximum length")]
    EvidenceUrlTooLong,
    #[msg("Maximum number of evidence submissions reached")]
    TooMuchEvidence,
    #[msg("Dispute is not in the correct state")]
    InvalidDisputeStatus,
    #[msg("Unauthorized: you are not a participant in this dispute")]
    NotParticipant,
    #[msg("Unauthorized: only the resolver can resolve disputes")]
    NotResolver,
    #[msg("Dispute has already been resolved")]
    AlreadyResolved,
    #[msg("Dispute has not been resolved yet")]
    NotResolved,
    #[msg("Invalid resolution outcome")]
    InvalidOutcome,
    #[msg("Invalid split percentage (must be 0-10000 basis points)")]
    InvalidSplitPercentage,
    #[msg("Dispute counter overflow")]
    CounterOverflow,
    #[msg("Job not found for this dispute")]
    JobNotFound,
}
