use regex::{Regex, RegexBuilder};
use serde::Deserialize;

use super::finding::ContextEffect;
use super::span::Span;

/// Direction in which a context modifier applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Forward,
    Backward,
    Bidirectional,
}

/// Optional controls over a context modifier's scope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextOptions {
    /// Maximum number of intervening tokens.
    pub max_scope: Option<usize>,

    /// Maximum number of targets affected by one modifier occurrence.
    pub max_targets: Option<usize>,

    /// Other context effects which explicitly terminate this modifier.
    pub terminated_by: Vec<ContextEffect>,
}

/// Behavior of a context-producing rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBehavior {
    pub direction: Direction,
    pub effect: ContextEffect,
    pub options: ContextOptions,
}

/// Behavior of a scope-termination rule.
///
/// An empty `effects` vector means that the rule terminates every context effect.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminationBehavior {
    pub effects: Vec<ContextEffect>,
}

impl TerminationBehavior {
    /// Return whether this termination rule applies to an effect.
    pub fn applies_to(&self, effect: &ContextEffect) -> bool {
        self.effects.is_empty()
            || self
                .effects
                .iter()
                .any(|candidate| candidate.intersects(effect))
    }
}

/// Behavior of a pseudo-trigger rule.
///
/// An empty `effects` vector means that the pseudo suppresses every overlapping
/// context modifier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PseudoBehavior {
    pub effects: Vec<ContextEffect>,
}

impl PseudoBehavior {
    /// Return whether this pseudo rule suppresses an effect.
    pub fn applies_to(&self, effect: &ContextEffect) -> bool {
        self.effects.is_empty()
            || self
                .effects
                .iter()
                .any(|candidate| candidate.intersects(effect))
    }
}

/// Semantic behavior attached to a lexical rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBehavior {
    Context(ContextBehavior),
    Terminate(TerminationBehavior),
    Pseudo(PseudoBehavior),
}

/// Declarative lexical rule used by a context algorithm.
#[derive(Debug, Clone)]
pub struct ContextRule {
    pub pattern: String,
    pub behavior: RuleBehavior,
}

impl ContextRule {
    /// Create a context-producing rule.
    pub fn context(
        pattern: impl Into<String>,
        direction: Direction,
        effect: ContextEffect,
        options: ContextOptions,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Context(ContextBehavior {
                direction,
                effect,
                options,
            }),
        }
    }

    /// Create a termination rule which stops every context effect.
    pub fn terminate(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Terminate(TerminationBehavior::default()),
        }
    }

    /// Create a termination rule applying only to selected effects.
    pub fn terminate_effects(pattern: impl Into<String>, effects: Vec<ContextEffect>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Terminate(TerminationBehavior { effects }),
        }
    }

    /// Create a pseudo rule which suppresses every overlapping context rule.
    pub fn pseudo(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Pseudo(PseudoBehavior::default()),
        }
    }

    /// Create a pseudo rule applying only to selected context effects.
    pub fn pseudo_effects(pattern: impl Into<String>, effects: Vec<ContextEffect>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Pseudo(PseudoBehavior { effects }),
        }
    }
}

struct CompiledRule {
    regex: Regex,
    behavior: RuleBehavior,
}

impl CompiledRule {
    fn find_iter<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Span> + 'a {
        self.regex
            .find_iter(text)
            .map(|finding| Span::new(finding.start(), finding.end()))
    }
}

/// Raw occurrence of a lexical rule in text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    pub span: Span,
    pub behavior: RuleBehavior,
}

/// Compiled collection of lexical context rules.
pub struct RuleSet {
    rules: Vec<CompiledRule>,
}

impl RuleSet {
    /// Compile a set of rules using case-insensitive regex matching.
    pub fn compile(rules: &[ContextRule]) -> Result<Self, regex::Error> {
        let rules = rules
            .iter()
            .map(|rule| {
                let regex = RegexBuilder::new(&rule.pattern)
                    .case_insensitive(true)
                    .build()?;

                Ok(CompiledRule {
                    regex,
                    behavior: rule.behavior.clone(),
                })
            })
            .collect::<Result<Vec<_>, regex::Error>>()?;

        Ok(Self { rules })
    }

    /// Return all raw rule matches sorted by source-text position.
    ///
    /// Pseudo suppression and scope interpretation deliberately happen later in
    /// the algorithm layer.
    pub fn find_matches(&self, text: &str) -> Vec<RuleMatch> {
        let mut matches = self
            .rules
            .iter()
            .flat_map(|rule| {
                let behavior = rule.behavior.clone();

                rule.find_iter(text).map(move |span| RuleMatch {
                    span,
                    behavior: behavior.clone(),
                })
            })
            .collect::<Vec<_>>();

        matches.sort_by_key(|rule_match| (rule_match.span.start, rule_match.span.end));

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, Experiencer, Temporality};

    fn negated_effect() -> ContextEffect {
        ContextEffect::new().with_assertion(Assertion::Negated)
    }

    #[test]
    fn creates_context_rule() {
        let effect = negated_effect();

        let rule = ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        );

        assert_eq!(rule.pattern, r"\bno\b");

