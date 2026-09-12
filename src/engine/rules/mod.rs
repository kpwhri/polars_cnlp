pub mod assertion;
pub mod control;
pub mod experiencer;
pub mod temporality;

use crate::engine::rule::ContextRule;

/// Return the package's default sentence-level ConText rule set.
pub fn context_rules() -> Vec<ContextRule> {
    let mut rules = assertion::negation_rules();

    rules.extend(control::assertion_rules());

    rules.extend(assertion::context_possible_rules());

    rules.extend(temporality::rules());

    rules.extend(experiencer::rules());

    rules.extend(control::context_rules());

    rules
}

/// Return the package's default NegEx trigger set.
///
/// This includes the later PREP/POSP "possible" categories in addition to
/// negation, pseudo, and conjunction rules.
pub fn negex_rules() -> Vec<ContextRule> {
    let mut rules = assertion::negation_rules();

    rules.extend(assertion::negex_possible_rules());

    rules.extend(control::assertion_rules());

    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, Experiencer, Temporality};
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn context_rules_compile() {
        assert!(RuleSet::compile(&context_rules(),).is_ok());
    }

    #[test]
    fn negex_rules_compile() {
        assert!(RuleSet::compile(&negex_rules(),).is_ok());
    }

    #[test]
    fn context_rules_include_hypothetical_temporality() {
        assert!(context_rules().iter().any(|rule| {
            let RuleBehavior::Context(behavior) = &rule.behavior else {
                return false;
            };

            behavior.effect.temporality == Some(Temporality::Hypothetical)
        },));
    }

    #[test]
    fn context_rules_include_other_experiencer() {
        assert!(context_rules().iter().any(|rule| {
            let RuleBehavior::Context(behavior) = &rule.behavior else {
                return false;
            };

            behavior.effect.experiencer == Some(Experiencer::Other)
        },));
    }

    #[test]
    fn negex_rules_include_possible_assertion() {
        assert!(negex_rules().iter().any(|rule| {
            let RuleBehavior::Context(behavior) = &rule.behavior else {
                return false;
            };

            behavior.effect.assertion == Some(Assertion::Possible)
        },));
    }

    #[test]
    fn negex_rules_do_not_modify_temporality() {
        assert!(negex_rules().iter().all(|rule| {
            let RuleBehavior::Context(behavior) = &rule.behavior else {
                return true;
            };

            behavior.effect.temporality.is_none()
        },));
    }
}
