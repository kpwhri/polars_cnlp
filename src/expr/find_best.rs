use super::algorithm::build_analyzer;
use super::findings::{
    FindKwargs, build_optional_finding_struct, compile_find_concepts, find_best_output,
};
use crate::engine::concept::matches_any_concept;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

#[polars_expr(output_type_func=find_best_output)]
fn find_best_concept(inputs: &[Series], kwargs: FindKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_find_concepts(&kwargs)?;

    let analyzer = build_analyzer(&kwargs.algorithm)?;

    let findings = text
        .iter()
        .map(|text| {
            text.and_then(|text| {
                if kwargs.prefilter && !matches_any_concept(text, &concepts) {
                    return None;
                }

                analyzer
                    .find_best(text, &concepts)
                    .map(|finding| kwargs.render(finding))
            })
        })
        .collect::<Vec<_>>();

    build_optional_finding_struct(text.name().clone(), &findings)
}
