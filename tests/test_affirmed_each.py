import polars as pl
import pytest

import polars_cnlp
from polars_cnlp import cnlp


def namespace_affirmed_each(expr: pl.Expr, terms: dict) -> pl.Expr:
    return expr.cnlp.affirmed_each(terms)


def function_affirmed_each(expr: pl.Expr, terms: dict) -> pl.Expr:
    return cnlp.affirmed_each(expr, terms)


@pytest.mark.parametrize('func', [
    namespace_affirmed_each,
    function_affirmed_each,
])
def test_affirmed_each(terms, func):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia. Anaphylaxis present.',
        'Possible anaphylaxis.',
        'Asthma present.',
        None,
    ]},
        schema={'note_text': pl.String},
    )

    result = df.select(
        func(pl.col('note_text'), terms).alias('status'),
    )

    assert result['status'].to_list() == [
        {'pneumonia': True, 'anaphylaxis': None},
        {'pneumonia': False, 'anaphylaxis': True},
        {'pneumonia': None, 'anaphylaxis': False},
        {'pneumonia': None, 'anaphylaxis': None},
        {'pneumonia': None, 'anaphylaxis': None},
    ]


def test_affirmed_each_preserves_field_order():
    terms = {
        'anaphylaxis': r'\banaphylaxis\b',
        'pneumonia': r'\bpneumonia\b',
    }

    result = pl.DataFrame(
        {'note_text': ['Pneumonia present.']},
    ).select(
        pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
    )

    assert [field.name for field in result['status'].dtype.fields] == ['anaphylaxis', 'pneumonia']


def test_affirmed_each_fields_can_be_selected(terms):
    result = (
        pl.DataFrame({'note_text': [
            'Pneumonia present.',
            'No pneumonia.',
        ]})
        .with_columns(
            pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
        )
        .select(
            pl.col('status').struct.field('pneumonia'),
        )
    )

    assert result['pneumonia'].to_list() == [True, False]


def test_affirmed_each_works_lazily(terms):
    result = (
        pl.DataFrame({'note_text': [
            'Pneumonia present.',
            'No pneumonia. Anaphylaxis present.',
        ]})
        .lazy()
        .select(
            pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
        )
        .collect()
    )

    assert result['status'].to_list() == [
        {'pneumonia': True, 'anaphylaxis': None},
        {'pneumonia': False, 'anaphylaxis': True},
    ]
