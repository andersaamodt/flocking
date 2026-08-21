use thiserror::Error;

/// A validation failure at a Flocking trust boundary.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("invalid lowercase 32-byte hexadecimal {kind}")]
    InvalidHex32 { kind: &'static str },
    #[error("topic is empty")]
    EmptyTopic,
    #[error("topic exceeds 64 bytes")]
    TopicTooLong,
    #[error("topic contains characters outside ASCII letters, digits, and underscore")]
    InvalidTopic,
    #[error("invalid addressable Nostr coordinate")]
    InvalidCoordinate,
    #[error("faculty {faculty} does not support scope {scope} and target {target}")]
    InvalidQuestion {
        faculty: String,
        scope: String,
        target: String,
    },
    #[error("action {action} is invalid for faculty {faculty}")]
    InvalidAction { faculty: String, action: String },
    #[error("silence requires a cutoff")]
    MissingSilenceCutoff,
    #[error("only silence may contain a cutoff")]
    UnexpectedSilenceCutoff,
    #[error("silence cutoff exceeds judgment creation time")]
    FutureSilenceCutoff,
    #[error("reason exceeds 500 UTF-8 bytes")]
    ReasonTooLong,
    #[error("judgment event ID is required for published evidence")]
    MissingEventId,
    #[error("configuration version is not supported")]
    UnknownConfigVersion,
    #[error("configuration JSON is invalid: {0}")]
    InvalidConfig(String),
    #[error("configuration contains duplicate source {0}")]
    DuplicateSource(String),
    #[error("source {source_key} contains duplicate grant for {faculty}")]
    DuplicateGrant { source_key: String, faculty: String },
    #[error("rank must be a positive integer")]
    InvalidRank,
    #[error("pin grants must omit rank")]
    PinHasRank,
    #[error("non-pin grants require rank")]
    MissingRank,
    #[error("faculty {faculty} does not support configured scope {scope}")]
    InvalidGrantScope { faculty: String, scope: String },
    #[error("rank {rank} is duplicated for faculty {faculty}")]
    DuplicateRank { faculty: String, rank: u32 },
    #[error("source state does not correspond to an enabled source grant")]
    InvalidSourceState,
    #[error("source state is duplicated for one source, faculty, and scope")]
    DuplicateSourceState,
    #[error("local pin dismissal requires a content target")]
    InvalidPinDismissal,
    #[error("judgment address does not match its semantic tuple")]
    AddressMismatch,
    #[error("integer timestamp is invalid")]
    InvalidTimestamp,
}
