import polars as pl

import polars_cnlp


def test_find_best_with_single_regex():
    result = pl.DataFrame(
        {'note_text': [
            'No pneumonia.',
            'Asthma present.',
            None,
        ]},
        schema={'note_text': pl.String},
    ).select(
        pl.col('note_text')
        .cnlp.find_best(r'\bpneumonia\b')
        .alias('finding'),
    )

    assert result['finding'].to_list() == [
        {
            'label': None,
            'start': 3,
            'end': 12,
            'assertion': 'negated',
            'temporality': 'current',
            'experiencer': 'patient',
        },
        None,
        None,
    ]


def test_find_best_with_mapping():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'No pneumonia. Anaphylaxis present.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    assert result['finding'].to_list() == [{
        'label': 'anaphylaxis',
        'start': 14,
        'end': 25,
        'assertion': 'affirmed',
        'temporality': 'current',
        'experiencer': 'patient',
    }]


def test_find_best_prefers_affirmed_over_possible_and_negated():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
        'asthma': r'\basthma\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'No pneumonia. Possible anaphylaxis. Asthma present.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['label'] == 'asthma'
    assert finding['assertion'] == 'affirmed'


def test_find_best_prefers_current_over_historical():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'History of pneumonia. Possible anaphylaxis.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['label'] == 'anaphylaxis'
    assert finding['temporality'] == 'current'


def test_find_best_prefers_patient_over_other_experiencer():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'Mother has pneumonia. History of anaphylaxis.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['label'] == 'anaphylaxis'
    assert finding['experiencer'] == 'patient'


def test_find_best_uses_earliest_occurrence_for_equal_context():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'Pneumonia then anaphylaxis.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    assert result['finding'][0]['label'] == 'pneumonia'


def test_find_best_uses_mapping_order_for_identical_span():
    terms = {
        'disease': r'\bpneumonia\b',
        'pneumonia': r'\bpneumonia\b',
    }

    result = pl.DataFrame(
        {'note_text': [
            'Pneumonia present.',
        ]},
    ).select(
        pl.col('note_text').cnlp.find_best(terms).alias('finding'),
    )

    assert result['finding'][0]['label'] == 'disease'


def test_find_best_works_lazily():
    result = (
        pl.DataFrame(
            {'note_text': [
                'No pneumonia.',
                'Pneumonia present.',
            ]},
        )
        .lazy()
        .select(
            pl.col('note_text')
            .cnlp.find_best(r'\bpneumonia\b')
            .alias('finding'),
        )
        .collect()
    )

    assert [
               finding['assertion']
               for finding in result['finding']
           ] == [
               'negated',
               'affirmed',
           ]
