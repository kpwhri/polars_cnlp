use super::concept::Concept;
use super::finding::{Assertion, ContextEffect, Experiencer, Finding, FindingContext, Temporality};
use super::modifier::ContextModifier;
use super::rule::{ContextRule, RuleBehavior, RuleSet};
use super::rules;
use super::scope::{TokenRange, token_range};
use super::span::Span;
use super::tokenizer::tokenize;

pub struct Analyzer {
    rules: RuleSet,
}

struct PreparedNote {
    tokens: Vec<Span>,
    modifiers: Vec<ContextModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexedFinding {
    pub concept_index: usize,
    pub finding: Finding,
}

struct TargetOccurrence {
    span: Span,
    token_range: Option<TokenRange>,
    concept_indices: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct ModifierApplication {
    distance: usize,
    modifier_start: usize,
    effect: ContextEffect,
}

#[derive(Debug, Clone, Copy)]
struct RankedValue<T> {
    distance: usize,
    modifier_start: usize,
    value: T,
}

impl Analyzer {
    pub fn compile(rules: &[ContextRule]) -> Result<Self, regex::Error> {
        Ok(Self {
            rules: RuleSet::compile(rules)?,
        })
    }

    pub fn compile_default() -> Result<Self, regex::Error> {
        Self::compile(&rules::default_rules())
    }

    pub fn findings(&self, text: &str, concept: &Concept) -> Vec<Finding> {
        self.findings_all(text, std::slice::from_ref(concept))
            .into_iter()
            .next()
            .unwrap_or_default()
    }

