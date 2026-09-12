import polars as pl
import pytest


@pytest.mark.parametrize('value, patterns, expected', [
    ('testing', ['ing', 'ed'], True),
    ('tested', ['ing', 'ed'], True),
    ('test', ['ing', 'ed'], False),
    ('ing', ['ing', 'ed'], True),
    ('g', ['ing', 'ed'], False),
    ('', ['ing', 'ed'], False),
    ('TESTING', ['ing'], False),
    ('a.b', ['.b'], True),
    ('axb', ['.b'], False),
    ('testing', ['ing', 'testing'], True),
    ('test', [], False),
    ('test', [''], True),
    ('', [''], True),
    (None, ['ing', 'ed'], None),
])
def test_ends_with_any(value, patterns, expected):
    result = (
        pl.DataFrame(
            {'value': [value]},
            schema={'value': pl.String},
        )
        .select(
            pl.col('value').rdw.ends_with_any(patterns)
        )
        .item()
    )

    assert result == expected


@pytest.mark.parametrize('patterns, expected', [
    (['ing', 'ed'], ['running', 'walked']),
    (['ly'], ['quickly']),
    (['g'], ['running']),
    ([], []),
    ([''], ['running', 'walked', 'quickly', 'run']),
])
def test_ends_with_any_filter(patterns, expected):
    df = pl.DataFrame({
        'value': ['running', 'walked', 'quickly', 'run', None],
    })

    result = df.filter(
        pl.col('value').rdw.ends_with_any(patterns)
    )

    assert result['value'].to_list() == expected


@pytest.mark.parametrize('patterns', [
    ['ing', 'ed'],
    ('ing', 'ed'),
    {'ing', 'ed'},
])
def test_ends_with_any_accepts_collections(patterns):
    result = (
        pl.DataFrame({'value': ['testing']})
        .select(
            pl.col('value')
            .rdw.ends_with_any(patterns)
        )
        .item()
    )

    assert result is True


@pytest.mark.parametrize('value, expected', [
    ('thing', True),
    ('something', True),
    ('king', True),
    ('ing', True),
    ('in', False),
    ('thingy', False),
])
def test_ends_with_any_overlapping_suffixes(value, expected):
    result = (
        pl.DataFrame({'value': [value]})
        .select(
            pl.col('value').rdw.ends_with_any(
                ['ing', 'thing', 'something']
            )
        )
        .item()
    )

    assert result == expected
