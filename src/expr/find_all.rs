use polars::chunked_array::builder::get_list_builder;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

use crate::engine::analyze::{Analyzer, IndexedFinding};

use super::findings::{
    FindKwargs, build_finding_struct, compile_find_concepts, find_all_output, finding_dtype,
};

#[derive(Debug, Clone, Copy)]
struct FindingRange {
    offset: usize,
    length: usize,
}

#[polars_expr(output_type_func=find_all_output)]
fn find_all_concepts(inputs: &[Series], kwargs: FindKwargs) -> PolarsResult<Series> {
    let text = inputs[0].str()?;

    let concepts = compile_find_concepts(&kwargs)?;

    let analyzer = Analyzer::compile_default().map_err(|error| {
        polars_err!(
            ComputeError:
            "failed to compile CNLP context rules: {}",
            error
        )
    })?;

    let mut flat_findings: Vec<IndexedFinding> = Vec::new();

    let mut rows: Vec<Option<FindingRange>> = Vec::with_capacity(text.len());

    for text_value in text.iter() {
        match text_value {
            None => {
                rows.push(None);
            }

            Some(text_value) => {
                let offset = flat_findings.len();

                let findings = analyzer.find_all(text_value, &concepts);

                let length = findings.len();

                flat_findings.extend(findings);

                rows.push(Some(FindingRange { offset, length }));
            }
        }
    }

    let rendered = flat_findings
        .iter()
        .copied()
        .map(|finding| kwargs.render(finding))
        .collect::<Vec<_>>();

    let flat_series = build_finding_struct("".into(), &rendered)?;

    let inner_dtype = finding_dtype();

    let mut builder = get_list_builder(
        &inner_dtype,
        flat_findings.len(),
        rows.len(),
        text.name().clone(),
    );

    for row in rows {
        match row {
            None => {
                builder.append_null();
            }

            Some(row) => {
                let offset = i64::try_from(row.offset).map_err(|_| {
                    polars_err!(
                        ComputeError:
                        "finding offset exceeds Int64"
                    )
                })?;

                let findings = flat_series.slice(offset, row.length);

                builder.append_series(&findings)?;
            }
        }
    }

    Ok(builder.finish().into_series())
}
