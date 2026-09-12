pub mod assertion;
pub mod control;
pub mod experiencer;
pub mod temporality;

use crate::engine::rule::ContextRule;

pub fn default_rules() -> Vec<ContextRule> {
    let mut rules = Vec::new();

    rules.extend(assertion::rules());

    rules.extend(temporality::rules());

    rules.extend(experiencer::rules());

    rules.extend(control::rules());

    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn default_rules_are_not_empty() {
        assert!(!default_rules().is_empty());
    }

    #[test]
    fn default_rules_compile() {
        assert!(RuleSet::compile(&default_rules(),).is_ok());
    }

    #[test]
    fn default_rules_include_context_rules() {
        assert!(
            default_rules()
                .iter()
                .any(|rule| { matches!(rule.behavior, RuleBehavior::Context(_)) })
        );
    }

    #[test]
    fn default_rules_include_terminators() {
        assert!(
            default_rules()
                .iter()
                .any(|rule| { rule.behavior == RuleBehavior::Terminate })
        );
    }

    #[test]
    fn default_rules_include_pseudo_rules() {
        assert!(
            default_rules()
                .iter()
                .any(|rule| { rule.behavior == RuleBehavior::Pseudo })
        );
    }
}
