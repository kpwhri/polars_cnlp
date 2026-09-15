use serde::Deserialize;

use crate::engine::finding::ContextEffect;
use crate::engine::rule::{ContextOptions, ContextRule, Direction};
use crate::engine::rules;

use super::{
    AlgorithmError, ConTextAlgorithm, ContextAlgorithm, DEFAULT_NEGEX_WINDOW, NegExAlgorithm,
    RuleBasedAlgorithm, ScopePolicy,
};

/// Serializable algorithm configuration received from Python.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AlgorithmSpec {
    Context {
        #[serde(default)]
        rules: Option<Vec<RuleSpec>>,

        #[serde(default)]
        additional_rules: Vec<RuleSpec>,
    },

    Negex {
        #[serde(default)]
        rules: Option<Vec<RuleSpec>>,

        #[serde(default)]
        additional_rules: Vec<RuleSpec>,

        #[serde(default = "default_negex_window")]
        window: usize,

        #[serde(default = "default_true")]
        propagate_same_concept: bool,
    },

    RuleBased {
        rules: Vec<RuleSpec>,

        #[serde(default)]
        window: Option<usize>,
    },
}

impl Default for AlgorithmSpec {
    fn default() -> Self {
        Self::Context {
            rules: None,
            additional_rules: Vec::new(),
        }
    }
}

impl AlgorithmSpec {
    /// Build the configured runtime algorithm.
    pub fn build(&self) -> Result<Box<dyn ContextAlgorithm>, AlgorithmError> {
        match self {
            Self::Context {
                rules: replacement_rules,
                additional_rules,
            } => {
                let resolved_rules =
                    resolve_rules(replacement_rules, additional_rules, rules::context_rules)?;

                Ok(Box::new(ConTextAlgorithm::compile(&resolved_rules)?))
            }

            Self::Negex {
                rules: replacement_rules,
                additional_rules,
                window,
                propagate_same_concept,
            } => {
                if *window == 0 {
                    return Err(AlgorithmError::InvalidConfig(
                        "NegEx window must be at least 1".to_string(),
                    ));
                }

                let resolved_rules =
                    resolve_rules(replacement_rules, additional_rules, rules::negex_rules)?;

                Ok(Box::new(NegExAlgorithm::compile(
                    &resolved_rules,
                    *window,
                    *propagate_same_concept,
                )?))
            }

            Self::RuleBased { rules, window } => {
                if matches!(window, Some(0)) {
                    return Err(AlgorithmError::InvalidConfig(
                        "rule-based window must be at least 1".to_string(),
                    ));
                }

                let rules = compile_rule_specs(rules)?;

                let scope_policy = match window {
                    Some(window) => ScopePolicy::FixedWindow(*window),

                    None => ScopePolicy::RuleDefined,
                };

                Ok(Box::new(RuleBasedAlgorithm::compile(&rules, scope_policy)?))
            }
        }
    }
}

/// Serializable representation of one lexical context rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RuleSpec {
    Context {
        pattern: String,
        direction: Direction,
        effect: ContextEffect,

        #[serde(default)]
        max_scope: Option<usize>,

        #[serde(default)]
        max_targets: Option<usize>,

        #[serde(default)]
        terminated_by: Vec<ContextEffect>,
    },

    Terminate {
        pattern: String,

        #[serde(default)]
        effects: Vec<ContextEffect>,
    },

    Pseudo {
        pattern: String,

        #[serde(default)]
        effects: Vec<ContextEffect>,
    },
}

impl RuleSpec {
    fn to_context_rule(&self) -> Result<ContextRule, AlgorithmError> {
        match self {
            Self::Context {
                pattern,
                direction,
                effect,
                max_scope,
                max_targets,
                terminated_by,
            } => {
                validate_pattern(pattern)?;

                if effect.is_empty() {
                    return Err(AlgorithmError::InvalidConfig(
                        "context rule effect must modify at least one context dimension"
                            .to_string(),
                    ));
                }

                if matches!(max_targets, Some(0)) {
                    return Err(AlgorithmError::InvalidConfig(
                        "max_targets must be at least 1".to_string(),
                    ));
                }

                if terminated_by.iter().any(ContextEffect::is_empty) {
                    return Err(AlgorithmError::InvalidConfig(
                        "terminated_by effects must not be empty".to_string(),
                    ));
                }

                Ok(ContextRule::context(
                    pattern,
                    *direction,
                    *effect,
                    ContextOptions {
                        max_scope: *max_scope,
                        max_targets: *max_targets,
                        terminated_by: terminated_by.clone(),
                    },
                ))
            }

            Self::Terminate { pattern, effects } => {
                validate_pattern(pattern)?;

                if effects.iter().any(ContextEffect::is_empty) {
                    return Err(AlgorithmError::InvalidConfig(
                        "termination effects must not be empty".to_string(),
                    ));
                }

                Ok(ContextRule::terminate_effects(pattern, effects.clone()))
            }

            Self::Pseudo { pattern, effects } => {
                validate_pattern(pattern)?;

                if effects.iter().any(ContextEffect::is_empty) {
                    return Err(AlgorithmError::InvalidConfig(
                        "pseudo effects must not be empty".to_string(),
                    ));
                }

                Ok(ContextRule::pseudo_effects(pattern, effects.clone()))
            }
        }
    }
}

