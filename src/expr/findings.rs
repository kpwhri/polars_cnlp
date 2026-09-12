use std::collections::HashSet;

use polars::prelude::*;
use serde::Deserialize;

use crate::engine::analyze::IndexedFinding;
use crate::engine::concept::Concept;

#[derive(Debug, Clone, Deserialize)]
pub struct FindTermSpec {
    pub label: Option<String>,
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FindKwargs {
    pub terms: Vec<FindTermSpec>,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderedFinding<'a> {
    pub label: Option<&'a str>,
    pub indexed: IndexedFinding,
}

impl FindKwargs {
    pub fn validate(&self) -> PolarsResult<()> {
        if self.terms.is_empty() {
            polars_bail!(
                ComputeError:
                "find terms must contain at least one regular expression"
            );
        }

        let unlabeled = self
            .terms
            .iter()
            .filter(|term| term.label.is_none())
            .count();

        if unlabeled > 0 && (self.terms.len() != 1 || unlabeled != 1) {
            polars_bail!(
                ComputeError:
                "multiple find terms must have labels"
            );
        }

        let mut labels = HashSet::with_capacity(self.terms.len());

        for term in &self.terms {
            if term.pattern.is_empty() {
                polars_bail!(
                    ComputeError:
                    "find regular expressions must not be empty"
                );
            }

            if let Some(label) = &term.label {
                if label.trim().is_empty() {
                    polars_bail!(
                        ComputeError:
                        "term labels must not be empty"
                    );
                }

                if !labels.insert(label.as_str()) {
                    polars_bail!(
                        ComputeError:
                        "duplicate term label '{}'",
                        label
                    );
                }
            }
        }

        Ok(())
    }

    pub fn render(&self, indexed: IndexedFinding) -> RenderedFinding<'_> {
        RenderedFinding {
            label: self.terms[indexed.concept_index].label.as_deref(),
            indexed,
        }
    }
}

pub fn compile_find_concepts(kwargs: &FindKwargs) -> PolarsResult<Vec<Concept>> {
    kwargs.validate()?;

    kwargs
        .terms
        .iter()
        .map(|term| {
            Concept::new(&term.pattern).map_err(|error| match &term.label {
                Some(label) => {
                    polars_err!(
                        ComputeError:
                        "invalid regex for term '{}': {}",
                        label,
                        error
                    )
                }

                None => {
                    polars_err!(
                        ComputeError:
                        "invalid regex: {}",
                        error
                    )
                }
            })
        })
        .collect()
}

pub fn finding_dtype() -> DataType {
    DataType::Struct(finding_fields())
}

pub fn find_best_output(input_fields: &[Field]) -> PolarsResult<Field> {
    let input = input_fields.first().ok_or_else(|| {
        polars_err!(
            ComputeError:
            "CNLP expression requires one input field"
        )
    })?;

    Ok(Field::new(input.name().clone(), finding_dtype()))
}

pub fn find_all_output(input_fields: &[Field]) -> PolarsResult<Field> {
    let input = input_fields.first().ok_or_else(|| {
        polars_err!(
            ComputeError:
            "CNLP expression requires one input field"
        )
    })?;

    Ok(Field::new(
        input.name().clone(),
        DataType::List(Box::new(finding_dtype())),
    ))
}

