use std::cmp::{max, min};

use super::finding::ContextEffect;
use super::rule::{ContextBehavior, Direction, RuleBehavior, RuleMatch};
use super::scope::{TokenRange, sentence_token_range, token_range, tokens_between};
use super::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextModifier {
    span: Span,
    token_span: TokenRange,
    scope: TokenRange,
    behavior: ContextBehavior,
}

impl ContextModifier {
    pub fn from_rule_match(text: &str, tokens: &[Span], rule_match: RuleMatch) -> Option<Self> {
        let RuleBehavior::Context(behavior) = rule_match.behavior else {
            return None;
        };

        let token_span = token_range(rule_match.span, tokens)?;

        let sentence = sentence_token_range(text, tokens, token_span)?;

        let scope = initial_scope(token_span, sentence, &behavior);

        Some(Self {
            span: rule_match.span,
            token_span,
            scope,
            behavior,
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn token_span(&self) -> TokenRange {
        self.token_span
    }

    pub fn scope(&self) -> TokenRange {
        self.scope
    }

    pub fn effect(&self) -> ContextEffect {
        self.behavior.effect
    }

    pub fn max_targets(&self) -> Option<usize> {
        self.behavior.options.max_targets
    }

    pub fn modifies(&self, target: TokenRange) -> bool {
        !self.token_span.overlaps(target) && self.scope.overlaps(target)
    }

    pub fn distance_to(&self, target: TokenRange) -> Option<usize> {
        tokens_between(self.token_span, target)
    }

    pub fn limit_scope_to_terminator(&mut self, terminator: TokenRange) -> bool {
        self.limit_scope_to(terminator)
    }

    pub fn limit_scope_to_modifier(&mut self, other: &Self) -> bool {
        let same_effect = self.behavior.effect == other.behavior.effect;

        let explicitly_terminates = self
            .behavior
            .options
            .terminated_by
            .contains(&other.behavior.effect);

        if !same_effect && !explicitly_terminates {
            return false;
        }

        self.limit_scope_to(other.token_span)
    }

    fn limit_scope_to(&mut self, limiter: TokenRange) -> bool {
        if !self.scope.overlaps(limiter) {
            return false;
        }

        let original = self.scope;

        match self.behavior.direction {
            Direction::Forward => {
                if limiter.start >= self.token_span.end {
                    self.scope.end = min(self.scope.end, limiter.start);
                }
            }

            Direction::Backward => {
                if limiter.end <= self.token_span.start {
                    self.scope.start = max(self.scope.start, limiter.end);
                }
            }

            Direction::Bidirectional => {
                if limiter.start >= self.token_span.end {
                    self.scope.end = min(self.scope.end, limiter.start);
                }

                if limiter.end <= self.token_span.start {
                    self.scope.start = max(self.scope.start, limiter.end);
                }
            }
        }

        self.scope != original
    }
}

fn initial_scope(
    modifier: TokenRange,
    sentence: TokenRange,
    behavior: &ContextBehavior,
) -> TokenRange {
    match behavior.direction {
        Direction::Forward => {
            let end = behavior
                .options
                .max_scope
                .map(|max_scope| {
                    let reach = max_scope.saturating_add(1);

                    min(sentence.end, modifier.end.saturating_add(reach))
                })
                .unwrap_or(sentence.end);

            TokenRange::new(modifier.end, end)
        }

        Direction::Backward => {
            let start = behavior
                .options
                .max_scope
                .map(|max_scope| {
                    let reach = max_scope.saturating_add(1);

                    max(sentence.start, modifier.start.saturating_sub(reach))
                })
                .unwrap_or(sentence.start);

            TokenRange::new(start, modifier.start)
        }

        Direction::Bidirectional => {
            let (start, end) = match behavior.options.max_scope {
                Some(max_scope) => {
                    let reach = max_scope.saturating_add(1);

                    (
                        max(sentence.start, modifier.start.saturating_sub(reach)),
                        min(sentence.end, modifier.end.saturating_add(reach)),
                    )
                }

                None => (sentence.start, sentence.end),
            };

            TokenRange::new(start, end)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, ContextEffect};
    use crate::engine::rule::{ContextBehavior, ContextOptions, RuleBehavior};
    use crate::engine::tokenizer::tokenize;

    fn effect(assertion: Assertion) -> ContextEffect {
        ContextEffect::new().with_assertion(assertion)
    }

    fn rule_match(
        span: Span,
        direction: Direction,
        assertion: Assertion,
        options: ContextOptions,
    ) -> RuleMatch {
        RuleMatch {
            span,
            behavior: RuleBehavior::Context(ContextBehavior {
                direction,
                effect: effect(assertion),
                options,
            }),
        }
    }

    #[test]
    fn forward_scope_extends_to_sentence_end() {
        let text = "No fever or pneumonia. Cough.";

        let tokens = tokenize(text);

        let modifier = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        assert_eq!(modifier.scope(), TokenRange::new(1, 4,),);
    }

    #[test]
    fn max_scope_zero_includes_adjacent_target() {
        let text = "No pneumonia.";

        let tokens = tokenize(text);

        let modifier = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions {
                    max_scope: Some(0),
                    ..Default::default()
                },
            ),
        )
        .unwrap();

        assert_eq!(modifier.scope(), TokenRange::new(1, 2,),);

        assert!(modifier.modifies(TokenRange::new(1, 2,),));
    }

    #[test]
    fn max_scope_one_allows_one_intervening_token() {
        let text = "No severe pneumonia.";

        let tokens = tokenize(text);

        let modifier = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions {
                    max_scope: Some(1),
                    ..Default::default()
                },
            ),
        )
        .unwrap();

        let pneumonia = token_range(Span::new(10, 19), &tokens).unwrap();

        assert_eq!(modifier.distance_to(pneumonia,), Some(1),);

        assert!(modifier.modifies(pneumonia,));
    }

