import polars as pl
import pytest

import polars_cnlp

PNEUMONIA = r'\bpneumonia\b'


@pytest.mark.parametrize('algorithm', [
    'context',
    'negex',
])
def test_affirmed_prefilter_matches_normal_processing(algorithm):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'Possible pneumonia.',
        'Patient has asthma.',
        '',
        None,
    ]})

    normal = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm),
    )

    prefiltered = df.select(
        pl.col('note_text').cnlp.affirmed(
            PNEUMONIA,
            algorithm=algorithm,
            prefilter=True,
        ),
    )

    assert normal.equals(prefiltered)


@pytest.mark.parametrize('method', [
    'affirmed_any',
    'affirmed_all',
])
@pytest.mark.parametrize('algorithm', [
    'context',
    'negex',
])
def test_predicate_prefilter_matches_normal_processing(patterns, method, algorithm):
    df = pl.DataFrame({'note_text': [
        'Pneumonia and anaphylaxis present.',
        'Pneumonia present.',
        'No anaphylaxis.',
        'Possible pneumonia.',
        'Patient has asthma.',
        '',
        None,
    ]})

    expr = getattr(pl.col('note_text').cnlp, method)

    normal = df.select(
        expr(patterns, algorithm=algorithm),
    )

    prefiltered = df.select(
        expr(
            patterns,
            algorithm=algorithm,
            prefilter=True,
        ),
    )

    assert normal.equals(prefiltered)


@pytest.mark.parametrize('algorithm', [
    'context',
    'negex',
])
def test_affirmed_each_prefilter_matches_normal_processing(terms, algorithm):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'Possible anaphylaxis.',
        'Patient has asthma.',
        '',
        None,
    ]})

    normal = df.select(
        pl.col('note_text').cnlp.affirmed_each(
            terms,
            algorithm=algorithm,
        ),
    )

    prefiltered = df.select(
        pl.col('note_text').cnlp.affirmed_each(
            terms,
            algorithm=algorithm,
            prefilter=True,
        ),
    )

    assert normal.equals(prefiltered)


@pytest.mark.parametrize('algorithm', [
    'context',
    'negex',
])
def test_find_best_prefilter_matches_normal_processing(terms, algorithm):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'Possible anaphylaxis.',
        'Patient has asthma.',
        '',
        None,
    ]})

    normal = df.select(
        pl.col('note_text').cnlp.find_best(
            terms,
            algorithm=algorithm,
        ),
    )

    prefiltered = df.select(
        pl.col('note_text').cnlp.find_best(
            terms,
            algorithm=algorithm,
            prefilter=True,
        ),
    )

    assert normal.equals(prefiltered)


@pytest.mark.parametrize('algorithm', [
    'context',
    'negex',
])
def test_find_all_prefilter_matches_normal_processing(terms, algorithm):
    df = pl.DataFrame({'note_text': [
        'Pneumonia present.',
        'No pneumonia.',
        'Possible anaphylaxis.',
        'Patient has asthma.',
        '',
        None,
    ]})

    normal = df.select(
        pl.col('note_text').cnlp.find_all(
            terms,
            algorithm=algorithm,
        ),
    )

    prefiltered = df.select(
        pl.col('note_text').cnlp.find_all(
            terms,
            algorithm=algorithm,
            prefilter=True,
        ),
    )

    assert normal.equals(prefiltered)


def test_affirmed_prefilter_no_match():
    df = pl.DataFrame({'note_text': [
        'Patient has asthma.',
        None,
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(
            PNEUMONIA,
            prefilter=True,
        ).alias('result'),
    )

    assert result['result'].to_list() == [None, None]


@pytest.mark.parametrize('method', [
    'affirmed_any',
    'affirmed_all',
])
def test_predicate_prefilter_no_match(patterns, method):
    df = pl.DataFrame({'note_text': [
        'Patient has asthma.',
        None,
    ]})

    expr = getattr(pl.col('note_text').cnlp, method)

    result = df.select(
        expr(patterns, prefilter=True).alias('result'),
    )

    assert result['result'].to_list() == [None, None]


def test_affirmed_each_prefilter_no_match(terms):
    df = pl.DataFrame({'note_text': [
        'Patient has asthma.',
        None,
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_each(
            terms,
            prefilter=True,
        ).alias('status'),
    )

    assert result['status'].to_list() == [
        {'pneumonia': None, 'anaphylaxis': None},
        {'pneumonia': None, 'anaphylaxis': None},
    ]


def test_find_best_prefilter_no_match(terms):
    df = pl.DataFrame({'note_text': [
        'Patient has asthma.',
        None,
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(
            terms,
            prefilter=True,
        ).alias('finding'),
    )

    assert result['finding'].to_list() == [None, None]


def test_find_all_prefilter_no_match(terms):
    df = pl.DataFrame({'note_text': [
        'Patient has asthma.',
        'Nothing relevant.',
        'Pneumonia present.',
        None,
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_all(
            terms,
            prefilter=True,
        ).alias('findings'),
    )

    findings = result['findings'].to_list()

    assert findings[0] == []
    assert findings[1] == []
    assert len(findings[2]) == 1
    assert findings[2][0]['label'] == 'pneumonia'
    assert findings[3] is None
