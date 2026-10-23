import polars as pl
import pytest

import polars_cnlp
from polars_cnlp import cnlp


def namespace_count(expr: pl.Expr, terms: dict) -> pl.Expr:
    return expr.cnlp.count(terms)


def function_count(expr: pl.Expr, terms: dict) -> pl.Expr:
    return cnlp.count(expr, terms)


@pytest.mark.parametrize('func', [
    namespace_count,
    function_count,
])
def test_count_no_mentions(func):
    df = pl.DataFrame({
        'note_text': ['Patient has asthma.'],
    })

    result = df.select(
        func(pl.col('note_text'), r'\bpneumonia\b').alias('count'),
    )

    assert result['count'].to_list() == [0]


def test_count_one_mention():
    df = pl.DataFrame({
        'note_text': ['Patient has pneumonia.'],
    })

    result = df.select(
        pl.col('note_text')
        .cnlp.count(r'\bpneumonia\b')
        .alias('count')
    )

    assert result['count'].to_list() == [1]


def test_count_multiple_mentions():
    df = pl.DataFrame({
        'note_text': [
            'Pneumonia suspected, and pneumonia confirmed.',
        ],
    })

    result = df.select(
        pl.col('note_text')
        .cnlp.count(r'\bpneumonia\b')
        .alias('count')
    )

    assert result['count'].to_list() == [2]


def test_count_preserves_null():
    df = pl.DataFrame(
        {
            'note_text': [None],
        },
        schema={'note_text': pl.String},
    )

    result = df.select(
        pl.col('note_text')
        .cnlp.count(r'\bpneumonia\b')
        .alias('count')
    )

    assert result['count'].to_list() == [None]
