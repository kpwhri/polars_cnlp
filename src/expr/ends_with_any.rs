use crate::rdw_engine::ends_with_any::SuffixMatcher;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EndsWithAnyKwargs {
    prefixes: Vec<String>,
}

#[polars_expr(output_type=Boolean)]
pub fn ends_with_any(inputs: &[Series], kwargs: EndsWithAnyKwargs) -> PolarsResult<Series> {
    let ca = inputs[0].str()?;

    let matcher = SuffixMatcher::new(kwargs.prefixes);

    let out: BooleanChunked = ca
        .iter()
        .map(|value| value.map(|value| matcher.is_match(value)))
        .collect();

    Ok(out.into_series())
}
