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
    },

    Negex {
        #[serde(default)]
        rules: Option<Vec<RuleSpec>>,

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
        Self::Context { rules: None }
    }
}

impl AlgorithmSpec {
    /// Build the configured runtime algorithm.
    pub fn build(&self) -> Result<Box<dyn ContextAlgorithm>, AlgorithmError> {
        match self {
            Self::Context { rules } => {
                let custom_rules = compile_optional_rules(rules)?;

                match custom_rules {
                    Some(rules) => Ok(Box::new(ConTextAlgorithm::compile(&rules)?)),

                    None => Ok(Box::new(ConTextAlgorithm::compile_default()?)),
                }
            }

            Self::Negex {
                rules,
                window,
                propagate_same_concept,
            } => {
                if *window == 0 {
                    return Err(AlgorithmError::InvalidConfig(
                        "NegEx window must be at least 1".to_string(),
                    ));
                }

                let rules = match compile_optional_rules(rules)? {
                    Some(rules) => rules,
                    None => rules::negex_rules(),
                };

                Ok(Box::new(NegExAlgorithm::compile(
                    &rules,
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

fn compile_optional_rules(
    rules: &Option<Vec<RuleSpec>>,
) -> Result<Option<Vec<ContextRule>>, AlgorithmError> {
    rules
        .as_ref()
        .map(|rules| compile_rule_specs(rules))
        .transpose()
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

    #[test]
    fn context_is_default_algorithm() {
        assert!(matches!(
            AlgorithmSpec::default(),
            AlgorithmSpec::Context { rules: None }
        ));
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
                window: 6,
                propagate_same_concept: true,
            }
            .build()
            .is_ok()
        );
    }

    #[test]
    fn custom_context_rules_replace_defaults() {
        let spec = AlgorithmSpec::Context {
            rules: Some(vec![RuleSpec::Context {
                pattern: r"\bremote\b".to_string(),
                direction: Direction::Forward,
                effect: ContextEffect::new().with_temporality(Temporality::Historical),
                max_scope: None,
                max_targets: None,
                terminated_by: Vec::new(),
            }]),
        };

        assert!(spec.build().is_ok());
    }

    #[test]
    fn builds_custom_rule_based_algorithm() {
        let spec = AlgorithmSpec::RuleBased {
            rules: vec![RuleSpec::Context {
                pattern: r"\babsent\b".to_string(),
                direction: Direction::Forward,
                effect: ContextEffect::new().with_assertion(Assertion::Negated),
                max_scope: None,
                max_targets: None,
                terminated_by: Vec::new(),
            }],
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
            window: 0,
            propagate_same_concept: true,
        };

        assert!(spec.build().is_err());
    }
}
