import polars as pl
import polars_cnlp


def test_count_no_mentions() -> None:
    df = pl.DataFrame({
        'note_text': ['Patient has asthma.'],
    })

    result = df.select(
        pl.col('note_text')
        .cnlp.count(r'\bpneumonia\b')
        .alias('count')
    )

    assert result['count'].to_list() == [0]


def test_count_one_mention() -> None:
    df = pl.DataFrame({
        'note_text': ['Patient has pneumonia.'],
    })

    result = df.select(
        pl.col('note_text')
        .cnlp.count(r'\bpneumonia\b')
        .alias('count')
    )

    assert result['count'].to_list() == [1]


def test_count_multiple_mentions() -> None:
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


def test_count_preserves_null() -> None:
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
