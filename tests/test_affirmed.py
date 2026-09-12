import polars as pl
import polars_cnlp

PNEUMONIA = r'\bpneumonia\b'


def test_affirmed_truth_table():
    df = pl.DataFrame({'note_text': [
        'Patient has pneumonia.',
        'No pneumonia.',
        'Possible pneumonia.',
        'History of pneumonia.',
        'Family history of pneumonia.',
        'Patient has asthma.',
        None,
    ]},
        schema={
            'note_text': pl.String,
        },
    )

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA).alias('pneumonia'),
    )

    assert result['pneumonia'].to_list() == [True, False, False, False, False, None, None]


def test_affirmed_respects_boundaries():
    df = pl.DataFrame({'note_text': [
        'No fever. Pneumonia present.',
        'No fever but pneumonia present.',
    ]})

    result = df.select(
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA)
        .alias('pneumonia')
    )

    assert result['pneumonia'].to_list() == [True, True]


def test_affirmed_respects_pseudo_rules():
    df = pl.DataFrame({'note_text': [
        'Not only pneumonia was documented.',
    ]})

    result = df.select(
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA)
        .alias('pneumonia')
    )

    assert result['pneumonia'].to_list() == [True]


def test_affirmed_handles_competing_assertion_rules():
    df = pl.DataFrame({'note_text': [
        'No possible pneumonia.',
        'Possible no pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA)
        .alias('pneumonia')
    )

    assert result['pneumonia'].to_list() == [False, False]


def test_affirmed_any_positive_occurrence_wins():
    df = pl.DataFrame({'note_text': [
        'No pneumonia initially. Pneumonia developed later.',
    ]})

    result = df.select(
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA)
        .alias('pneumonia')
    )

    assert result['pneumonia'].to_list() == [True]


def test_affirmed_works_lazily():
    result = (
        pl.DataFrame({'note_text': [
            'No pneumonia.',
            'Pneumonia present.',
        ]})
        .lazy()
        .select(
            pl.col('note_text')
            .cnlp.affirmed(PNEUMONIA)
            .alias('pneumonia')
        )
        .collect()
    )

    assert result['pneumonia'].to_list() == [False, True]