    #[test]
    fn max_scope_one_rejects_two_intervening_tokens() {
        let text = "No very severe pneumonia.";

        let tokens = tokenize(text);

        let modifier = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions {
                    max_scope: Some(1),
                    ..Default::default()
                },
            ),
        )
        .unwrap();

        let pneumonia_start = text.find("pneumonia").unwrap();

        let pneumonia = token_range(
            Span::new(pneumonia_start, pneumonia_start + "pneumonia".len()),
            &tokens,
        )
        .unwrap();

        assert_eq!(modifier.distance_to(pneumonia,), Some(2),);

        assert!(!modifier.modifies(pneumonia,));
    }

    #[test]
    fn terminator_limits_forward_scope() {
        let text = "No fever but pneumonia.";

        let tokens = tokenize(text);

        let mut modifier = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        let but_start = text.find("but").unwrap();

        let but = token_range(Span::new(but_start, but_start + 3), &tokens).unwrap();

        assert!(modifier.limit_scope_to_terminator(but,));

        assert_eq!(modifier.scope(), TokenRange::new(1, 2,),);
    }

    #[test]
    fn same_effect_modifier_limits_scope() {
        let text = "No fever and no pneumonia.";

        let tokens = tokenize(text);

        let mut first_no = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        let second_start = text.rfind("no").unwrap();

        let second_no = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(second_start, second_start + 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        assert!(first_no.limit_scope_to_modifier(&second_no,));

        assert_eq!(first_no.scope().end, second_no.token_span().start,);
    }

    #[test]
    fn configured_effect_limits_scope() {
        let text = "No possible pneumonia.";

        let tokens = tokenize(text);

        let possible_effect = effect(Assertion::Possible);

        let mut negation = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions {
                    terminated_by: vec![possible_effect],
                    ..Default::default()
                },
            ),
        )
        .unwrap();

        let possible_start = text.find("possible").unwrap();

        let possible = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(possible_start, possible_start + "possible".len()),
                Direction::Forward,
                Assertion::Possible,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        assert!(negation.limit_scope_to_modifier(&possible,));

        assert_eq!(negation.scope().end, possible.token_span().start,);
    }

    #[test]
    fn unrelated_effect_does_not_limit_scope() {
        let text = "No possible pneumonia.";

        let tokens = tokenize(text);

        let mut negation = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(0, 2),
                Direction::Forward,
                Assertion::Negated,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        let possible_start = text.find("possible").unwrap();

        let possible = ContextModifier::from_rule_match(
            text,
            &tokens,
            rule_match(
                Span::new(possible_start, possible_start + "possible".len()),
                Direction::Forward,
                Assertion::Possible,
                ContextOptions::default(),
            ),
        )
        .unwrap();

        assert!(!negation.limit_scope_to_modifier(&possible,));
    }
}
