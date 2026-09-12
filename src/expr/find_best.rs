use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

use crate::engine::analyze::Analyzer;

use super::findings::{
    FindKwargs, build_optional_finding_struct, compile_find_concepts, find_best_output,
};

#[polars_expr(output_type_func=find_best_output)]
fn find_best_concept(inputs: &[Series], kwargs: FindKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_find_concepts(&kwargs)?;

    let analyzer = Analyzer::compile_default().map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to compile CNLP context rules: {}",
            error
        )
    })?;

    let findings = text
        .iter()
        .map(|text| {
            text.and_then(|text| {
                analyzer
                    .find_best(text, &concepts)
                    .map(|finding| kwargs.render(finding))
            })
        })
        .collect::<Vec<_>>();

    build_optional_finding_struct(text.name().clone(), &findings)
}
