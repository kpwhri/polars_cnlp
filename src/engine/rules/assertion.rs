use crate::engine::finding::{Assertion, ContextEffect};
use crate::engine::rule::{ContextOptions, ContextRule, Direction};

fn negated() -> ContextEffect {
    ContextEffect::new().with_assertion(Assertion::Negated)
}

fn possible() -> ContextEffect {
    ContextEffect::new().with_assertion(Assertion::Possible)
}

/// Default negation modifier rules.
pub fn negation_rules() -> Vec<ContextRule> {
    let effect = negated();

    vec![
        ContextRule::context(
            r"\bno\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bnot\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bwithout\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bden(?:y|ies|ied|ying)\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\babsence\W+of\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bfree\W+of\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bnegative\W+for\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bno\W+evidence(?:\W+of)?\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bno\W+signs?\W+of\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b(?:is|are|was|were)\W+ruled\W+out\b",
            Direction::Backward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b(?:not\W+seen|not\W+present|not\W+identified)\b",
            Direction::Backward,
            effect,
            ContextOptions::default(),
        ),
    ]
}

pub fn context_possible_rules() -> Vec<ContextRule> {
    let possible = ContextEffect::new().with_assertion(Assertion::Possible);

    vec![
        ContextRule::context(
            r"\bpossible\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bpossibly\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bprobable\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bprobably\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\blikely\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bconcern(?:ed)?\W+(?:for|about)\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bsuspicious\W+for\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bsuggestive\W+of\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\brule\W+out\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\br\s*/\s*o\b",
            Direction::Forward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bcannot\W+(?:be\W+)?(?:excluded|ruled\W+out)\b",
            Direction::Backward,
            possible,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bnot\W+(?:been\W+)?ruled\W+out\b",
            Direction::Backward,
            possible,
            ContextOptions::default(),
        ),
    ]
}

/// Default NegEx-style conditional-possibility rules.
pub fn negex_possible_rules() -> Vec<ContextRule> {
    let effect = possible();

    vec![
        ContextRule::context(
            r"\brule\W+out\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\br\s*/\s*o\b",
            Direction::Forward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\b(?:should|may|might|could|must)\W+be\W+ruled\W+out\b",
            Direction::Backward,
            effect,
            ContextOptions::default(),
        ),
        ContextRule::context(
            r"\bnot\W+(?:been\W+)?ruled\W+out\b",
            Direction::Backward,
            effect,
            ContextOptions::default(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::RuleSet;

    #[test]
    fn negation_rules_compile() {
        assert!(RuleSet::compile(&negation_rules(),).is_ok());
    }

    #[test]
    fn possibility_rules_compile() {
        assert!(RuleSet::compile(&negex_possible_rules(),).is_ok());
    }
}
