import polars as pl
import pytest

import polars_cnlp
from polars_cnlp import cnlp


def namespace_affirmed_any(expr: pl.Expr, terms: dict) -> pl.Expr:
    return expr.cnlp.affirmed_any(terms)


def function_affirmed_any(expr: pl.Expr, terms: dict) -> pl.Expr:
    return cnlp.affirmed_any(expr, terms)


@pytest.mark.parametrize('func', [
    namespace_affirmed_any,
    function_affirmed_any,
])
def test_affirmed_any_with_dict(terms, func):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'No pneumonia. Anaphylaxis present.',
        'Possible anaphylaxis.',
        'Asthma present.',
        None,
    ]},
        schema={'note_text': pl.String},
    )

    result = df.select(
        func(pl.col('note_text'), terms).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True, False, True, False, None, None]


def test_affirmed_any_with_list(patterns):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'No pneumonia. Anaphylaxis present.',
        'Possible anaphylaxis.',
        'Asthma present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True, False, True, False, None]


def test_affirmed_any_true_when_one_concept_is_affirmed(patterns):
    df = pl.DataFrame({'note_text': [
        'No pneumonia. Anaphylaxis present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True]


def test_affirmed_any_false_when_mentions_are_nonaffirmed(patterns):
    df = pl.DataFrame({'note_text': [
        'No pneumonia. Possible anaphylaxis.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_affirmed_any_null_when_no_concepts_are_mentioned(patterns):
    df = pl.DataFrame({'note_text': [
        'Asthma present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [None]


def test_affirmed_any_works_lazily(patterns):
    result = (
        pl.DataFrame({'note_text': [
            'No pneumonia.',
            'Anaphylaxis present.',
        ]})
        .lazy()
        .select(
            pl.col('note_text').cnlp.affirmed_any(patterns).alias('affirmed'),
        )
        .collect()
    )

    assert result['affirmed'].to_list() == [False, True]