pub fn build_finding_struct(
    name: PlSmallStr,
    findings: &[RenderedFinding<'_>],
) -> PolarsResult<Series> {
    let labels: Vec<Option<&str>> = findings.iter().map(|finding| finding.label).collect();

    let starts: Vec<u64> = findings
        .iter()
        .map(|finding| finding.indexed.finding.span.start as u64)
        .collect();

    let ends: Vec<u64> = findings
        .iter()
        .map(|finding| finding.indexed.finding.span.end as u64)
        .collect();

    let assertions: Vec<&str> = findings
        .iter()
        .map(|finding| finding.indexed.finding.context.assertion.as_str())
        .collect();

    let temporalities: Vec<&str> = findings
        .iter()
        .map(|finding| finding.indexed.finding.context.temporality.as_str())
        .collect();

    let experiencers: Vec<&str> = findings
        .iter()
        .map(|finding| finding.indexed.finding.context.experiencer.as_str())
        .collect();

    let fields = [
        Series::new("label".into(), labels),
        Series::new("start".into(), starts),
        Series::new("end".into(), ends),
        Series::new("assertion".into(), assertions),
        Series::new("temporality".into(), temporalities),
        Series::new("experiencer".into(), experiencers),
    ];

    Ok(StructChunked::from_series(name, findings.len(), fields.iter())?.into_series())
}

pub fn build_optional_finding_struct(
    name: PlSmallStr,
    findings: &[Option<RenderedFinding<'_>>],
) -> PolarsResult<Series> {
    let fields = finding_fields();

    let dtype = DataType::Struct(fields.clone());

    let values: Vec<AnyValue<'_>> = findings
        .iter()
        .map(|finding| {
            let Some(finding) = finding else {
                return AnyValue::Null;
            };

            let context = finding.indexed.finding.context;

            let label = finding
                .label
                .map(AnyValue::String)
                .unwrap_or(AnyValue::Null);

            AnyValue::StructOwned(Box::new((
                vec![
                    label,
                    AnyValue::UInt64(finding.indexed.finding.span.start as u64),
                    AnyValue::UInt64(finding.indexed.finding.span.end as u64),
                    AnyValue::String(context.assertion.as_str()),
                    AnyValue::String(context.temporality.as_str()),
                    AnyValue::String(context.experiencer.as_str()),
                ],
                fields.clone(),
            )))
        })
        .collect();

    Series::from_any_values_and_dtype(name, &values, &dtype, true)
}

fn finding_fields() -> Vec<Field> {
    vec![
        Field::new("label".into(), DataType::String),
        Field::new("start".into(), DataType::UInt64),
        Field::new("end".into(), DataType::UInt64),
        Field::new("assertion".into(), DataType::String),
        Field::new("temporality".into(), DataType::String),
        Field::new("experiencer".into(), DataType::String),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::finding::{Assertion, Experiencer, Finding, FindingContext, Temporality};
    use crate::engine::span::Span;

    fn indexed_finding() -> IndexedFinding {
        IndexedFinding {
            concept_index: 0,
            finding: Finding {
                span: Span::new(3, 12),
                context: FindingContext {
                    assertion: Assertion::Negated,
                    temporality: Temporality::Current,
                    experiencer: Experiencer::Patient,
                },
            },
        }
    }

    fn named_kwargs() -> FindKwargs {
        FindKwargs {
            terms: vec![FindTermSpec {
                label: Some("pneumonia".to_string()),
                pattern: r"\bpneumonia\b".to_string(),
            }],
        }
    }

    #[test]
    fn validates_single_unlabeled_term() {
        let kwargs = FindKwargs {
            terms: vec![FindTermSpec {
                label: None,
                pattern: r"\bpneumonia\b".to_string(),
            }],
        };

        assert!(kwargs.validate().is_ok());
    }

    #[test]
    fn validates_named_terms() {
        assert!(named_kwargs().validate().is_ok());
    }

    #[test]
    fn rejects_multiple_unlabeled_terms() {
        let kwargs = FindKwargs {
            terms: vec![
                FindTermSpec {
                    label: None,
                    pattern: r"\bpneumonia\b".to_string(),
                },
                FindTermSpec {
                    label: None,
                    pattern: r"\basthma\b".to_string(),
                },
            ],
        };

        assert!(kwargs.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_labels() {
        let kwargs = FindKwargs {
            terms: vec![
                FindTermSpec {
                    label: Some("disease".to_string()),
                    pattern: r"\bpneumonia\b".to_string(),
                },
                FindTermSpec {
                    label: Some("disease".to_string()),
                    pattern: r"\basthma\b".to_string(),
                },
            ],
        };

        assert!(kwargs.validate().is_err());
    }

    #[test]
    fn finding_dtype_is_struct() {
        assert_eq!(
            finding_dtype(),
            DataType::Struct(vec![
                Field::new("label".into(), DataType::String,),
                Field::new("start".into(), DataType::UInt64,),
                Field::new("end".into(), DataType::UInt64,),
                Field::new("assertion".into(), DataType::String,),
                Field::new("temporality".into(), DataType::String,),
                Field::new("experiencer".into(), DataType::String,),
            ],),
        );
    }

    #[test]
    fn builds_named_finding_struct() {
        let kwargs = named_kwargs();

        let rendered = kwargs.render(indexed_finding());

        let series = build_finding_struct("finding".into(), &[rendered]).unwrap();

        let value = series.get(0).unwrap();

        assert_eq!(value.dtype(), finding_dtype(),);
    }

    #[test]
    fn builds_null_optional_finding() {
        let series = build_optional_finding_struct("finding".into(), &[None]).unwrap();

        assert!(series.get(0).unwrap().is_null());
    }

    #[test]
    fn builds_unlabeled_finding_with_null_label() {
        let kwargs = FindKwargs {
            terms: vec![FindTermSpec {
                label: None,
                pattern: r"\bpneumonia\b".to_string(),
            }],
        };

        let series = build_optional_finding_struct(
            "finding".into(),
            &[Some(kwargs.render(indexed_finding()))],
        )
        .unwrap();

        assert!(!series.get(0).unwrap().is_null());

        let label = series.struct_().unwrap().field_by_name("label").unwrap();

        assert_eq!(label.null_count(), 1,);
    }
}
