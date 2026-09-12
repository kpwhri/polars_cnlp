use crate::engine::finding::{Assertion, ContextEffect};
use crate::engine::rule::ContextRule;

fn negated() -> ContextEffect {
    ContextEffect::new().with_assertion(Assertion::Negated)
}

fn possible() -> ContextEffect {
    ContextEffect::new().with_assertion(Assertion::Possible)
}

/// Pseudo-negation and scope-stop rules used for assertion processing.
pub fn assertion_rules() -> Vec<ContextRule> {
    let effects = vec![negated(), possible()];

    vec![
        ContextRule::pseudo_effects(r"\bno\W+increase\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bno\W+(?:significant\W+)?change\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bnot\W+cause\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bgram\W+negative\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bwithout\W+difficulty\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bnot\W+necessarily\b", effects.clone()),
        ContextRule::pseudo_effects(r"\bnot\W+only\b", effects.clone()),
        ContextRule::terminate_effects(r"\bbut\b", effects.clone()),
        ContextRule::terminate_effects(r"\bhowever\b", effects.clone()),
        ContextRule::terminate_effects(r"\bnevertheless\b", effects.clone()),
        ContextRule::terminate_effects(r"\balthough\b", effects.clone()),
        ContextRule::terminate_effects(r"\bexcept\b", effects.clone()),
        ContextRule::terminate_effects(r"\baside\W+from\b", effects),
    ]
}

/// General clause boundaries useful across ConText dimensions.
pub fn context_rules() -> Vec<ContextRule> {
    vec![
        ContextRule::terminate(r"\bbut\b"),
        ContextRule::terminate(r"\bhowever\b"),
        ContextRule::terminate(r"\bnevertheless\b"),
        ContextRule::terminate(r"\balthough\b"),
        ContextRule::terminate(r"\bexcept\b"),
        ContextRule::terminate(r"\baside\W+from\b"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::RuleSet;

    #[test]
    fn assertion_control_rules_compile() {
        assert!(RuleSet::compile(&assertion_rules(),).is_ok());
    }

    #[test]
    fn context_control_rules_compile() {
        assert!(RuleSet::compile(&context_rules(),).is_ok());
    }
}
