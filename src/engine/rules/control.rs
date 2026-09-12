use crate::engine::rule::ContextRule;

pub fn rules() -> Vec<ContextRule> {
    vec![
        ContextRule::terminate(r"\bbut\b"),
        ContextRule::terminate(r"\bhowever\b"),
        ContextRule::terminate(r"\balthough\b"),
        ContextRule::terminate(r"\bexcept\b"),
        ContextRule::pseudo(r"\bnot\W+only\b"),
        ContextRule::pseudo(r"\bwithout\W+contrast\b"),
        ContextRule::pseudo(r"\bno\W+increase\b"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{RuleBehavior, RuleSet};

    #[test]
    fn control_rules_compile() {
        assert!(RuleSet::compile(&rules(),).is_ok());
    }

    #[test]
    fn but_is_terminator() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bbut\b")
            .unwrap();

        assert_eq!(rule.behavior, RuleBehavior::Terminate,);
    }

    #[test]
    fn not_only_is_pseudo() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bnot\W+only\b")
            .unwrap();

        assert_eq!(rule.behavior, RuleBehavior::Pseudo,);
    }

    #[test]
    fn without_contrast_is_pseudo() {
        let rule = rules()
            .into_iter()
            .find(|rule| rule.pattern == r"\bwithout\W+contrast\b")
            .unwrap();

        assert_eq!(rule.behavior, RuleBehavior::Pseudo,);
    }

    #[test]
    fn finds_multiple_control_rules() {
        let matches = RuleSet::compile(&rules())
            .unwrap()
            .find_matches("Not only pneumonia but asthma.");

        assert_eq!(matches.len(), 2,);
    }
}
