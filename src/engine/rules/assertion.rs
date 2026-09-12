use crate::engine::finding::{Assertion, ContextEffect};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn effect(assertion: Assertion) -> ContextEffect {
    ContextEffect::new().with_assertion(assertion)
}

fn options(assertion: Assertion, max_scope: Option<usize>) -> ContextOptions {
    let terminated_by = match assertion {
        Assertion::Affirmed => {
            vec![effect(Assertion::Negated), effect(Assertion::Possible)]
        }

        Assertion::Possible => {
            vec![effect(Assertion::Affirmed), effect(Assertion::Negated)]
        }

        Assertion::Negated => {
            vec![effect(Assertion::Affirmed), effect(Assertion::Possible)]
        }
    };

    ContextOptions {
        max_scope,
        max_targets: None,
        terminated_by,
    }
}

fn context_rule(
    pattern: &str,
    direction: Direction,
    assertion: Assertion,
    max_scope: Option<usize>,
) -> ContextRule {
    ContextRule::context(
        pattern,
        direction,
        effect(assertion),
        options(assertion, max_scope),
    )
}

pub fn rules() -> Vec<ContextRule> {
    vec![
        context_rule(r"\bno\b", Direction::Forward, Assertion::Negated, None),
        context_rule(r"\bnot\b", Direction::Forward, Assertion::Negated, None),
        context_rule(r"\bwithout\b", Direction::Forward, Assertion::Negated, None),
        context_rule(r"\bdenies?\b", Direction::Forward, Assertion::Negated, None),
        context_rule(
            r"\bnegative\W+for\b",
            Direction::Forward,
            Assertion::Negated,
            None,
        ),
        context_rule(
            r"\bruled\W+out\b",
            Direction::Backward,
            Assertion::Negated,
            None,
        ),
        context_rule(
            r"\bnot\W+(?:seen|present|identified)\b",
            Direction::Backward,
            Assertion::Negated,
            Some(5),
        ),
        context_rule(
            r"\bpossible\b",
            Direction::Forward,
            Assertion::Possible,
            None,
        ),
        context_rule(
            r"\bpossibly\b",
            Direction::Forward,
            Assertion::Possible,
            None,
        ),
        context_rule(
            r"\bconcern\W+for\b",
            Direction::Forward,
            Assertion::Possible,
            None,
        ),
        context_rule(
            r"\brule\W+out\b",
            Direction::Forward,
            Assertion::Possible,
            None,
        ),
        context_rule(
            r"\bcannot\W+exclude\b",
            Direction::Forward,
            Assertion::Possible,
            None,
        ),
        context_rule(
            r"\bsuspected\b",
            Direction::Bidirectional,
            Assertion::Possible,
            Some(3),
        ),
        context_rule(
            r"\bpositive\W+for\b",
            Direction::Forward,
            Assertion::Affirmed,
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn assertion_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }

    #[test]
    fn no_is_forward_negation() {
        let rules = rules();

        let rule = rules.iter().find(|rule| rule.pattern == r"\bno\b").unwrap();

        let RuleBehavior::Context(behavior) = &rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.direction, Direction::Forward,);

        assert_eq!(behavior.effect, effect(Assertion::Negated,),);
    }

    #[test]
    fn negation_is_terminated_by_possible() {
        let rules = rules();

        let rule = rules.iter().find(|rule| rule.pattern == r"\bno\b").unwrap();

        let RuleBehavior::Context(behavior) = &rule.behavior else {
            panic!("expected context rule");
        };

        assert!(
            behavior
                .options
                .terminated_by
                .contains(&effect(Assertion::Possible,),)
        );
    }

    #[test]
    fn ruled_out_is_backward() {
        let rules = rules();

        let rule = rules
            .iter()
            .find(|rule| rule.pattern == r"\bruled\W+out\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = &rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.direction, Direction::Backward,);
    }

    #[test]
    fn positive_for_is_explicit_affirmation() {
        let rules = rules();

        let rule = rules
            .iter()
            .find(|rule| rule.pattern == r"\bpositive\W+for\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = &rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect(Assertion::Affirmed,),);
    }

    #[test]
    fn assertion_rules_match_expected_text() {
        let matches = RuleSet::compile(&rules())
            .unwrap()
            .find_matches("No fever. Possible pneumonia. Influenza ruled out.");

        assert_eq!(matches.len(), 3,);
    }
}
