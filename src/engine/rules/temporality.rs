use crate::engine::finding::{ContextEffect, Temporality};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn effect(temporality: Temporality) -> ContextEffect {
    ContextEffect::new().with_temporality(temporality)
}

fn options(
    temporality: Temporality,
    max_scope: Option<usize>,
    max_targets: Option<usize>,
) -> ContextOptions {
    let opposite = match temporality {
        Temporality::Current => Temporality::Historical,

        Temporality::Historical => Temporality::Current,
    };

    ContextOptions {
        max_scope,
        max_targets,
        terminated_by: vec![effect(opposite)],
    }
}

fn context_rule(
    pattern: &str,
    direction: Direction,
    temporality: Temporality,
    max_scope: Option<usize>,
    max_targets: Option<usize>,
) -> ContextRule {
    ContextRule::context(
        pattern,
        direction,
        effect(temporality),
        options(temporality, max_scope, max_targets),
    )
}

pub fn rules() -> Vec<ContextRule> {
    vec![
        context_rule(
            r"\bhistory\W+of\b",
            Direction::Forward,
            Temporality::Historical,
            None,
            None,
        ),
        context_rule(
            r"\bprior\b",
            Direction::Forward,
            Temporality::Historical,
            Some(5),
            None,
        ),
        context_rule(
            r"\bprevious\b",
            Direction::Forward,
            Temporality::Historical,
            Some(5),
            None,
        ),
        context_rule(
            r"\bstatus\W+post\b",
            Direction::Forward,
            Temporality::Historical,
            None,
            Some(1),
        ),
        context_rule(
            r"\bs\s*/\s*p\b",
            Direction::Forward,
            Temporality::Historical,
            None,
            Some(1),
        ),
        context_rule(
            r"\bpresent(?:ing|s|ed)\b",
            Direction::Forward,
            Temporality::Current,
            None,
            None,
        ),
        context_rule(
            r"\bcurrently\b",
            Direction::Forward,
            Temporality::Current,
            Some(5),
            None,
        ),
        context_rule(
            r"\brecent(?:ly)?\b",
            Direction::Forward,
            Temporality::Current,
            Some(5),
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn temporality_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }

    #[test]
    fn history_of_is_historical() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bhistory\W+of\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect(Temporality::Historical,),);
    }

    #[test]
    fn presenting_is_current() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bpresent(?:ing|s|ed)\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect(Temporality::Current,),);
    }

    #[test]
    fn status_post_modifies_one_target() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bstatus\W+post\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.options.max_targets, Some(1),);
    }

    #[test]
    fn historical_and_current_effects_terminate_each_other() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bhistory\W+of\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert!(
            behavior
                .options
                .terminated_by
                .contains(&effect(Temporality::Current,),)
        );
    }
}
