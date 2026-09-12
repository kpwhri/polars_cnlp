use crate::engine::finding::{Assertion, ContextEffect, Experiencer, FindingContext, Temporality};
use crate::engine::modifier::ContextModifier;
use crate::engine::rule::{
    ContextRule, PseudoBehavior, RuleBehavior, RuleMatch, RuleSet, TerminationBehavior,
};
use crate::engine::scope::{TokenRange, token_range};
use crate::engine::span::Span;
use crate::engine::tokenizer::tokenize;

use super::{AlgorithmError, ContextAlgorithm, ContextTarget};

/// Policy used to determine the initial directional scope of a modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopePolicy {
    /// Preserve each rule's own scope configuration.
    RuleDefined,

    /// Restrict every modifier to the specified number of target positions.
    ///
    /// A window of six corresponds to 0-5 intervening tokens.
    FixedWindow(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModifierBoundaryPolicy {
    SameEffectOrConfigured,
    AnyModifier,
}

#[derive(Debug, Clone, Copy)]
struct RuleBasedConfig {
    scope_policy: ScopePolicy,
    pseudo_terminates_scope: bool,
    modifier_boundary_policy: ModifierBoundaryPolicy,
}

impl RuleBasedConfig {
    fn context() -> Self {
        Self {
            scope_policy: ScopePolicy::RuleDefined,
            pseudo_terminates_scope: false,
            modifier_boundary_policy: ModifierBoundaryPolicy::SameEffectOrConfigured,
        }
    }

    fn negex(window: usize) -> Self {
        Self {
            scope_policy: ScopePolicy::FixedWindow(window),
            pseudo_terminates_scope: true,
            modifier_boundary_policy: ModifierBoundaryPolicy::AnyModifier,
        }
    }

    fn custom(scope_policy: ScopePolicy) -> Self {
        Self {
            scope_policy,
            ..Self::context()
        }
    }
}

/// Generic context-rule execution engine.
///
/// ConText and NegEx share lexical matching and target-resolution machinery but
/// provide different scope mechanics through `RuleBasedConfig`.
pub struct RuleBasedAlgorithm {
    rules: RuleSet,
    config: RuleBasedConfig,
}

struct PreparedNote {
    tokens: Vec<Span>,
    modifiers: Vec<ContextModifier>,
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

impl RuleBasedAlgorithm {
    /// Compile a custom rule-based algorithm.
    ///
    /// Custom algorithms use ConText-style modifier interaction with either
    /// rule-defined or fixed scope.
    pub fn compile(
        rules: &[ContextRule],
        scope_policy: ScopePolicy,
    ) -> Result<Self, AlgorithmError> {
        Self::compile_with_config(rules, RuleBasedConfig::custom(scope_policy))
    }

    /// Compile the mechanics used by ConText.
    pub fn compile_context(rules: &[ContextRule]) -> Result<Self, AlgorithmError> {
        Self::compile_with_config(rules, RuleBasedConfig::context())
    }

    /// Compile the mechanics used by NegEx.
    pub fn compile_negex(rules: &[ContextRule], window: usize) -> Result<Self, AlgorithmError> {
        Self::compile_with_config(rules, RuleBasedConfig::negex(window))
    }

    fn compile_with_config(
        rules: &[ContextRule],
        config: RuleBasedConfig,
    ) -> Result<Self, AlgorithmError> {
        if matches!(config.scope_policy, ScopePolicy::FixedWindow(0)) {
            return Err(AlgorithmError::InvalidConfig(
                "fixed scope window must be at least 1".to_string(),
            ));
        }

        Ok(Self {
            rules: RuleSet::compile(rules)?,
            config,
        })
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

        let pseudo_matches = matches
            .iter()
            .filter_map(|rule_match| {
                let RuleBehavior::Pseudo(behavior) = &rule_match.behavior else {
                    return None;
                };

                Some((rule_match.span, behavior.clone()))
            })
            .collect::<Vec<_>>();

        let mut terminators: Vec<(TokenRange, TerminationBehavior)> = Vec::new();

        let mut pseudo_boundaries: Vec<(TokenRange, PseudoBehavior)> = Vec::new();

        for (span, behavior) in &pseudo_matches {
            if let Some(range) = token_range(*span, &tokens) {
                pseudo_boundaries.push((range, behavior.clone()));
            }
        }

        let mut modifiers = Vec::new();

        for mut rule_match in matches {
            match &rule_match.behavior {
                RuleBehavior::Pseudo(_) => {
                    continue;
                }

                RuleBehavior::Terminate(behavior) => {
                    if let Some(range) = token_range(rule_match.span, &tokens) {
                        terminators.push((range, behavior.clone()));
                    }

                    continue;
                }

                RuleBehavior::Context(behavior) => {
                    if pseudo_matches.iter().any(|(pseudo_span, pseudo_behavior)| {
                        pseudo_span.overlaps(rule_match.span)
                            && pseudo_behavior.applies_to(&behavior.effect)
                    }) {
                        continue;
                    }
                }
            }

            self.apply_scope_policy(&mut rule_match);

            if let Some(modifier) = ContextModifier::from_rule_match(text, &tokens, rule_match) {
                modifiers.push(modifier);
            }
        }

        modifiers.sort_by_key(|modifier| (modifier.span().start, modifier.span().end));

        for modifier in &mut modifiers {
            for (terminator, behavior) in &terminators {
                if behavior.applies_to(&modifier.effect()) {
                    modifier.limit_scope_to_terminator(*terminator);
                }
            }

            if self.config.pseudo_terminates_scope {
                for (pseudo, behavior) in &pseudo_boundaries {
                    if behavior.applies_to(&modifier.effect()) {
                        modifier.limit_scope_to_terminator(*pseudo);
                    }
                }
            }
        }

        let modifier_snapshot = modifiers.clone();

        for (modifier_index, modifier) in modifiers.iter_mut().enumerate() {
            for (other_index, other) in modifier_snapshot.iter().enumerate() {
                if modifier_index == other_index {
                    continue;
                }

                match self.config.modifier_boundary_policy {
                    ModifierBoundaryPolicy::SameEffectOrConfigured => {
                        modifier.limit_scope_to_modifier(other);
                    }

                    ModifierBoundaryPolicy::AnyModifier => {
                        modifier.limit_scope_to_terminator(other.token_span());
                    }
                }
            }
        }

        PreparedNote { tokens, modifiers }
    }

    fn apply_scope_policy(&self, rule_match: &mut RuleMatch) {
        let ScopePolicy::FixedWindow(window) = self.config.scope_policy else {
            return;
        };

        let RuleBehavior::Context(behavior) = &mut rule_match.behavior else {
            return;
        };

        // max_scope counts intervening tokens, while NegEx's published
        // window counts possible target positions.
        let max_intervening = window - 1;

        behavior.options.max_scope = Some(
            behavior
                .options
                .max_scope
                .map_or(max_intervening, |configured| {
                    configured.min(max_intervening)
                }),
        );
    }
}

impl ContextAlgorithm for RuleBasedAlgorithm {
    fn resolve(&self, text: &str, targets: &[ContextTarget]) -> Vec<FindingContext> {
        if targets.is_empty() {
            return Vec::new();
        }

        let prepared = self.prepare(text);

        if prepared.tokens.is_empty() {
            return vec![FindingContext::default(); targets.len()];
        }

        resolve_targets(targets, &prepared.tokens, &prepared.modifiers)
    }
}

fn resolve_targets(
    targets: &[ContextTarget],
    tokens: &[Span],
    modifiers: &[ContextModifier],
) -> Vec<FindingContext> {
    let target_ranges = targets
        .iter()
        .map(|target| token_range(target.span, tokens))
        .collect::<Vec<_>>();

    let mut applications = vec![Vec::new(); targets.len()];

    for modifier in modifiers {
        let mut candidates = target_ranges
            .iter()
            .enumerate()
            .filter_map(|(target_index, target_range)| {
                let target_range = (*target_range)?;

                if !modifier.modifies(target_range) {
                    return None;
                }

                modifier
                    .distance_to(target_range)
                    .map(|distance| (target_index, distance))
            })
            .collect::<Vec<_>>();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::Assertion;
    use crate::engine::rule::{ContextOptions, Direction};

    fn negation_rule() -> ContextRule {
        ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            ContextEffect::new().with_assertion(Assertion::Negated),
            ContextOptions::default(),
        )
    }

    fn target(text: &str, term: &str) -> ContextTarget {
        let start = text.find(term).unwrap();

        ContextTarget {
            span: Span::new(start, start + term.len()),
            concept_indices: vec![0],
        }
    }

    #[test]
    fn rule_defined_scope_reaches_sentence_end() {
        let algorithm = RuleBasedAlgorithm::compile_context(&[negation_rule()]).unwrap();

        let text = "No evidence today whatsoever of severe bilateral pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);
    }

    #[test]
    fn six_position_window_allows_five_intervening_tokens() {
        let algorithm = RuleBasedAlgorithm::compile_negex(&[negation_rule()], 6).unwrap();

        let text = "No one two three four five pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Negated,);
    }

    #[test]
    fn six_position_window_rejects_six_intervening_tokens() {
        let algorithm = RuleBasedAlgorithm::compile_negex(&[negation_rule()], 6).unwrap();

        let text = "No one two three four five six pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Affirmed,);
    }

    #[test]
    fn pseudo_suppresses_overlapping_trigger() {
        let rules = vec![
            negation_rule(),
            ContextRule::pseudo_effects(
                r"\bno\W+increase\b",
                vec![ContextEffect::new().with_assertion(Assertion::Negated)],
            ),
        ];

        let algorithm = RuleBasedAlgorithm::compile_context(&rules).unwrap();

        let text = "No increase in pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Affirmed,);
    }

    #[test]
    fn selective_pseudo_does_not_suppress_unrelated_effect() {
        let historical = ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            ContextEffect::new().with_temporality(Temporality::Historical),
            ContextOptions::default(),
        );

        let rules = vec![
            historical,
            ContextRule::pseudo_effects(
                r"\bno\W+increase\b",
                vec![ContextEffect::new().with_assertion(Assertion::Negated)],
            ),
        ];

        let algorithm = RuleBasedAlgorithm::compile_context(&rules).unwrap();

        let text = "No increase in pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].temporality, Temporality::Historical,);
    }

    #[test]
    fn selective_terminator_stops_only_matching_effect() {
        let rules = vec![
            ContextRule::context(
                r"\bhistory\W+of\b",
                Direction::Forward,
                ContextEffect::new().with_temporality(Temporality::Historical),
                ContextOptions::default(),
            ),
            ContextRule::context(
                r"\bfamily\b",
                Direction::Forward,
                ContextEffect::new().with_experiencer(Experiencer::Other),
                ContextOptions::default(),
            ),
            ContextRule::terminate_effects(
                r"\bpresenting\b",
                vec![ContextEffect::new().with_temporality(Temporality::Historical)],
            ),
        ];

        let algorithm = RuleBasedAlgorithm::compile_context(&rules).unwrap();

        let text = "Family history of asthma presenting with pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].temporality, Temporality::Current,);

        assert_eq!(contexts[0].experiencer, Experiencer::Other,);
    }

    #[test]
    fn negex_pseudo_also_terminates_existing_scope() {
        let rules = vec![
            negation_rule(),
            ContextRule::pseudo_effects(
                r"\bnot\W+only\b",
                vec![ContextEffect::new().with_assertion(Assertion::Negated)],
            ),
        ];

        let algorithm = RuleBasedAlgorithm::compile_negex(&rules, 6).unwrap();

        let text = "No fever not only pneumonia.";

        let contexts = algorithm.resolve(text, &[target(text, "pneumonia")]);

        assert_eq!(contexts[0].assertion, Assertion::Affirmed,);
    }

    #[test]
    fn fixed_window_zero_is_rejected() {
        assert!(RuleBasedAlgorithm::compile_negex(&[negation_rule()], 0,).is_err());
    }
}
