use regex::{Regex, RegexBuilder};

use super::finding::ContextEffect;
use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
    Bidirectional,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextOptions {
    pub max_scope: Option<usize>,
    pub max_targets: Option<usize>,
    pub terminated_by: Vec<ContextEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBehavior {
    pub direction: Direction,
    pub effect: ContextEffect,
    pub options: ContextOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBehavior {
    Context(ContextBehavior),
    Terminate,
    Pseudo,
}

#[derive(Debug, Clone)]
pub struct ContextRule {
    pub pattern: String,
    pub behavior: RuleBehavior,
}

impl ContextRule {
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

    pub fn terminate(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Terminate,
        }
    }

    pub fn pseudo(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            behavior: RuleBehavior::Pseudo,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMatch {
    pub span: Span,
    pub behavior: RuleBehavior,
}

pub struct RuleSet {
    rules: Vec<CompiledRule>,
}

impl RuleSet {
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

    pub fn find_matches(&self, text: &str) -> Vec<RuleMatch> {
        let mut matches: Vec<RuleMatch> = self
            .rules
            .iter()
            .flat_map(|rule| {
                let behavior = rule.behavior.clone();

                rule.find_iter(text).map(move |span| RuleMatch {
                    span,
                    behavior: behavior.clone(),
                })
            })
            .collect();

        matches.sort_by_key(|rule_match| (rule_match.span.start, rule_match.span.end));

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, ContextEffect, Experiencer, Temporality};

    #[test]
    fn creates_context_rule() {
        let effect = ContextEffect::new().with_assertion(Assertion::Negated);

        let rule = ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        );

        assert_eq!(rule.pattern, r"\bno\b",);

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
    fn context_effect_can_modify_multiple_dimensions() {
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

        assert_eq!(behavior.effect, effect,);
    }

    #[test]
    fn context_options_preserve_scope_configuration() {
        let possible = ContextEffect::new().with_assertion(Assertion::Possible);

        let options = ContextOptions {
            max_scope: Some(5),
            max_targets: Some(1),
            terminated_by: vec![possible],
        };

        let rule = ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            ContextEffect::new().with_assertion(Assertion::Negated),
            options.clone(),
        );

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.options, options,);
    }

    #[test]
    fn creates_terminate_rule() {
        let rule = ContextRule::terminate(r"\bbut\b");

        assert_eq!(rule.behavior, RuleBehavior::Terminate,);
    }

    #[test]
    fn creates_pseudo_rule() {
        let rule = ContextRule::pseudo(r"\bnot\W+only\b");

        assert_eq!(rule.behavior, RuleBehavior::Pseudo,);
    }

    #[test]
    fn invalid_regex_fails_to_compile() {
        let rules = vec![ContextRule::terminate(r"[invalid")];

        assert!(RuleSet::compile(&rules).is_err());
    }

    #[test]
    fn matches_are_case_insensitive() {
        let rules = vec![ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            ContextEffect::new().with_assertion(Assertion::Negated),
            ContextOptions::default(),
        )];

        let rule_set = RuleSet::compile(&rules).unwrap();

        assert_eq!(rule_set.find_matches("NO pneumonia.",).len(), 1,);
    }

    #[test]
    fn match_preserves_behavior() {
        let behavior = ContextBehavior {
            direction: Direction::Backward,
            effect: ContextEffect::new().with_assertion(Assertion::Possible),
            options: ContextOptions {
                max_scope: Some(4),
                max_targets: None,
                terminated_by: Vec::new(),
            },
        };

        let rules = vec![ContextRule {
            pattern: r"\bunlikely\b".to_string(),
            behavior: RuleBehavior::Context(behavior.clone()),
        }];

        let matches = RuleSet::compile(&rules)
            .unwrap()
            .find_matches("Pneumonia is unlikely.");

        assert_eq!(matches[0].behavior, RuleBehavior::Context(behavior,),);
    }

    #[test]
    fn matches_are_sorted_by_text_position() {
        let rules = vec![
            ContextRule::context(
                r"\bno\b",
                Direction::Forward,
                ContextEffect::new().with_assertion(Assertion::Negated),
                ContextOptions::default(),
            ),
            ContextRule::terminate(r"\bbut\b"),
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
}
