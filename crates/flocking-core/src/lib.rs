#![forbid(unsafe_code)]
//! Pure, deterministic Flocking v1 semantics.
//!
//! This crate performs no network, storage, signing, clock, or user-interface
//! work. Protocol translation belongs in `flocking-nostr`.

mod config;
mod error;
mod evaluate;
mod judgment;
mod pin;
mod reverse;
mod types;
mod visibility;

pub use config::{
    Config, FacultyGrant, LocalPinDismissal, PinTargetType, ReverseBlockGrant, Source, SourceState,
};
pub use error::Error;
pub use evaluate::{
    Certainty, Effective, Evaluation, EvaluationInput, Evidence, EvidenceKind, evaluate,
};
pub use judgment::{Judgment, JudgmentEvidence, address, canonical_current, select_current};
pub use pin::{PinInput, PinResult, PinSupport, evaluate_pins};
pub use reverse::{Rescue, ReverseInput, ReverseResult, ReverseTarget, evaluate_reverse, rescue};
pub use types::{
    Action, Completeness, ContentTarget, Context, EventId, Faculty, PublicKey, Scope,
    SourceProvenance, Target, Topic,
};
pub use visibility::{Contribution, Exclusion, Visibility, VisibilityInput, evaluate_visibility};

/// Experimental addressable event kind used by Flocking v1.
pub const JUDGMENT_KIND: u16 = 30_820;

/// Wire-format version tag used by Flocking v1.
pub const PROTOCOL_VERSION: &str = "flocking/1";

/// Portable source-configuration version used by Flocking v1.
pub const CONFIG_VERSION: &str = "flocking-config/1";
