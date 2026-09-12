import polars as pl

import polars_cnlp
from polars_cnlp.algorithms import ConText, NegEx

PNEUMONIA = r'\bpneumonia\b'


def test_context_is_default():
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_context_string_matches_context_object():
    df = pl.DataFrame({'note_text': [
        'Patient has pneumonia.',
        'No pneumonia.',
        'History of pneumonia.',
        None,
    ]},
        schema={
            'note_text': pl.String,
        },
    )

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='context').alias('string'),
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=ConText()).alias('object'),
    )

    assert result['string'].to_list() == result['object'].to_list()


def test_context_recognizes_historical_temporality():
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA).alias('finding'),
    )

    assert result['finding'][0]['temporality'] == 'historical'


def test_context_recognizes_hypothetical_temporality():
    df = pl.DataFrame({'note_text': [
        'Return if pneumonia develops.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA).alias('finding'),
    )

    assert result['finding'][0]['temporality'] == 'hypothetical'


def test_context_recognizes_other_experiencer():
    df = pl.DataFrame({'note_text': [
        'Mother has pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA).alias('finding'),
    )

    assert result['finding'][0]['experiencer'] == 'other'


def test_context_has_no_negex_six_position_limit():
    df = pl.DataFrame({'note_text': [
        'No one two three four five six pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='context').alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_context_presenting_terminates_historical_scope():
    df = pl.DataFrame({'note_text': [
        'History of asthma, presenting with pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA).alias('finding'),
    )

    assert result['finding'][0]['temporality'] == 'current'


def test_negex_does_not_apply_historical_context():
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='negex').alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True]


def test_negex_six_position_window():
    df = pl.DataFrame({'note_text': [
        'No one two three four five pneumonia.',
        'No one two three four five six pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='negex').alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False, True]


def test_negex_supports_post_triggers():
    df = pl.DataFrame({'note_text': [
        'Pneumonia was ruled out.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='negex').alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_negex_supports_possible_triggers():
    df = pl.DataFrame({'note_text': [
        'Rule out pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm='negex').alias('finding'),
    )

    assert result['finding'][0]['assertion'] == 'possible'


def test_negex_custom_window():
    df = pl.DataFrame({'note_text': [
        'No severe pneumonia.',
        'No severe bilateral pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(
            PNEUMONIA,
            algorithm=NegEx(window=2),
        ).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False, True]


def test_algorithm_integrates_with_affirmed_any(patterns):
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(patterns, algorithm='context').alias('context'),
        pl.col('note_text').cnlp.affirmed_any(patterns, algorithm='negex').alias('negex'),
    )

    assert result['context'].to_list() == [False]
    assert result['negex'].to_list() == [True]


def test_algorithm_integrates_with_affirmed_all(patterns):
    df = pl.DataFrame({'note_text': [
        'Pneumonia and anaphylaxis are present.',
        'No pneumonia but anaphylaxis is present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_all(patterns, algorithm='context').alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True, False]


def test_algorithm_integrates_with_affirmed_each(terms):
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_each(terms, algorithm='negex').alias('status'),
    )

    assert result['status'].to_list() == [{
        'pneumonia': True,
        'anaphylaxis': None,
    }]


def test_algorithm_integrates_with_find_best():
    df = pl.DataFrame({'note_text': [
        'History of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm='context').alias('finding'),
    )

    assert result['finding'][0]['temporality'] == 'historical'


def test_algorithm_integrates_with_find_all():
    df = pl.DataFrame({'note_text': [
        'No pneumonia. Pneumonia later developed.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_all(PNEUMONIA, algorithm='context').alias('findings'),
    )

    assert [finding['assertion'] for finding in result['findings'][0]] == [
        'negated',
        'affirmed',
    ]


def test_algorithm_works_lazily():
    df = pl.DataFrame({'note_text': [
        'No pneumonia.',
        'Pneumonia present.',
    ]})

    result = (
        df.lazy()
        .select(
            pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm='negex').alias('affirmed'),
        )
        .collect()
    )

    assert result['affirmed'].to_list() == [False, True]
