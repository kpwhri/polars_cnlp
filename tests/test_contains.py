import polars as pl
import polars_cnlp


def test_contains_target():
    df = pl.DataFrame({
        'note_text': [
            'Patient has pneumonia.',
            'Patient has asthma.',
        ],
    })

    result = df.select(
        pl.col('note_text')
        .rdw.contains('pneumonia')
        .alias('result')
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
        .rdw.contains('pneumonia')
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
            .rdw.contains('pneumonia')
            .alias('result')
        )
    )

    result = query.collect()

    assert result['result'].to_list() == [True, False]
