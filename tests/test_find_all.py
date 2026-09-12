import polars as pl

import polars_cnlp


def test_find_all_with_single_regex():
    df = pl.DataFrame(
        {'note_text': [
            'No pneumonia.',
            'Asthma present.',
            None,
        ]},
        schema={'note_text': pl.String},
    )

    result = df.select(
        pl.col('note_text')
        .cnlp.find_all(r'\bpneumonia\b')
        .alias('findings'),
    )

    assert result['findings'].to_list() == [
        [{
            'label': None,
            'start': 3,
            'end': 12,
            'assertion': 'negated',
            'temporality': 'current',
            'experiencer': 'patient',
        }],
        [],
        None,
    ]


def test_find_all_with_mapping():
    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    result = pl.DataFrame({'note_text': [
        'No pneumonia. Possible anaphylaxis.',
    ]}).select(
        pl.col('note_text').cnlp.find_all(terms).alias('findings'),
    )

    assert result['findings'].to_list() == [
        [
            {
                'label': 'pneumonia',
                'start': 3,
                'end': 12,
                'assertion': 'negated',
                'temporality': 'current',
                'experiencer': 'patient',
            },
            {
                'label': 'anaphylaxis',
                'start': 23,
                'end': 34,
                'assertion': 'possible',
                'temporality': 'current',
                'experiencer': 'patient',
            },
        ],
    ]


def test_find_all_returns_multiple_occurrences_in_text_order():
    result = pl.DataFrame(
        {'note_text': [
            'No pneumonia. Pneumonia later developed.',
        ]},
    ).select(
        pl.col('note_text')
        .cnlp.find_all(r'\bpneumonia\b')
        .alias('findings'),
    )

    findings = result['findings'][0]

    assert len(findings) == 2
    assert findings[0]['assertion'] == 'negated'
    assert findings[1]['assertion'] == 'affirmed'
    assert findings[0]['start'] < findings[1]['start']


def test_find_all_preserves_labels_for_same_span():
    terms = {
        'disease': r'\bpneumonia\b',
        'pneumonia': r'\bpneumonia\b',
    }

    result = pl.DataFrame(
        {'note_text': ['Pneumonia present.']},
    ).select(
        pl.col('note_text').cnlp.find_all(terms).alias('findings'),
    )

    assert [finding['label'] for finding in result['findings'][0]] == [
        'disease',
        'pneumonia',
    ]


def test_find_all_offsets_are_utf8_bytes():
    result = pl.DataFrame(
        {'note_text': ['é pneumonia']},
    ).select(
        pl.col('note_text')
        .cnlp.find_all(r'\bpneumonia\b')
        .alias('findings'),
    )

    finding = result['findings'][0][0]

    assert finding['start'] == 3
    assert finding['end'] == 12


def test_find_all_works_lazily():
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
            .cnlp.find_all(r'\bpneumonia\b')
            .alias('findings'),
        )
        .collect()
    )

    assert result['findings'].list.len().to_list() == [1, 1]
