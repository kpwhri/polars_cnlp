use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use serde::Deserialize;

use crate::engine::concept::Concept;
use crate::engine::count;

#[derive(Deserialize)]
struct CountKwargs {
    pattern: String,
}

#[polars_expr(output_type=UInt32)]
fn count_concept(inputs: &[Series], kwargs: CountKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concept = Concept::new(&kwargs.pattern).map_err(|error| {
        polars_err!(
            ComputeError:
            "invalid target regex '{}': {}",
            kwargs.pattern,
            error
        )
    })?;

    let result: UInt32Chunked = text
        .iter()
        .map(|text| text.map(|text| count::count(text, &concept) as u32))
        .collect();
    Ok(result.into_series())
}
