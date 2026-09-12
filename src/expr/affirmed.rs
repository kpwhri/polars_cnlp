use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;

use crate::engine::analyze::Analyzer;
use crate::engine::concept::Concept;

#[derive(Deserialize)]
struct AffirmedKwargs {
    pattern: String,
}

#[polars_expr(output_type=Boolean)]
fn affirmed_concept(inputs: &[Series], kwargs: AffirmedKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concept = Concept::new(&kwargs.pattern).map_err(|error| {
        polars_err!(
            ComputeError:
            "invalid concept regex '{}': {}",
            kwargs.pattern,
            error
        )
    })?;

    let analyzer = Analyzer::compile_default().map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to compile CNLP context rules: {}",
            error
        )
    })?;

    let result: BooleanChunked = text
        .iter()
        .map(|text| text.and_then(|text| analyzer.affirmed(text, &concept)))
        .collect();

    Ok(result.into_series())
}
