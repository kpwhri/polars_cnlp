use std::collections::HashSet;

use polars::prelude::*;
use serde::Deserialize;

use crate::engine::algorithm::AlgorithmSpec;
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

    #[serde(default)]
    pub algorithm: AlgorithmSpec,

    #[serde(default)]
    pub prefilter: bool,
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

    let values = findings
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
        .collect::<Vec<_>>();

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
