use std::collections::HashSet;

use polars::prelude::*;
use serde::Deserialize;

use crate::engine::algorithm::AlgorithmSpec;
use crate::engine::concept::Concept;

/// One labeled target pattern.
#[derive(Debug, Clone, Deserialize)]
pub struct TermSpec {
    pub label: String,
    pub pattern: String,
}

/// Keyword arguments shared by multi-term expressions.
///
/// Count expressions ignore `algorithm`; contextual expressions use it.
#[derive(Debug, Clone, Deserialize)]
pub struct TermsKwargs {
    pub terms: Vec<TermSpec>,

    #[serde(default)]
    pub algorithm: AlgorithmSpec,

    #[serde(default)]
    pub prefilter: bool,
}

pub fn validate_terms(terms: &[TermSpec]) -> PolarsResult<()> {
    if terms.is_empty() {
        polars_bail!(
            ComputeError:
            "terms must contain at least one regular expression"
        );
    }

    let mut labels = HashSet::with_capacity(terms.len());

    for term in terms {
        if term.label.trim().is_empty() {
            polars_bail!(
                ComputeError:
                "term labels must not be empty"
            );
        }

        if term.pattern.is_empty() {
            polars_bail!(
                ComputeError:
                "regex for term '{}' must not be empty",
                term.label
            );
        }

        if !labels.insert(term.label.as_str()) {
            polars_bail!(
                ComputeError:
                "duplicate term label '{}'",
                term.label
            );
        }
    }

    Ok(())
}

pub fn compile_concepts(terms: &[TermSpec]) -> PolarsResult<Vec<Concept>> {
    validate_terms(terms)?;

    terms
        .iter()
        .map(|term| {
            Concept::new(&term.pattern).map_err(|error| {
                polars_err!(
                    ComputeError:
                    "invalid regex for term '{}': {}",
                    term.label,
                    error
                )
            })
        })
        .collect()
}

pub fn labels(terms: &[TermSpec]) -> Vec<String> {
    terms.iter().map(|term| term.label.clone()).collect()
}

pub fn struct_output_field(
    input_fields: &[Field],
    terms: &[TermSpec],
    inner_dtype: DataType,
) -> PolarsResult<Field> {
    validate_terms(terms)?;

    let input_field = input_fields.first().ok_or_else(|| {
        polars_err!(
            ComputeError:
            "CNLP expression requires one input field"
        )
    })?;

    let fields = terms
        .iter()
        .map(|term| Field::new(term.label.clone().into(), inner_dtype.clone()))
        .collect();

    Ok(Field::new(
        input_field.name().clone(),
        DataType::Struct(fields),
    ))
}

pub fn build_boolean_struct(
    labels: &[String],
    columns: Vec<Vec<Option<bool>>>,
    length: usize,
) -> PolarsResult<Series> {
    if labels.len() != columns.len() {
        polars_bail!(
            ComputeError:
            "internal CNLP error: label and result column counts differ"
        );
    }

    let fields = labels
        .iter()
        .zip(columns)
        .map(|(label, values)| Series::new(label.clone().into(), values))
        .collect::<Vec<_>>();

    Ok(StructChunked::from_series("".into(), length, fields.iter())?.into_series())
}

pub fn build_u32_struct(
    labels: &[String],
    columns: Vec<Vec<Option<u32>>>,
    length: usize,
) -> PolarsResult<Series> {
    if labels.len() != columns.len() {
        polars_bail!(
            ComputeError:
            "internal CNLP error: label and result column counts differ"
        );
    }

    let fields = labels
        .iter()
        .zip(columns)
        .map(|(label, values)| Series::new(label.clone().into(), values))
        .collect::<Vec<_>>();

    Ok(StructChunked::from_series("".into(), length, fields.iter())?.into_series())
}
