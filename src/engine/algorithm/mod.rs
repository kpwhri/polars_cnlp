mod context;
mod negex;
mod rule_based;
mod spec;

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::engine::finding::FindingContext;
use crate::engine::span::Span;

pub use context::ConTextAlgorithm;
pub use negex::{DEFAULT_NEGEX_WINDOW, NegExAlgorithm};
pub use rule_based::{RuleBasedAlgorithm, ScopePolicy};
pub use spec::{AlgorithmSpec, RuleSpec};

/// One distinct target occurrence seen by a context algorithm.
///
/// `concept_indices` can contain multiple indices when multiple concept regexes
/// identify the exact same source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTarget {
    pub span: Span,
    pub concept_indices: Vec<usize>,
}

/// Runtime-pluggable contextual interpretation algorithm.
///
/// Implementing this trait is sufficient to integrate a new Rust algorithm with
/// every existing `Analyzer` operation.
pub trait ContextAlgorithm: Send + Sync {
    /// Resolve one context for every target occurrence.
    fn resolve(&self, text: &str, targets: &[ContextTarget]) -> Vec<FindingContext>;
}

/// Error while compiling or configuring a context algorithm.
#[derive(Debug)]
pub enum AlgorithmError {
    Regex(regex::Error),
    InvalidConfig(String),
}

impl Display for AlgorithmError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regex(error) => {
                write!(formatter, "{error}")
            }

            Self::InvalidConfig(message) => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl Error for AlgorithmError {}

impl From<regex::Error> for AlgorithmError {
    fn from(error: regex::Error) -> Self {
        Self::Regex(error)
    }
}
