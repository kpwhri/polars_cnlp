use crate::engine::finding::{ContextEffect, Temporality};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn historical() -> ContextEffect {
    ContextEffect::new().with_temporality(Temporality::Historical)
}

fn hypothetical() -> ContextEffect {
    ContextEffect::new().with_temporality(Temporality::Hypothetical)
}

/// Default ConText temporality rules.
pub fn rules() -> Vec<ContextRule> {
    let historical = historical();
    let hypothetical = hypothetical();

    vec![
        ContextRule::context(
            r"\bhistory\W+of\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bhx\W+of\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bpast\W+history\W+of\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bprior\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bprevious(?:ly)?\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bremote\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bpre[\W_]*existing\b",
            Direction::Forward,
            historical,
            ContextOptions {
                max_targets: Some(1),
                ..ContextOptions::default()
            },
        ),
        ContextRule::context(
            r"\bstatus\W+post\b",
            Direction::Forward,
            historical,
            ContextOptions {
                max_targets: Some(1),
                ..ContextOptions::default()
            },
        ),
        ContextRule::context(
            r"\bs\s*/\s*p\b",
            Direction::Forward,
            historical,
            ContextOptions {
                max_targets: Some(1),
                ..ContextOptions::default()
            },
        ),
        ContextRule::context(
            r"\b(?:1[5-9]|[2-9]\d|\d{3,})\W+days?\W+ago\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b(?:[3-9]|\d{2,})\W+weeks?\W+ago\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b\d+\W+(?:months?|years?)\W+ago\b",
            Direction::Forward,
            historical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bif\b",
            Direction::Forward,
            hypothetical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bshould\b",
            Direction::Forward,
            hypothetical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bmay\b",
            Direction::Forward,
            hypothetical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bmight\b",
            Direction::Forward,
            hypothetical,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bcould\b",
            Direction::Forward,
            hypothetical,
            ContextOptions::default(),
        ),
        ContextRule::pseudo_effects(r"\bhistory\W+exam\b", vec![historical]),
        ContextRule::pseudo_effects(r"\bsocial\W+history\b", vec![historical]),
        ContextRule::pseudo_effects(r"\bpoor\W+histor(?:y|ian)\b", vec![historical]),
        ContextRule::pseudo_effects(r"\bif\W+negative\b", vec![hypothetical]),
        ContextRule::pseudo_effects(r"\bknow\W+if\b", vec![hypothetical]),
        ContextRule::terminate_effects(r"\bpresent(?:s|ed|ing)?\b", vec![historical, hypothetical]),
        ContextRule::terminate_effects(
            r"\b(?:currently|today|now|recently)\b",
            vec![historical, hypothetical],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::RuleSet;

    #[test]
    fn temporality_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }

    #[test]
    fn historical_single_target_rules_exist() {
        assert!(rules().iter().any(|rule| {
            let crate::engine::rule::RuleBehavior::Context(behavior) = &rule.behavior else {
                return false;
            };

            behavior.options.max_targets == Some(1)
        },));
    }
}
