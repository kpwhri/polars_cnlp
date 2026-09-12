use crate::engine::finding::FindingContext;
use crate::engine::rule::ContextRule;
use crate::engine::rules;

use super::{AlgorithmError, ContextAlgorithm, ContextTarget, RuleBasedAlgorithm};

/// Sentence-level implementation of the published ConText algorithm.
///
/// The original section-title extension requires externally identified section
/// boundaries and is therefore intentionally outside this text-only algorithm.
pub struct ConTextAlgorithm {
    inner: RuleBasedAlgorithm,
}

impl ConTextAlgorithm {
    /// Compile ConText using an explicitly supplied rule set.
    pub fn compile(rules: &[ContextRule]) -> Result<Self, AlgorithmError> {
        Ok(Self {
            inner: RuleBasedAlgorithm::compile_context(rules)?,
        })
    }

    /// Compile ConText using the package's default rule set.
    pub fn compile_default() -> Result<Self, AlgorithmError> {
        Self::compile(&rules::context_rules())
    }
}

impl ContextAlgorithm for ConTextAlgorithm {
    fn resolve(&self, text: &str, targets: &[ContextTarget]) -> Vec<FindingContext> {
        self.inner.resolve(text, targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, Experiencer, Temporality};
    use crate::engine::span::Span;

    fn target(text: &str, term: &str) -> ContextTarget {
        let start = text.find(term).unwrap();

        ContextTarget {
            span: Span::new(start, start + term.len()),
            concept_indices: vec![0],
        }
    }

    #[test]
    fn context_has_no_six_token_limit() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "No evidence today whatsoever of severe bilateral pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);
    }

    #[test]
    fn context_resolves_historical_status() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "History of pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].temporality, Temporality::Historical,);
    }

    #[test]
    fn context_resolves_hypothetical_status() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "Return if pneumonia develops.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].temporality, Temporality::Hypothetical,);
    }

    #[test]
    fn context_resolves_other_experiencer() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "Mother has pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].experiencer, Experiencer::Other,);
    }

    #[test]
    fn presenting_terminates_historical_scope() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "History of asthma, presenting with pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].temporality, Temporality::Current,);
    }

    #[test]
    fn status_post_modifies_only_nearest_target() {
        let algorithm = ConTextAlgorithm::compile_default().unwrap();

        let text = "Status post pneumonia and asthma.";

        let targets = [target(text, "pneumonia"), target(text, "asthma")];

        let contexts = algorithm.resolve(text, &targets);

        assert_eq!(contexts[0].temporality, Temporality::Historical,);

        assert_eq!(contexts[1].temporality, Temporality::Current,);
    }
}
