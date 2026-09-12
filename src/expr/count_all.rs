use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

use crate::engine::count;

use super::terms::{TermsKwargs, build_u32_struct, compile_concepts, struct_output_field};

fn count_all_output(input_fields: &[Field], kwargs: TermsKwargs) -> PolarsResult<Field> {
    struct_output_field(input_fields, &kwargs, DataType::UInt32)
}

#[polars_expr(output_type_func_with_kwargs=count_all_output)]
fn count_all_concepts(inputs: &[Series], kwargs: TermsKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_concepts(&kwargs)?;

    let labels = kwargs.labels();

    let mut columns: Vec<Vec<Option<u32>>> = (0..concepts.len())
        .map(|_| Vec::with_capacity(text.len()))
        .collect();

    for text_value in text.iter() {
        match text_value {
            Some(text_value) => {
                let counts = count::count_all(text_value, &concepts);

                for (index, (column, count)) in columns.iter_mut().zip(counts).enumerate() {
                    let count = u32::try_from(count).map_err(|_| {
                        polars_err!(
                            ComputeError:
                            "match count for term '{}' exceeds UInt32",
                            kwargs.terms[index].label
                        )
                    })?;

                    column.push(Some(count));
                }
            }

            None => {
                for column in &mut columns {
                    column.push(None);
                }
            }
        }
    }

    build_u32_struct(&labels, columns, text.len())
}
