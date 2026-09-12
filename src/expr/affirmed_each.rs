use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

use crate::engine::analyze::Analyzer;

use super::terms::{TermsKwargs, build_boolean_struct, compile_concepts, struct_output_field};

fn affirmed_each_output(input_fields: &[Field], kwargs: TermsKwargs) -> PolarsResult<Field> {
    struct_output_field(input_fields, &kwargs, DataType::Boolean)
}

#[polars_expr(output_type_func_with_kwargs=affirmed_each_output)]
fn affirmed_each_concepts(inputs: &[Series], kwargs: TermsKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_concepts(&kwargs)?;

    let analyzer = Analyzer::compile_default().map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to compile CNLP context rules: {}",
            error
        )
    })?;

    let labels = kwargs.labels();

    let mut columns: Vec<Vec<Option<bool>>> = (0..concepts.len())
        .map(|_| Vec::with_capacity(text.len()))
        .collect();

    for text_value in text.iter() {
        match text_value {
            Some(text_value) => {
                let values = analyzer.affirmed_each(text_value, &concepts);

                for (column, value) in columns.iter_mut().zip(values) {
                    column.push(value);
                }
            }

            None => {
                for column in &mut columns {
                    column.push(None);
                }
            }
        }
    }

    build_boolean_struct(&labels, columns, text.len())
}
