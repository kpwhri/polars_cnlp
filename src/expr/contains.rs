use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;

use crate::engine::contains;

#[derive(Deserialize)]
struct ContainsKwargs {
    pattern: String,
}

#[polars_expr(output_type=Boolean)]
fn contains_target(inputs: &[Series], kwargs: ContainsKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let result: BooleanChunked = text
        .iter()
        .map(|text| text.map(|text| contains::contains_target(text, &kwargs.pattern)))
        .collect();

    Ok(result.into_series())
}
