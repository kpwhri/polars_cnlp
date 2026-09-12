use std::collections::HashSet;

use polars::prelude::*;
use serde::Deserialize;

use crate::engine::concept::Concept;

#[derive(Debug, Clone, Deserialize)]
pub struct TermSpec {
    pub label: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TermsKwargs {
    pub terms: Vec<TermSpec>,
}

impl TermsKwargs {
    pub fn validate(&self) -> PolarsResult<()> {
        if self.terms.is_empty() {
            polars_bail!(
                ComputeError:
                "terms must contain at least one regular expression"
            );
        }

        let mut labels = HashSet::with_capacity(self.terms.len());

        for term in &self.terms {
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

    pub fn labels(&self) -> Vec<String> {
        self.terms.iter().map(|term| term.label.clone()).collect()
    }
}

pub fn compile_concepts(kwargs: &TermsKwargs) -> PolarsResult<Vec<Concept>> {
    kwargs.validate()?;

    kwargs
        .terms
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

pub fn struct_output_field(
    input_fields: &[Field],
    kwargs: &TermsKwargs,
    inner_dtype: DataType,
) -> PolarsResult<Field> {
    kwargs.validate()?;

    let input_field = input_fields.first().ok_or_else(|| {
        polars_err!(
            ComputeError:
            "CNLP expression requires one input field"
        )
    })?;

    let fields = kwargs
        .terms
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

    let fields: Vec<Series> = labels
        .iter()
        .zip(columns)
        .map(|(label, values)| Series::new(label.clone().into(), values))
        .collect();

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

    let fields: Vec<Series> = labels
        .iter()
        .zip(columns)
        .map(|(label, values)| Series::new(label.clone().into(), values))
        .collect();

    Ok(StructChunked::from_series("".into(), length, fields.iter())?.into_series())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kwargs() -> TermsKwargs {
        TermsKwargs {
            terms: vec![
                TermSpec {
                    label: "pneumonia".to_string(),
                    pattern: r"\bpneumonia\b".to_string(),
                },
                TermSpec {
                    label: "anaphylaxis".to_string(),
                    pattern: r"\banaphylaxis\b".to_string(),
                },
            ],
        }
    }

    #[test]
    fn validates_terms() {
        assert!(kwargs().validate().is_ok());
    }

    #[test]
    fn rejects_empty_terms() {
        assert!(TermsKwargs { terms: Vec::new() }.validate().is_err());
    }

    #[test]
    fn rejects_empty_label() {
        let kwargs = TermsKwargs {
            terms: vec![TermSpec {
                label: String::new(),
                pattern: r"\bpneumonia\b".to_string(),
            }],
        };

        assert!(kwargs.validate().is_err());
    }

    #[test]
    fn rejects_empty_pattern() {
        let kwargs = TermsKwargs {
            terms: vec![TermSpec {
                label: "pneumonia".to_string(),
                pattern: String::new(),
            }],
        };

        assert!(kwargs.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_labels() {
        let kwargs = TermsKwargs {
            terms: vec![
                TermSpec {
                    label: "disease".to_string(),
                    pattern: r"\bpneumonia\b".to_string(),
                },
                TermSpec {
                    label: "disease".to_string(),
                    pattern: r"\basthma\b".to_string(),
                },
            ],
        };

        assert!(kwargs.validate().is_err());
    }

    #[test]
    fn compiles_concepts_in_term_order() {
        let concepts = compile_concepts(&kwargs()).unwrap();

        assert_eq!(concepts[0].count("Pneumonia."), 1,);

        assert_eq!(concepts[1].count("Pneumonia."), 0,);
    }

    #[test]
    fn invalid_regex_reports_error() {
        let kwargs = TermsKwargs {
            terms: vec![TermSpec {
                label: "bad_pattern".to_string(),
                pattern: r"[invalid".to_string(),
            }],
        };

        assert!(compile_concepts(&kwargs).is_err());
    }

    #[test]
    fn boolean_struct_schema_uses_labels() {
        let input = vec![Field::new("note_text".into(), DataType::String)];

        let output = struct_output_field(&input, &kwargs(), DataType::Boolean).unwrap();

        assert_eq!(
            output.dtype(),
            &DataType::Struct(vec![
                Field::new("pneumonia".into(), DataType::Boolean,),
                Field::new("anaphylaxis".into(), DataType::Boolean,),
            ],),
        );
    }

    #[test]
    fn u32_struct_schema_uses_labels() {
        let input = vec![Field::new("note_text".into(), DataType::String)];

        let output = struct_output_field(&input, &kwargs(), DataType::UInt32).unwrap();

        assert_eq!(
            output.dtype(),
            &DataType::Struct(vec![
                Field::new("pneumonia".into(), DataType::UInt32,),
                Field::new("anaphylaxis".into(), DataType::UInt32,),
            ],),
        );
    }

    #[test]
    fn builds_boolean_struct() {
        let labels = vec!["pneumonia".to_string(), "anaphylaxis".to_string()];

        let series = build_boolean_struct(
            &labels,
            vec![
                vec![Some(true), Some(false), None],
                vec![None, Some(true), None],
            ],
            3,
        )
        .unwrap();

        let pneumonia = series
            .struct_()
            .unwrap()
            .field_by_name("pneumonia")
            .unwrap();

        assert_eq!(
            pneumonia.bool().unwrap().iter().collect::<Vec<_>>(),
            vec![Some(true), Some(false), None,],
        );
    }

    #[test]
    fn builds_u32_struct() {
        let labels = vec!["pneumonia".to_string(), "anaphylaxis".to_string()];

        let series = build_u32_struct(
            &labels,
            vec![vec![Some(2), Some(0), None], vec![Some(0), Some(1), None]],
            3,
        )
        .unwrap();

        let pneumonia = series
            .struct_()
            .unwrap()
            .field_by_name("pneumonia")
            .unwrap();

        assert_eq!(
            pneumonia.u32().unwrap().iter().collect::<Vec<_>>(),
            vec![Some(2), Some(0), None,],
        );
    }
}
