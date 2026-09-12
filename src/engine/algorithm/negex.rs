use std::collections::HashMap;

use crate::engine::finding::{Assertion, FindingContext};
use crate::engine::rule::ContextRule;
use crate::engine::rules;

use super::{AlgorithmError, ContextAlgorithm, ContextTarget, RuleBasedAlgorithm};

/// Published NegEx scope is six target positions, equivalent to 0-5
/// intervening terms.
pub const DEFAULT_NEGEX_WINDOW: usize = 6;

/// Implementation of NegEx scope mechanics.
pub struct NegExAlgorithm {
    inner: RuleBasedAlgorithm,
    propagate_same_concept: bool,
}

impl NegExAlgorithm {
    /// Compile NegEx using an explicitly supplied rule set.
    pub fn compile(
        rules: &[ContextRule],
        window: usize,
        propagate_same_concept: bool,
    ) -> Result<Self, AlgorithmError> {
        Ok(Self {
            inner: RuleBasedAlgorithm::compile_negex(rules, window)?,
            propagate_same_concept,
        })
    }

    /// Compile NegEx using the package's default trigger set.
    pub fn compile_default() -> Result<Self, AlgorithmError> {
        Self::compile(&rules::negex_rules(), DEFAULT_NEGEX_WINDOW, true)
    }
}

impl ContextAlgorithm for NegExAlgorithm {
    fn resolve(&self, text: &str, targets: &[ContextTarget]) -> Vec<FindingContext> {
        let mut contexts = self.inner.resolve(text, targets);

        if self.propagate_same_concept {
            propagate_assertion(targets, &mut contexts);
        }

        contexts
    }
}

fn propagate_assertion(targets: &[ContextTarget], contexts: &mut [FindingContext]) {
    let mut by_concept: HashMap<usize, Assertion> = HashMap::new();

    for (target, context) in targets.iter().zip(contexts.iter()) {
        for concept_index in &target.concept_indices {
            by_concept
                .entry(*concept_index)
                .and_modify(|existing| {
                    if assertion_rank(context.assertion) < assertion_rank(*existing) {
                        *existing = context.assertion;
                    }
                })
                .or_insert(context.assertion);
        }
    }

    for (target, context) in targets.iter().zip(contexts.iter_mut()) {
        if let Some(assertion) = target
            .concept_indices
            .iter()
            .filter_map(|concept_index| by_concept.get(concept_index).copied())
            .min_by_key(|assertion| assertion_rank(*assertion))
        {
            context.assertion = assertion;
        }
    }
}

fn assertion_rank(assertion: Assertion) -> u8 {
    match assertion {
        Assertion::Negated => 0,
        Assertion::Possible => 1,
        Assertion::Affirmed => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::span::Span;

    fn target(text: &str, term: &str, concept_index: usize) -> ContextTarget {
        let start = text.find(term).unwrap();

        ContextTarget {
            span: Span::new(start, start + term.len()),
            concept_indices: vec![concept_index],
        }
    }

    #[test]
    fn negex_negates_inside_window() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "No evidence of severe pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia", 0)]);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);
    }

    #[test]
    fn negex_does_not_negate_outside_window() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "No one two three four five six pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia", 0)]);

        assert_eq!(contexts[0].assertion, Assertion::Affirmed,);
    }

    #[test]
    fn negex_supports_post_triggers() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "Pneumonia was ruled out.";

        let contexts = algorithm.resolve(text, &[target(text, "Pneumonia", 0)]);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);
    }

    #[test]
    fn negex_supports_possible_triggers() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "Rule out pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia", 0)]);

        assert_eq!(contexts[0].assertion, Assertion::Possible,);
    }

    #[test]
    fn negex_does_not_assign_historical_context() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "History of pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia", 0)]);

        assert_eq!(
            contexts[0].temporality,
            crate::engine::finding::Temporality::Current,
        );
    }

    #[test]
    fn negex_propagates_negation_across_same_concept() {
        let algorithm = NegExAlgorithm::compile_default().unwrap();

        let text = "No pneumonia, pneumonia discussed.";

        let first_start = text.find("pneumonia").unwrap();

        let second_start = text.rfind("pneumonia").unwrap();

        let targets = [
            ContextTarget {
                span: Span::new(first_start, first_start + "pneumonia".len()),
                concept_indices: vec![0],
            },
            ContextTarget {
                span: Span::new(second_start, second_start + "pneumonia".len()),
                concept_indices: vec![0],
            },
        ];

        let contexts = algorithm.resolve(text, &targets);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);

        assert_eq!(contexts[1].assertion, Assertion::Negated,);
    }

    #[test]
    fn propagation_can_be_disabled() {
        let algorithm =
            NegExAlgorithm::compile(&rules::negex_rules(), DEFAULT_NEGEX_WINDOW, false).unwrap();

        let text = "No pneumonia, but pneumonia discussed.";

        let first_start = text.find("pneumonia").unwrap();

        let second_start = text.rfind("pneumonia").unwrap();

        let targets = [
            ContextTarget {
                span: Span::new(first_start, first_start + "pneumonia".len()),
                concept_indices: vec![0],
            },
            ContextTarget {
                span: Span::new(second_start, second_start + "pneumonia".len()),
                concept_indices: vec![0],
            },
        ];

        let contexts = algorithm.resolve(text, &targets);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);

        assert_eq!(contexts[1].assertion, Assertion::Affirmed,);
    }
}
