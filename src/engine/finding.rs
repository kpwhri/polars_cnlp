use serde::Deserialize;

use super::span::Span;

/// Assertion status of a clinical finding.
///
/// `Possible` is retained for NegEx PREP/POSP rules and custom algorithms.
/// Published ConText itself primarily uses `Affirmed` and `Negated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assertion {
    Affirmed,
    Possible,
    Negated,
}

impl Assertion {
    /// Return the stable external representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Affirmed => "affirmed",
            Self::Possible => "possible",
            Self::Negated => "negated",
        }
    }
}

/// Temporal status of a clinical finding.
///
/// `Current` corresponds to the "recent" default in the original ConText paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Temporality {
    Current,
    Historical,
    Hypothetical,
}

impl Temporality {
    /// Return the stable external representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Historical => "historical",
            Self::Hypothetical => "hypothetical",
        }
    }
}

/// Person who experiences the clinical finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Experiencer {
    Patient,
    Other,
}

impl Experiencer {
    /// Return the stable external representation of this value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::Other => "other",
        }
    }
}

/// Changes to one or more independent contextual dimensions.
///
/// `None` means that a rule does not alter that dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(default)]
pub struct ContextEffect {
    pub assertion: Option<Assertion>,
    pub temporality: Option<Temporality>,
    pub experiencer: Option<Experiencer>,
}

impl ContextEffect {
    /// Create an empty context effect.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the assertion effect.
    pub fn with_assertion(mut self, assertion: Assertion) -> Self {
        self.assertion = Some(assertion);
        self
    }

    /// Set the temporality effect.
    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    /// Set the experiencer effect.
    pub fn with_experiencer(mut self, experiencer: Experiencer) -> Self {
        self.experiencer = Some(experiencer);
        self
    }

    /// Return whether this effect changes no contextual dimensions.
    pub fn is_empty(&self) -> bool {
        self.assertion.is_none() && self.temporality.is_none() && self.experiencer.is_none()
    }

    /// Return whether two effects assign at least one identical value.
    ///
    /// This is used by selective termination and pseudo rules.
    pub fn intersects(&self, other: &Self) -> bool {
        (self.assertion.is_some() && self.assertion == other.assertion)
            || (self.temporality.is_some() && self.temporality == other.temporality)
            || (self.experiencer.is_some() && self.experiencer == other.experiencer)
    }
}

/// Fully resolved context of a clinical finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingContext {
    pub assertion: Assertion,
    pub temporality: Temporality,
    pub experiencer: Experiencer,
}

impl Default for FindingContext {
    fn default() -> Self {
        Self {
            assertion: Assertion::Affirmed,
            temporality: Temporality::Current,
            experiencer: Experiencer::Patient,
        }
    }
}

impl FindingContext {
    /// Apply a partial effect to this context.
    pub fn apply(&mut self, effect: ContextEffect) {
        if let Some(assertion) = effect.assertion {
            self.assertion = assertion;
        }

        if let Some(temporality) = effect.temporality {
            self.temporality = temporality;
        }

        if let Some(experiencer) = effect.experiencer {
            self.experiencer = experiencer;
        }
    }

    /// Return whether the finding represents an affirmed current patient finding.
    pub fn is_affirmed(&self) -> bool {
        self.assertion == Assertion::Affirmed
            && self.temporality == Temporality::Current
            && self.experiencer == Experiencer::Patient
    }
}

/// Contextualized occurrence of a target concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    pub span: Span,
    pub context: FindingContext,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_values_have_stable_names() {
        assert_eq!(Assertion::Affirmed.as_str(), "affirmed");
        assert_eq!(Assertion::Possible.as_str(), "possible");
        assert_eq!(Assertion::Negated.as_str(), "negated");

        assert_eq!(Temporality::Current.as_str(), "current");
        assert_eq!(Temporality::Historical.as_str(), "historical");
        assert_eq!(Temporality::Hypothetical.as_str(), "hypothetical");

        assert_eq!(Experiencer::Patient.as_str(), "patient");
        assert_eq!(Experiencer::Other.as_str(), "other");
    }

    #[test]
    fn default_context_is_affirmed() {
        let context = FindingContext::default();

        assert_eq!(context.assertion, Assertion::Affirmed);
        assert_eq!(context.temporality, Temporality::Current);
        assert_eq!(context.experiencer, Experiencer::Patient);
        assert!(context.is_affirmed());
    }

    #[test]
    fn effect_changes_only_selected_dimensions() {
        let mut context = FindingContext::default();

        context.apply(
            ContextEffect::new()
                .with_temporality(Temporality::Historical)
                .with_experiencer(Experiencer::Other),
        );

        assert_eq!(context.assertion, Assertion::Affirmed);
        assert_eq!(context.temporality, Temporality::Historical);
        assert_eq!(context.experiencer, Experiencer::Other);
    }

    #[test]
    fn hypothetical_context_is_not_affirmed() {
        let context = FindingContext {
            temporality: Temporality::Hypothetical,
            ..FindingContext::default()
        };

        assert!(!context.is_affirmed());
    }

    #[test]
    fn possible_context_is_not_affirmed() {
        let context = FindingContext {
            assertion: Assertion::Possible,
            ..FindingContext::default()
        };

        assert!(!context.is_affirmed());
    }

    #[test]
    fn effects_intersect_on_equal_dimension_values() {
        let historical = ContextEffect::new().with_temporality(Temporality::Historical);

        let historical_other = ContextEffect::new()
            .with_temporality(Temporality::Historical)
            .with_experiencer(Experiencer::Other);

        assert!(historical.intersects(&historical_other));
    }

    #[test]
    fn effects_do_not_intersect_on_different_values() {
        let historical = ContextEffect::new().with_temporality(Temporality::Historical);

        let hypothetical = ContextEffect::new().with_temporality(Temporality::Hypothetical);

        assert!(!historical.intersects(&hypothetical));
    }
}
