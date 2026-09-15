use crate::engine::algorithm::{
    AlgorithmError, AlgorithmSpec, ConTextAlgorithm, ContextAlgorithm, ContextTarget,
};
use crate::engine::concept::Concept;
use crate::engine::finding::{Assertion, Experiencer, Finding, Temporality};
use crate::engine::rule::ContextRule;
use crate::engine::span::Span;

/// Coordinates target matching and contextual interpretation.
pub struct Analyzer {
    algorithm: Box<dyn ContextAlgorithm>,
}

/// Finding paired with the concept index which produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedFinding {
    pub concept_index: usize,
    pub finding: Finding,
}

impl Analyzer {
    /// Compile a rule set using ConText mechanics.
    ///
    /// This preserves the previous `Analyzer::compile(rules)` API.
    pub fn compile(rules: &[ContextRule]) -> Result<Self, AlgorithmError> {
        Ok(Self::with_algorithm(ConTextAlgorithm::compile(rules)?))
    }

    /// Compile the default ConText analyzer.
    pub fn compile_default() -> Result<Self, AlgorithmError> {
        Self::from_spec(&AlgorithmSpec::default())
    }

    /// Build an analyzer from a serialized algorithm specification.
    pub fn from_spec(spec: &AlgorithmSpec) -> Result<Self, AlgorithmError> {
        Ok(Self {
            algorithm: spec.build()?,
        })
    }

    /// Build an analyzer from any Rust context algorithm.
    pub fn with_algorithm<A>(algorithm: A) -> Self
    where
        A: ContextAlgorithm + 'static,
    {
        Self {
            algorithm: Box::new(algorithm),
        }
    }

