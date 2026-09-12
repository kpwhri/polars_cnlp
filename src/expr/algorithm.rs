use polars::prelude::*;

use crate::engine::algorithm::AlgorithmSpec;
use crate::engine::analyze::Analyzer;

/// Build an analyzer and convert algorithm errors into Polars errors.
pub fn build_analyzer(spec: &AlgorithmSpec) -> PolarsResult<Analyzer> {
    Analyzer::from_spec(spec).map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to build CNLP algorithm: {}",
            error
        )
    })
}
