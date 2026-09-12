import polars as pl

import polars_cnlp


def test_count_all(terms):
    df = pl.DataFrame({'note_text': [
        'Pneumonia. Pneumonia.',
        'No pneumonia.',
        'Anaphylaxis and pneumonia.',
        'Asthma.',
        None,
    ]},
        schema={'note_text': pl.String},
    )

    result = df.select(
        pl.col('note_text').cnlp.count_all(terms).alias('counts'),
    )

    assert result['counts'].to_list() == [
        {'pneumonia': 2, 'anaphylaxis': 0},
        {'pneumonia': 1, 'anaphylaxis': 0},
        {'pneumonia': 1, 'anaphylaxis': 1},
        {'pneumonia': 0, 'anaphylaxis': 0},
        {'pneumonia': None, 'anaphylaxis': None},
    ]


def test_count_all_is_case_insensitive():
    terms = {
        'pneumonia': r'\bpneumonia\b',
    }

    result = pl.DataFrame({'note_text': [
        'PNEUMONIA pneumonia Pneumonia.',
    ]},
    ).select(
        pl.col('note_text').cnlp.count_all(terms).alias('counts'),
    )

    assert result['counts'].to_list() == [{'pneumonia': 3}]


def test_count_all_preserves_field_order():
    terms = {
        'anaphylaxis': r'\banaphylaxis\b',
        'pneumonia': r'\bpneumonia\b',
    }

    result = pl.DataFrame(
        {'note_text': ['Pneumonia.']},
    ).select(
        pl.col('note_text').cnlp.count_all(terms).alias('counts'),
    )

    assert [field.name for field in result['counts'].dtype.fields] == ['anaphylaxis', 'pneumonia', ]


def test_count_all_fields_can_be_selected(terms):
    result = (
        pl.DataFrame({'note_text': [
            'Pneumonia. Pneumonia.',
            'Anaphylaxis.',
        ]})
        .with_columns(
            pl.col('note_text').cnlp.count_all(terms).alias('counts'),
        )
        .select(
            pl.col('counts').struct.field('pneumonia'),
        )
    )

    assert result['pneumonia'].to_list() == [2, 0]


def test_count_all_works_lazily(terms):
    result = (
        pl.DataFrame({'note_text': [
            'Pneumonia.',
            'Anaphylaxis and pneumonia.',
        ]})
        .lazy()
        .select(
            pl.col('note_text').cnlp.count_all(terms).alias('counts'),
        )
        .collect()
    )

    assert result['counts'].to_list() == [
        {'pneumonia': 1, 'anaphylaxis': 0},
        {'pneumonia': 1, 'anaphylaxis': 1},
    ]
