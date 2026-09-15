use super::algorithm::build_analyzer;
use super::terms::{TermsKwargs, compile_concepts};
use crate::engine::concept::matches_any_concept;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

#[polars_expr(output_type=Boolean)]
fn affirmed_any_concepts(inputs: &[Series], kwargs: TermsKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_concepts(&kwargs.terms)?;

    let analyzer = build_analyzer(&kwargs.algorithm)?;

    let result: BooleanChunked = text
        .iter()
        .map(|text| {
            text.and_then(|text| {
                if kwargs.prefilter && !matches_any_concept(text, &concepts) {
                    return None;
                }

                analyzer.affirmed_any(text, &concepts)
            })
        })
        .collect();

    Ok(result.into_series())
}
