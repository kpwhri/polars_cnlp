use crate::engine::finding::{ContextEffect, Experiencer};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn effect(experiencer: Experiencer) -> ContextEffect {
    ContextEffect::new().with_experiencer(experiencer)
}

fn options(experiencer: Experiencer, max_scope: Option<usize>) -> ContextOptions {
    let opposite = match experiencer {
        Experiencer::Patient => Experiencer::Other,

        Experiencer::Other => Experiencer::Patient,
    };

    ContextOptions {
        max_scope,
        max_targets: None,
        terminated_by: vec![effect(opposite)],
    }
}

fn context_rule(
    pattern: &str,
    direction: Direction,
    experiencer: Experiencer,
    max_scope: Option<usize>,
) -> ContextRule {
    ContextRule::context(
        pattern,
        direction,
        effect(experiencer),
        options(experiencer, max_scope),
    )
}

pub fn rules() -> Vec<ContextRule> {
    vec![
        context_rule(
            r"\bfamily\W+history\W+of\b",
            Direction::Forward,
            Experiencer::Other,
            None,
        ),
        context_rule(
            r"\b(?:mother|father|sister|brother)\b",
            Direction::Forward,
            Experiencer::Other,
            Some(5),
        ),
        context_rule(
            r"\bpatient\b",
            Direction::Forward,
            Experiencer::Patient,
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn experiencer_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }

    #[test]
    fn family_history_sets_other() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bfamily\W+history\W+of\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect(Experiencer::Other,),);
    }

    #[test]
    fn patient_resets_experiencer() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bpatient\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert_eq!(behavior.effect, effect(Experiencer::Patient,),);
    }

    #[test]
    fn patient_terminates_other_scope() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bfamily\W+history\W+of\b")
            .unwrap();

        let RuleBehavior::Context(behavior) = rule.behavior else {
            panic!("expected context rule");
        };

        assert!(
            behavior
                .options
                .terminated_by
                .contains(&effect(Experiencer::Patient,),)
        );
    }
}