    pub fn findings_all(&self, text: &str, concepts: &[Concept]) -> Vec<Vec<Finding>> {
        if concepts.is_empty() {
            return Vec::new();
        }

        let concept_spans: Vec<Vec<Span>> = concepts
            .iter()
            .map(|concept| concept.find_iter(text).collect())
            .collect();

        if concept_spans.iter().all(Vec::is_empty) {
            return vec![Vec::new(); concepts.len()];
        }

        let prepared = self.prepare(text);

        let targets = collect_targets(&concept_spans, &prepared.tokens);

        let contexts = resolve_targets(&targets, &prepared.modifiers);

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

    pub fn find_best(&self, text: &str, concepts: &[Concept]) -> Option<IndexedFinding> {
        self.collect_indexed_findings(text, concepts)
            .into_iter()
            .min_by_key(finding_rank)
    }

    pub fn affirmed(&self, text: &str, concept: &Concept) -> Option<bool> {
        affirmed_from_findings(&self.findings(text, concept))
    }

    pub fn affirmed_each(&self, text: &str, concepts: &[Concept]) -> Vec<Option<bool>> {
        self.findings_all(text, concepts)
            .iter()
            .map(|findings| affirmed_from_findings(findings))
            .collect()
    }

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

    fn prepare(&self, text: &str) -> PreparedNote {
        let tokens = tokenize(text);

        if tokens.is_empty() {
            return PreparedNote {
                tokens,
                modifiers: Vec::new(),
            };
        }

        let matches = self.rules.find_matches(text);

        let pseudo_spans: Vec<Span> = matches
            .iter()
            .filter_map(|rule_match| match &rule_match.behavior {
                RuleBehavior::Pseudo => Some(rule_match.span),
                _ => None,
            })
            .collect();

        let active_matches = matches
            .into_iter()
            .filter(|rule_match| !matches!(&rule_match.behavior, RuleBehavior::Pseudo))
            .filter(|rule_match| {
                !pseudo_spans
                    .iter()
                    .any(|pseudo| pseudo.overlaps(rule_match.span))
            })
            .collect::<Vec<_>>();

        let mut terminators = Vec::new();

        let mut modifiers = Vec::new();

        for rule_match in active_matches {
            match &rule_match.behavior {
                RuleBehavior::Terminate => {
                    if let Some(range) = token_range(rule_match.span, &tokens) {
                        terminators.push(range);
                    }
                }

                RuleBehavior::Context(_) => {
                    if let Some(modifier) =
                        ContextModifier::from_rule_match(text, &tokens, rule_match)
                    {
                        modifiers.push(modifier);
                    }
                }

                RuleBehavior::Pseudo => {}
            }
        }

        modifiers.sort_by_key(|modifier| (modifier.span().start, modifier.span().end));

        for modifier in &mut modifiers {
            for terminator in &terminators {
                modifier.limit_scope_to_terminator(*terminator);
            }
        }

        let modifier_snapshot = modifiers.clone();

        for (modifier_index, modifier) in modifiers.iter_mut().enumerate() {
            for (other_index, other) in modifier_snapshot.iter().enumerate() {
                if modifier_index == other_index {
                    continue;
                }

                modifier.limit_scope_to_modifier(other);
            }
        }

        PreparedNote { tokens, modifiers }
    }
}

fn collect_targets(concept_spans: &[Vec<Span>], tokens: &[Span]) -> Vec<TargetOccurrence> {
    let mut raw_targets = Vec::new();

    for (concept_index, spans) in concept_spans.iter().enumerate() {
        for span in spans {
            raw_targets.push((*span, concept_index));
        }
    }

    raw_targets.sort_by_key(|(span, concept_index)| (span.start, span.end, *concept_index));

    let mut targets: Vec<TargetOccurrence> = Vec::new();

    for (span, concept_index) in raw_targets {
        if let Some(last) = targets.last_mut()
            && last.span == span
        {
            last.concept_indices.push(concept_index);
            continue;
        }

        targets.push(TargetOccurrence {
            span,
            token_range: token_range(span, tokens),
            concept_indices: vec![concept_index],
        });
    }

    targets
}

fn resolve_targets(
    targets: &[TargetOccurrence],
    modifiers: &[ContextModifier],
) -> Vec<FindingContext> {
    let mut applications: Vec<Vec<ModifierApplication>> = vec![Vec::new(); targets.len()];

    for modifier in modifiers {
        let mut candidates: Vec<(usize, usize)> = targets
            .iter()
            .enumerate()
            .filter_map(|(target_index, target)| {
                let target_range = target.token_range?;

                if !modifier.modifies(target_range) {
                    return None;
                }

                modifier
                    .distance_to(target_range)
                    .map(|distance| (target_index, distance))
            })
            .collect();

        candidates.sort_by_key(|(target_index, distance)| {
            (
                *distance,
                targets[*target_index].span.start,
                targets[*target_index].span.end,
            )
        });

        if let Some(max_targets) = modifier.max_targets() {
            candidates.truncate(max_targets);
        }

        for (target_index, distance) in candidates {
            applications[target_index].push(ModifierApplication {
                distance,
                modifier_start: modifier.span().start,
                effect: modifier.effect(),
            });
        }
    }

    applications
        .iter()
        .map(|applications| resolve_context(applications))
        .collect()
}

fn resolve_context(applications: &[ModifierApplication]) -> FindingContext {
    let mut assertion: Option<RankedValue<Assertion>> = None;

    let mut temporality: Option<RankedValue<Temporality>> = None;

    let mut experiencer: Option<RankedValue<Experiencer>> = None;

    for application in applications {
        if let Some(value) = application.effect.assertion {
            update_ranked(&mut assertion, value, application);
        }

        if let Some(value) = application.effect.temporality {
            update_ranked(&mut temporality, value, application);
        }

        if let Some(value) = application.effect.experiencer {
            update_ranked(&mut experiencer, value, application);
        }
    }

    let effect = ContextEffect {
        assertion: assertion.map(|ranked| ranked.value),
        temporality: temporality.map(|ranked| ranked.value),
        experiencer: experiencer.map(|ranked| ranked.value),
    };

    let mut context = FindingContext::default();

    context.apply(effect);

    context
}

fn update_ranked<T: Copy>(
    slot: &mut Option<RankedValue<T>>,
    value: T,
    application: &ModifierApplication,
) {
    let should_replace = match slot {
        None => true,

        Some(current) => {
            application.distance < current.distance
                || (application.distance == current.distance
                    && application.modifier_start > current.modifier_start)
        }
    };

    if should_replace {
        *slot = Some(RankedValue {
            distance: application.distance,
            modifier_start: application.modifier_start,
            value,
        });
    }
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

    fn analyzer() -> Analyzer {
        Analyzer::compile_default().unwrap()
    }

    fn concept(pattern: &str) -> Concept {
        Concept::new(pattern).unwrap()
    }

    fn concepts() -> Vec<Concept> {
        vec![concept(r"\bpneumonia\b"), concept(r"\banaphylaxis\b")]
    }

    #[test]
    fn affirmed_each_returns_status_for_each_concept() {
        assert_eq!(
            analyzer().affirmed_each("No pneumonia. Anaphylaxis present.", &concepts(),),
            vec![Some(false), Some(true),],
        );
    }

    #[test]
    fn affirmed_any_true_when_any_is_affirmed() {
        assert_eq!(
            analyzer().affirmed_any("No pneumonia. Anaphylaxis present.", &concepts(),),
            Some(true),
        );
    }

    #[test]
    fn affirmed_all_true_when_every_concept_is_affirmed() {
        assert_eq!(
            analyzer().affirmed_all("Pneumonia and anaphylaxis are present.", &concepts(),),
            Some(true),
        );
    }

    #[test]
    fn find_all_is_sorted_by_text_position() {
        let concepts = vec![concept(r"\banaphylaxis\b"), concept(r"\bpneumonia\b")];

        let findings = analyzer().find_all("Pneumonia then anaphylaxis.", &concepts);

        assert_eq!(findings.len(), 2);

        assert_eq!(findings[0].concept_index, 1,);

        assert_eq!(findings[1].concept_index, 0,);

        assert!(findings[0].finding.span.start < findings[1].finding.span.start);
    }

    #[test]
    fn find_all_retains_multiple_occurrences() {
        let findings = analyzer().find_all(
            "No pneumonia. Pneumonia later developed.",
            &[concept(r"\bpneumonia\b")],
        );

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].finding.context.assertion, Assertion::Negated,);
        assert_eq!(findings[1].finding.context.assertion, Assertion::Affirmed,);
    }