    /// Return all findings for one concept.
    pub fn findings(&self, text: &str, concept: &Concept) -> Vec<Finding> {
        self.findings_all(text, std::slice::from_ref(concept))
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    /// Return findings for all concepts using one shared target universe.
    pub fn findings_all(&self, text: &str, concepts: &[Concept]) -> Vec<Vec<Finding>> {
        if concepts.is_empty() {
            return Vec::new();
        }

        let concept_spans = concepts
            .iter()
            .map(|concept| concept.find_iter(text).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        if concept_spans.iter().all(Vec::is_empty) {
            return vec![Vec::new(); concepts.len()];
        }

        let targets = collect_targets(&concept_spans);

        let contexts = self.algorithm.resolve(text, &targets);

        assert_eq!(
            contexts.len(),
            targets.len(),
            "ContextAlgorithm must return exactly one context per target",
        );

        let mut findings = vec![Vec::new(); concepts.len()];

        for (target, context) in targets.iter().zip(contexts) {
            for concept_index in &target.concept_indices {
                findings[*concept_index].push(Finding {
                    span: target.span,
                    context,
                });
            }
        }

        findings
    }

    /// Return all findings in source-text order.
    pub fn find_all(&self, text: &str, concepts: &[Concept]) -> Vec<IndexedFinding> {
        let mut findings = self.collect_indexed_findings(text, concepts);

        findings.sort_by_key(|indexed| {
            (
                indexed.finding.span.start,
                indexed.finding.span.end,
                indexed.concept_index,
            )
        });

        findings
    }

    /// Return the highest-ranked finding.
    pub fn find_best(&self, text: &str, concepts: &[Concept]) -> Option<IndexedFinding> {
        self.collect_indexed_findings(text, concepts)
            .into_iter()
            .min_by_key(finding_rank)
    }

    /// Return whether one concept has an affirmed occurrence.
    pub fn affirmed(&self, text: &str, concept: &Concept) -> Option<bool> {
        affirmed_from_findings(&self.findings(text, concept))
    }

    /// Return affirmation status independently for each concept.
    pub fn affirmed_each(&self, text: &str, concepts: &[Concept]) -> Vec<Option<bool>> {
        self.findings_all(text, concepts)
            .iter()
            .map(|findings| affirmed_from_findings(findings))
            .collect()
    }

    /// Return three-valued OR across requested concepts.
    pub fn affirmed_any(&self, text: &str, concepts: &[Concept]) -> Option<bool> {
        let values = self.affirmed_each(text, concepts);

        if values.is_empty() {
            return None;
        }

        if values.contains(&Some(true)) {
            return Some(true);
        }

        if values.iter().any(Option::is_some) {
            return Some(false);
        }

        None
    }

    /// Return three-valued AND across requested concepts.
    pub fn affirmed_all(&self, text: &str, concepts: &[Concept]) -> Option<bool> {
        let values = self.affirmed_each(text, concepts);

        if values.is_empty() {
            return None;
        }

        if values.contains(&Some(false)) {
            return Some(false);
        }

        if values.iter().all(|value| *value == Some(true)) {
            return Some(true);
        }

        None
    }

    fn collect_indexed_findings(&self, text: &str, concepts: &[Concept]) -> Vec<IndexedFinding> {
        self.findings_all(text, concepts)
            .into_iter()
            .enumerate()
            .flat_map(|(concept_index, findings)| {
                findings.into_iter().map(move |finding| IndexedFinding {
                    concept_index,
                    finding,
                })
            })
            .collect()
    }
}

fn collect_targets(concept_spans: &[Vec<Span>]) -> Vec<ContextTarget> {
    let mut raw_targets = Vec::new();

    for (concept_index, spans) in concept_spans.iter().enumerate() {
        for span in spans {
            raw_targets.push((*span, concept_index));
        }
    }

    raw_targets.sort_by_key(|(span, concept_index)| (span.start, span.end, *concept_index));

    let mut targets: Vec<ContextTarget> = Vec::new();

    for (span, concept_index) in raw_targets {
        if let Some(last) = targets.last_mut()
            && last.span == span
        {
            last.concept_indices.push(concept_index);
            continue;
        }

        targets.push(ContextTarget {
            span,
            concept_indices: vec![concept_index],
        });
    }

    targets
}

fn affirmed_from_findings(findings: &[Finding]) -> Option<bool> {
    if findings.is_empty() {
        return None;
    }

    Some(findings.iter().any(|finding| finding.context.is_affirmed()))
}

fn finding_rank(indexed: &IndexedFinding) -> (u8, u8, u8, usize, usize, usize) {
    let context = indexed.finding.context;

    let experiencer = match context.experiencer {
        Experiencer::Patient => 0,
        Experiencer::Other => 1,
    };

    let temporality = match context.temporality {
        Temporality::Current => 0,
        Temporality::Historical => 1,
        Temporality::Hypothetical => 2,
    };

    let assertion = match context.assertion {
        Assertion::Affirmed => 0,
        Assertion::Possible => 1,
        Assertion::Negated => 2,
    };

    (
        experiencer,
        temporality,
        assertion,
        indexed.finding.span.start,
        indexed.finding.span.end,
        indexed.concept_index,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::algorithm::ContextTarget;
    use crate::engine::finding::FindingContext;

    fn concept(pattern: &str) -> Concept {
        Concept::new(pattern).unwrap()
    }

    #[test]
    fn default_analyzer_uses_context() {
        let analyzer = Analyzer::compile_default().unwrap();

        assert_eq!(
            analyzer.affirmed("History of pneumonia.", &concept(r"\bpneumonia\b",),),
            Some(false),
        );
    }

    #[test]
    fn analyzer_can_use_negex() {
        let analyzer = Analyzer::from_spec(&AlgorithmSpec::Negex {
            rules: None,
            window: 6,
            propagate_same_concept: true,
            additional_rules: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            analyzer.affirmed("History of pneumonia.", &concept(r"\bpneumonia\b",),),
            Some(true),
        );
    }

    struct AlwaysHistorical;

    impl ContextAlgorithm for AlwaysHistorical {
        fn resolve(&self, _text: &str, targets: &[ContextTarget]) -> Vec<FindingContext> {
            targets
                .iter()
                .map(|_| FindingContext {
                    temporality: Temporality::Historical,
                    ..FindingContext::default()
                })
                .collect()
        }
    }

    #[test]
    fn analyzer_accepts_arbitrary_rust_algorithm() {
        let analyzer = Analyzer::with_algorithm(AlwaysHistorical);

        let findings = analyzer.findings("Pneumonia present.", &concept(r"\bpneumonia\b"));

        assert_eq!(findings[0].context.temporality, Temporality::Historical,);

        assert!(!findings[0].context.is_affirmed());
    }
}