        assert_eq!(
            rule.behavior,
            RuleBehavior::Context(ContextBehavior {
                direction: Direction::Forward,
                effect,
                options: ContextOptions::default(),
            },),
        );
    }

    #[test]
    fn creates_global_terminate_rule() {
        let rule = ContextRule::terminate(r"\bbut\b");

        assert_eq!(
            rule.behavior,
            RuleBehavior::Terminate(TerminationBehavior::default(),),
        );
    }

    #[test]
    fn creates_selective_terminate_rule() {
        let historical = ContextEffect::new().with_temporality(Temporality::Historical);

        let rule = ContextRule::terminate_effects(r"\bpresenting\b", vec![historical]);

        assert_eq!(
            rule.behavior,
            RuleBehavior::Terminate(TerminationBehavior {
                effects: vec![historical],
            },),
        );
    }

    #[test]
    fn creates_global_pseudo_rule() {
        let rule = ContextRule::pseudo(r"\bnot\W+only\b");

        assert_eq!(
            rule.behavior,
            RuleBehavior::Pseudo(PseudoBehavior::default(),),
        );
    }

    #[test]
    fn creates_selective_pseudo_rule() {
        let historical = ContextEffect::new().with_temporality(Temporality::Historical);

        let rule = ContextRule::pseudo_effects(r"\bhistory\W+exam\b", vec![historical]);

        assert_eq!(
            rule.behavior,
            RuleBehavior::Pseudo(PseudoBehavior {
                effects: vec![historical],
            },),
        );
    }

    #[test]
    fn context_rule_preserves_scope_configuration() {
        let possible = ContextEffect::new().with_assertion(Assertion::Possible);

        let options = ContextOptions {
            max_scope: Some(5),
            max_targets: Some(1),
            terminated_by: vec![possible],
        };

        let rule = ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            negated_effect(),
            options.clone(),
        );

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.options, options);
    }

    #[test]
    fn context_rule_preserves_direction() {
        let rule = ContextRule::context(
            r"\bruled\W+out\b",
            Direction::Backward,
            negated_effect(),
            ContextOptions::default(),
        );

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.direction, Direction::Backward,);
    }

    #[test]
    fn context_rule_preserves_multi_dimensional_effect() {
        let effect = ContextEffect::new()
            .with_temporality(Temporality::Historical)
            .with_experiencer(Experiencer::Other);

        let rule = ContextRule::context(
            r"\bfamily\W+history\W+of\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        );

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect);
    }

    #[test]
    fn global_termination_applies_to_any_effect() {
        let behavior = TerminationBehavior::default();

        assert!(behavior.applies_to(&negated_effect(),));

        assert!(
            behavior.applies_to(&ContextEffect::new().with_temporality(Temporality::Historical,),)
        );
    }

    #[test]
    fn selective_termination_applies_only_to_matching_effect() {
        let historical = ContextEffect::new().with_temporality(Temporality::Historical);

        let behavior = TerminationBehavior {
            effects: vec![historical],
        };

        assert!(behavior.applies_to(&historical));
        assert!(!behavior.applies_to(&negated_effect()));
    }

    #[test]
    fn invalid_regex_fails_to_compile() {
        let rules = vec![ContextRule::terminate(r"[invalid")];

        assert!(RuleSet::compile(&rules).is_err());
    }

    #[test]
    fn matches_case_insensitively() {
        let rules = vec![ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            negated_effect(),
            ContextOptions::default(),
        )];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("NO pneumonia.");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn returns_exact_byte_span() {
        let rules = vec![ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            negated_effect(),
            ContextOptions::default(),
        )];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("Patient: no pneumonia.");

        assert_eq!(matches[0].span, Span::new(9, 11),);
    }

    #[test]
    fn match_preserves_context_behavior() {
        let behavior = ContextBehavior {
            direction: Direction::Backward,
            effect: ContextEffect::new().with_assertion(Assertion::Possible),
            options: ContextOptions {
                max_scope: Some(4),
                max_targets: Some(1),
                terminated_by: vec![negated_effect()],
            },
        };

        let rules = vec![ContextRule {
            pattern: r"\bunlikely\b".to_string(),
            behavior: RuleBehavior::Context(behavior.clone()),
        }];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("Pneumonia is unlikely.");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].behavior, RuleBehavior::Context(behavior),);
    }

    #[test]
    fn returns_all_occurrences_of_rule() {
        let rules = vec![ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            negated_effect(),
            ContextOptions::default(),
        )];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("No fever and no cough.");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].span, Span::new(0, 2),);
        assert_eq!(matches[1].span, Span::new(13, 15),);
    }

    #[test]
    fn matches_are_sorted_by_text_position() {
        let rules = vec![
            ContextRule::terminate(r"\bbut\b"),
            ContextRule::context(
                r"\bno\b",
                Direction::Forward,
                negated_effect(),
                ContextOptions::default(),
            ),
        ];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("No fever but no cough.");

        assert!(
            matches
                .windows(2)
                .all(|pair| { pair[0].span.start <= pair[1].span.start },)
        );
    }

    #[test]
    fn raw_matches_include_context_overlapping_pseudo() {
        let rules = vec![
            ContextRule::context(
                r"\bnot\b",
                Direction::Forward,
                negated_effect(),
                ContextOptions::default(),
            ),
            ContextRule::pseudo(r"\bnot\W+only\b"),
        ];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("Not only pneumonia.");

        assert_eq!(matches.len(), 2);

        assert!(
            matches
                .iter()
                .any(|rule_match| matches!(rule_match.behavior, RuleBehavior::Context(_)),)
        );

        assert!(
            matches
                .iter()
                .any(|rule_match| matches!(rule_match.behavior, RuleBehavior::Pseudo(_)),)
        );
    }

    #[test]
    fn raw_matches_include_terminators_and_context_rules() {
        let rules = vec![
            ContextRule::context(
                r"\bno\b",
                Direction::Forward,
                negated_effect(),
                ContextOptions::default(),
            ),
            ContextRule::terminate(r"\bbut\b"),
        ];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("No fever but pneumonia.");

        assert_eq!(matches.len(), 2);

        assert!(
            matches
                .iter()
                .any(|rule_match| matches!(rule_match.behavior, RuleBehavior::Context(_)),)
        );

        assert!(
            matches
                .iter()
                .any(|rule_match| matches!(rule_match.behavior, RuleBehavior::Terminate(_)),)
        );
    }
}