    #[test]
    fn find_all_retains_same_span_for_multiple_concepts() {
        let concepts = vec![concept(r"\bpneumonia\b"), concept(r"\bpneumonia\b")];

        let findings = analyzer().find_all("Pneumonia present.", &concepts);

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].finding.span, findings[1].finding.span,);
        assert_eq!(findings[0].concept_index, 0);
        assert_eq!(findings[1].concept_index, 1);
    }

    #[test]
    fn find_best_prefers_affirmed_over_possible_and_negated() {
        let findings = vec![
            concept(r"\bpneumonia\b"),
            concept(r"\banaphylaxis\b"),
            concept(r"\basthma\b"),
        ];

        let best = analyzer()
            .find_best(
                "No pneumonia. Possible anaphylaxis. Asthma present.",
                &findings,
            )
            .unwrap();

        assert_eq!(best.concept_index, 2);
        assert_eq!(best.finding.context.assertion, Assertion::Affirmed,);
    }

    #[test]
    fn find_best_prefers_current_over_historical() {
        let best = analyzer()
            .find_best("History of pneumonia. Possible anaphylaxis.", &concepts())
            .unwrap();

        assert_eq!(best.concept_index, 1);
        assert_eq!(best.finding.context.temporality, Temporality::Current,);
    }

    #[test]
    fn find_best_prefers_patient_over_other_experiencer() {
        let best = analyzer()
            .find_best("Mother has pneumonia. History of anaphylaxis.", &concepts())
            .unwrap();

        assert_eq!(best.concept_index, 1);
        assert_eq!(best.finding.context.experiencer, Experiencer::Patient,);
    }

    #[test]
    fn find_best_uses_earliest_occurrence_for_equal_context() {
        let best = analyzer()
            .find_best("Pneumonia then anaphylaxis.", &concepts())
            .unwrap();

        assert_eq!(best.concept_index, 0);
    }

    #[test]
    fn find_best_uses_concept_order_for_identical_span() {
        let concepts = vec![concept(r"\bpneumonia\b"), concept(r"\bpneumonia\b")];

        let best = analyzer()
            .find_best("Pneumonia present.", &concepts)
            .unwrap();

        assert_eq!(best.concept_index, 0);
    }

    #[test]
    fn find_best_returns_none_when_nothing_matches() {
        assert_eq!(analyzer().find_best("Asthma present.", &concepts(),), None,);
    }

    #[test]
    fn family_history_preserves_independent_dimensions() {
        let findings =
            analyzer().findings("Family history of pneumonia.", &concept(r"\bpneumonia\b"));

        assert_eq!(findings[0].context.temporality, Temporality::Historical,);

        assert_eq!(findings[0].context.experiencer, Experiencer::Other,);
    }

    #[test]
    fn max_targets_is_shared_across_concepts() {
        let concepts = vec![concept(r"\bpneumonia\b"), concept(r"\binfluenza\b")];

        let findings = analyzer().findings_all("Status post pneumonia and influenza.", &concepts);

        assert_eq!(findings[0][0].context.temporality, Temporality::Historical,);

        assert_eq!(findings[1][0].context.temporality, Temporality::Current,);
    }
}
