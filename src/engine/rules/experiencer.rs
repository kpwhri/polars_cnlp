use crate::engine::finding::{ContextEffect, Experiencer};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn other() -> ContextEffect {
    ContextEffect::new().with_experiencer(Experiencer::Other)
}

/// Default ConText experiencer rules.
pub fn rules() -> Vec<ContextRule> {
    let other = other();

    vec![
        ContextRule::context(
            r"\bfamily\W+history\W+of\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bfamily\W+hx\W+of\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bmother(?:'s)?\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bfather(?:'s)?\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bsister(?:'s)?\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bbrother(?:'s)?\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b(?:son|daughter|grandmother|grandfather)(?:'s)?\b",
            Direction::Forward,
            other,
            ContextOptions::default(),
        ),
        ContextRule::pseudo_effects(
            r"\bby\W+(?:her|his)\W+(?:husband|wife|brother|sister)\b",
            vec![other],
        ),
        ContextRule::terminate_effects(r"\bpatient\b", vec![other]),
        ContextRule::terminate_effects(r"\bpresent(?:s|ed|ing)?\b", vec![other]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::RuleSet;

    #[test]
    fn experiencer_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }
}
