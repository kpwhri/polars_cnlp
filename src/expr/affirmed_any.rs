use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

use crate::engine::analyze::Analyzer;

use super::terms::{TermsKwargs, compile_concepts};

#[polars_expr(output_type=Boolean)]
fn affirmed_any_concepts(inputs: &[Series], kwargs: TermsKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_concepts(&kwargs)?;

    let analyzer = Analyzer::compile_default().map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to compile CNLP context rules: {}",
            error
        )
    })?;

    let result: BooleanChunked = text
        .iter()
        .map(|text| text.and_then(|text| analyzer.affirmed_any(text, &concepts)))
        .collect();

    Ok(result.into_series())
}
