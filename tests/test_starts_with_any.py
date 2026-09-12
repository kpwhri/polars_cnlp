import polars as pl
import pytest


@pytest.mark.parametrize('value, patterns, expected', [
    ('abc', ['ab', 'cd'], True),
    ('cdef', ['ab', 'cd'], True),
    ('xyz', ['ab', 'cd'], False),
    ('ab', ['ab', 'cd'], True),
    ('a', ['ab', 'cd'], False),
    ('', ['ab', 'cd'], False),
    ('ABC', ['ab', 'cd'], False),
    ('a.b', ['a.'], True),
    ('axb', ['a.'], False),  # literal, not regex
    ('abcdef', ['abc', 'abcdef'], True),
    ('abcdef', ['abcdef', 'abc'], True),
    ('abc', [], False),
    ('abc', [''], True),
    ('', [''], True),
    (None, ['ab', 'cd'], None),
])
def test_starts_with_any(value, patterns, expected):
    result = (
        pl.DataFrame(
            {'value': [value]},
            schema={'value': pl.String},
        )
        .select(
            pl.col('value')
            .rdw.starts_with_any(patterns)
            .alias('result')
        )
        .item()
    )

    assert result == expected


@pytest.mark.parametrize('patterns', [
    ['ab', 'cd'],
    ('ab', 'cd'),
    {'ab', 'cd'},
])
def test_starts_with_any_accepts_collections(patterns):
    result = (
        pl.DataFrame({'value': ['abc']})
        .select(pl.col('value').rdw.starts_with_any(patterns))
        .item()
    )

    assert result is True


def test_starts_with_any_vectorized():
    df = pl.DataFrame({'value': [
        'abc', 'cdef', 'xyz', '', None,
    ]})

    result = df.select(
        pl.col('value').rdw.starts_with_any(['ab', 'cd'])
    ).to_series()

    assert result.to_list() == [True, True, False, False, None]


def test_starts_with_any_preserves_null():
    df = pl.DataFrame({
        'value': ['abc', None, 'xyz'],
    })

    result = df.select(
        pl.col('value').rdw.starts_with_any(['ab'])
    )

    assert result.to_series().to_list() == [True, None, False, ]
