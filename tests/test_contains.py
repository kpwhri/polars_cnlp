import polars as pl
import pytest

import polars_cnlp
from polars_cnlp import cnlp


def namespace_contains(expr: pl.Expr, pattern: str) -> pl.Expr:
    return expr.cnlp.contains(pattern)


def function_contains(expr: pl.Expr, pattern: str) -> pl.Expr:
    return cnlp.contains(expr, pattern)


@pytest.mark.parametrize('func', [
    namespace_contains,
    function_contains,
])
def test_contains_target(func):
    df = pl.DataFrame({
        'note_text': [
            'Patient has pneumonia.',
            'Patient has asthma.',
        ],
    })

    result = df.select(
        func(pl.col('note_text'), 'pneumonia').alias('result'),
    )

    assert result['result'].to_list() == [True, False]


def test_contains_preserves_null():
    df = pl.DataFrame({
        'note_text': [
            'Patient has pneumonia.',
            None,
        ],
    })

    result = df.select(
        pl.col('note_text')
        .cnlp.contains('pneumonia')
        .alias('result')
    )

    assert result['result'].to_list() == [True, None]


def test_contains_lazy():
    query = (
        pl.DataFrame({
            'note_text': [
                'Patient has pneumonia.',
                'Patient has asthma.',
            ],
        })
        .lazy()
        .with_columns(
            pl.col('note_text')
            .cnlp.contains('pneumonia')
            .alias('result')
        )
    )

    result = query.collect()

    assert result['result'].to_list() == [True, False]
