import polars as pl

import polars_cnlp


def test_affirmed_all_with_dict(terms):
    df = pl.DataFrame({'note_text': [
        'Pneumonia and anaphylaxis are present.',
        'No pneumonia. Anaphylaxis present.',
        'Pneumonia present.',
        'No pneumonia.',
        'Asthma present.',
        None,
    ]},
        schema={'note_text': pl.String},
    )

    result = df.select(
        pl.col('note_text').cnlp.affirmed_all(terms).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True, False, None, False, None, None]


def test_affirmed_all_with_list(patterns):
    df = pl.DataFrame({'note_text': [
        'Pneumonia and anaphylaxis are present.',
        'No pneumonia. Anaphylaxis present.',
        'Pneumonia present.',
        'No pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_all(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True, False, None, False]


def test_affirmed_all_false_dominates_missing(patterns):
    df = pl.DataFrame({'note_text': [
        'No pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_all(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_affirmed_all_null_when_one_concept_is_missing(patterns):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_all(patterns).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [None]


def test_affirmed_all_can_filter(patterns):
    df = pl.DataFrame({'note_text': [
        'Pneumonia and anaphylaxis are present.',
        'No pneumonia. Anaphylaxis present.',
        'Pneumonia present.',
        'Asthma present.',
    ]})

    result = df.filter(
        pl.col('note_text').cnlp.affirmed_all(patterns),
    )

    assert result['note_text'].to_list() == [
        'Pneumonia and anaphylaxis are present.',
    ]


def test_affirmed_all_works_lazily(patterns):
    result = (
        pl.DataFrame(
            {'note_text': [
                'Pneumonia and anaphylaxis are present.',
                'No pneumonia. Anaphylaxis present.',
            ]},
        )
        .lazy()
        .select(
            pl.col('note_text').cnlp.affirmed_all(patterns).alias('affirmed'),
        )
        .collect()
    )

    assert result['affirmed'].to_list() == [True, False]
