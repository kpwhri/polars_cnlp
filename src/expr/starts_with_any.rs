use aho_corasick::{AhoCorasick, Anchored, Input, StartKind};
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StartsWithAnyKwargs {
    prefixes: Vec<String>,
}

#[polars_expr(output_type=Boolean)]
pub fn starts_with_any(inputs: &[Series], kwargs: StartsWithAnyKwargs) -> PolarsResult<Series> {
    let ca = inputs[0].str()?;

    if kwargs.prefixes.is_empty() {
        let out = BooleanChunked::full(ca.name().clone(), false, ca.len());

        return Ok(out.into_series());
    }

    let matcher = AhoCorasick::builder()
        .start_kind(StartKind::Anchored)
        .build(&kwargs.prefixes)
        .map_err(|err| {
            PolarsError::ComputeError(
                format!("failed to build starts_with_any matcher: {err}").into(),
            )
        })?;

    let out: BooleanChunked = ca
        .iter()
        .map(|value| {
            value.map(|value| {
                matcher
                    .find(Input::new(value).anchored(Anchored::Yes))
                    .is_some()
            })
        })
        .collect();

    Ok(out.into_series())
}