fn resolve_rules(
    replacement_rules: &Option<Vec<RuleSpec>>,
    additional_rules: &[RuleSpec],
    default_rules: fn() -> Vec<ContextRule>,
) -> Result<Vec<ContextRule>, AlgorithmError> {
    if replacement_rules.is_some() && !additional_rules.is_empty() {
        return Err(AlgorithmError::InvalidConfig(
            "rules and additional_rules cannot be used together".to_string(),
        ));
    }

    match replacement_rules {
        Some(replacement_rules) => compile_rule_specs(replacement_rules),

        None => {
            let mut rules = default_rules();

            rules.extend(compile_rule_specs(additional_rules)?);

            Ok(rules)
        }
    }
}

fn compile_rule_specs(rules: &[RuleSpec]) -> Result<Vec<ContextRule>, AlgorithmError> {
    rules.iter().map(RuleSpec::to_context_rule).collect()
}

fn validate_pattern(pattern: &str) -> Result<(), AlgorithmError> {
    if pattern.is_empty() {
        return Err(AlgorithmError::InvalidConfig(
            "rule patterns must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn default_negex_window() -> usize {
    DEFAULT_NEGEX_WINDOW
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, Temporality};

    fn negation_rule(pattern: &str) -> RuleSpec {
        RuleSpec::Context {
            pattern: pattern.to_string(),
            direction: Direction::Forward,
            effect: ContextEffect::new().with_assertion(Assertion::Negated),
            max_scope: None,
            max_targets: None,
            terminated_by: Vec::new(),
        }
    }

    #[test]
    fn context_is_default_algorithm() {
        let spec = AlgorithmSpec::default();

        let AlgorithmSpec::Context {
            rules,
            additional_rules,
        } = spec
        else {
            panic!("expected default ConText algorithm");
        };

        assert!(rules.is_none());
        assert!(additional_rules.is_empty());
    }

    #[test]
    fn builds_default_context() {
        assert!(AlgorithmSpec::default().build().is_ok());
    }

    #[test]
    fn builds_default_negex() {
        assert!(
            AlgorithmSpec::Negex {
                rules: None,
                additional_rules: Vec::new(),
                window: 6,
                propagate_same_concept: true,
            }
            .build()
            .is_ok()
        );
    }

    #[test]
    fn additional_context_rules_extend_defaults() {
        let default_count = rules::context_rules().len();

        let additional_rules = vec![negation_rule(r"\bcovid\b")];

        let resolved = resolve_rules(&None, &additional_rules, rules::context_rules).unwrap();

        assert_eq!(resolved.len(), default_count + 1,);
    }

    #[test]
    fn additional_negex_rules_extend_defaults() {
        let default_count = rules::negex_rules().len();

        let additional_rules = vec![negation_rule(r"\bcovid\b")];

        let resolved = resolve_rules(&None, &additional_rules, rules::negex_rules).unwrap();

        assert_eq!(resolved.len(), default_count + 1,);
    }

    #[test]
    fn replacement_rules_do_not_include_defaults() {
        let replacement_rules = Some(vec![negation_rule(r"\bcovid\b")]);

        let resolved = resolve_rules(&replacement_rules, &[], rules::negex_rules).unwrap();

        assert_eq!(resolved.len(), 1,);
    }

    #[test]
    fn replacement_and_additional_rules_are_rejected() {
        let replacement_rules = Some(vec![negation_rule(r"\bcovid\b")]);

        let additional_rules = vec![negation_rule(r"\babsence\b")];

        assert!(resolve_rules(&replacement_rules, &additional_rules, rules::negex_rules,).is_err());
    }

    #[test]
    fn builds_context_with_additional_rules() {
        let spec = AlgorithmSpec::Context {
            rules: None,
            additional_rules: vec![negation_rule(r"\bcovid\b")],
        };

        assert!(spec.build().is_ok());
    }

    #[test]
    fn builds_negex_with_additional_rules() {
        let spec = AlgorithmSpec::Negex {
            rules: None,
            additional_rules: vec![negation_rule(r"\bcovid\b")],
            window: 6,
            propagate_same_concept: true,
        };

        assert!(spec.build().is_ok());
    }

    #[test]
    fn builds_custom_rule_based_algorithm() {
        let spec = AlgorithmSpec::RuleBased {
            rules: vec![negation_rule(r"\babsent\b")],
            window: None,
        };

        assert!(spec.build().is_ok());
    }

    #[test]
    fn rejects_empty_context_effect() {
        let spec = AlgorithmSpec::RuleBased {
            rules: vec![RuleSpec::Context {
                pattern: r"\babsent\b".to_string(),
                direction: Direction::Forward,
                effect: ContextEffect::new(),
                max_scope: None,
                max_targets: None,
                terminated_by: Vec::new(),
            }],
            window: None,
        };

        assert!(spec.build().is_err());
    }

    #[test]
    fn rejects_zero_negex_window() {
        let spec = AlgorithmSpec::Negex {
            rules: None,
            additional_rules: Vec::new(),
            window: 0,
            propagate_same_concept: true,
        };

        assert!(spec.build().is_err());
    }

    #[test]
    fn custom_temporality_rule_builds() {
        let spec = AlgorithmSpec::Context {
            rules: None,
            additional_rules: vec![RuleSpec::Context {
                pattern: r"\bremote\b".to_string(),
                direction: Direction::Forward,
                effect: ContextEffect::new().with_temporality(Temporality::Historical),
                max_scope: None,
                max_targets: None,
                terminated_by: Vec::new(),
            }],
        };

        assert!(spec.build().is_ok());
    }
}
