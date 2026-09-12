use super::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assertion {
    Affirmed,
    Possible,
    Negated,
}

impl Assertion {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Affirmed => "affirmed",
            Self::Possible => "possible",
            Self::Negated => "negated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temporality {
    Current,
    Historical,
}

impl Temporality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Historical => "historical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Experiencer {
    Patient,
    Other,
}

impl Experiencer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextEffect {
    pub assertion: Option<Assertion>,
    pub temporality: Option<Temporality>,
    pub experiencer: Option<Experiencer>,
}

impl ContextEffect {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_assertion(mut self, assertion: Assertion) -> Self {
        self.assertion = Some(assertion);
        self
    }

    pub fn with_temporality(mut self, temporality: Temporality) -> Self {
        self.temporality = Some(temporality);
        self
    }

    pub fn with_experiencer(mut self, experiencer: Experiencer) -> Self {
        self.experiencer = Some(experiencer);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.assertion.is_none() && self.temporality.is_none() && self.experiencer.is_none()
    }
}

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

    pub fn is_affirmed(&self) -> bool {
        self.assertion == Assertion::Affirmed
            && self.temporality == Temporality::Current
            && self.experiencer == Experiencer::Patient
    }
}

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
    fn assertion_effect_changes_only_assertion() {
        let mut context = FindingContext::default();

        context.apply(ContextEffect::new().with_assertion(Assertion::Negated));

        assert_eq!(context.assertion, Assertion::Negated);
        assert_eq!(context.temporality, Temporality::Current);
        assert_eq!(context.experiencer, Experiencer::Patient);
    }

    #[test]
    fn effect_can_change_multiple_dimensions() {
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
    fn nonaffirmed_contexts_are_not_affirmed() {
        for effect in [
            ContextEffect::new().with_assertion(Assertion::Negated),
            ContextEffect::new().with_assertion(Assertion::Possible),
            ContextEffect::new().with_temporality(Temporality::Historical),
            ContextEffect::new().with_experiencer(Experiencer::Other),
        ] {
            let mut context = FindingContext::default();
            context.apply(effect);

            assert!(!context.is_affirmed());
        }
    }

    #[test]
    fn new_effect_is_empty() {
        assert!(ContextEffect::new().is_empty());
    }
}
