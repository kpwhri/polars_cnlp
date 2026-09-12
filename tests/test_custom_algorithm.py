import polars as pl

import polars_cnlp
from polars_cnlp.algorithms import (
    ConText,
    ContextEffect,
    ContextRule,
    NegEx,
    PseudoRule,
    RuleBased,
    RuleSet,
    TerminateRule,
)

PNEUMONIA = r'\bpneumonia\b'


def test_context_can_replace_default_rules():
    algorithm = ConText(
        rules=RuleSet([
            ContextRule(
                pattern=r'\babsent\b',
                direction='forward',
                effect=ContextEffect(assertion='negated'),
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Absent pneumonia.',
        'No pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False, True]


def test_same_rules_can_use_context_or_negex_mechanics():
    rules = RuleSet([
        ContextRule(
            pattern=r'\babsent\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ])

    df = pl.DataFrame({'note_text': [
        'Absent one two three four five six pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA, algorithm=ConText(rules=rules))
        .alias('context'),
        pl.col('note_text')
        .cnlp.affirmed(PNEUMONIA, algorithm=NegEx(rules=rules))
        .alias('negex'),
    )

    assert result['context'].to_list() == [False]
    assert result['negex'].to_list() == [True]


def test_custom_rule_can_change_multiple_dimensions():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bfamily\W+record\W+of\b',
                direction='forward',
                effect=ContextEffect(
                    temporality='historical',
                    experiencer='other',
                ),
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Family record of pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm=algorithm).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['assertion'] == 'affirmed'
    assert finding['temporality'] == 'historical'
    assert finding['experiencer'] == 'other'


def test_custom_rule_supports_backward_direction():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bexcluded\b',
                direction='backward',
                effect=ContextEffect(assertion='negated'),
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Pneumonia excluded.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False]


def test_custom_rule_supports_bidirectional_direction():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bunlikely\b',
                direction='bidirectional',
                effect=ContextEffect(assertion='possible'),
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Unlikely pneumonia.',
        'Pneumonia unlikely.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm=algorithm).alias('finding'),
    )

    assert [finding['assertion'] for finding in result['finding']] == [
        'possible',
        'possible',
    ]


def test_custom_algorithm_supports_terminators():
    negated = ContextEffect(assertion='negated')

    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\babsent\b',
                direction='forward',
                effect=negated,
            ),
            TerminateRule(
                pattern=r'\bbut\b',
                effects=[negated],
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Absent fever but pneumonia present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True]


def test_custom_algorithm_supports_selective_terminators():
    historical = ContextEffect(temporality='historical')
    other = ContextEffect(experiencer='other')

    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bhistory\W+of\b',
                direction='forward',
                effect=historical,
            ),
            ContextRule(
                pattern=r'\bfamily\b',
                direction='forward',
                effect=other,
            ),
            TerminateRule(
                pattern=r'\bpresenting\b',
                effects=[historical],
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Family history of asthma presenting with pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm=algorithm).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['temporality'] == 'current'
    assert finding['experiencer'] == 'other'


def test_custom_algorithm_supports_pseudo_rules():
    negated = ContextEffect(assertion='negated')

    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bnegative\b',
                direction='forward',
                effect=negated,
            ),
            PseudoRule(
                pattern=r'\bnegative\W+attitude\b',
                effects=[negated],
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Negative attitude regarding pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [True]


def test_custom_algorithm_supports_selective_pseudo_rules():
    negated = ContextEffect(assertion='negated')
    historical = ContextEffect(temporality='historical')

    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bno\b',
                direction='forward',
                effect=negated,
            ),
            ContextRule(
                pattern=r'\bno\b',
                direction='forward',
                effect=historical,
            ),
            PseudoRule(
                pattern=r'\bno\W+increase\b',
                effects=[negated],
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'No increase in pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_best(PNEUMONIA, algorithm=algorithm).alias('finding'),
    )

    finding = result['finding'][0]

    assert finding['assertion'] == 'affirmed'
    assert finding['temporality'] == 'historical'


def test_custom_algorithm_supports_max_scope():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\babsent\b',
                direction='forward',
                effect=ContextEffect(assertion='negated'),
                max_scope=1,
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Absent severe pneumonia.',
        'Absent very severe pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False, True]


def test_custom_algorithm_supports_max_targets():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\bremote\b',
                direction='forward',
                effect=ContextEffect(temporality='historical'),
                max_targets=1,
            ),
        ]),
    )

    terms = {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }

    df = pl.DataFrame({'note_text': [
        'Remote pneumonia and anaphylaxis.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.find_all(terms, algorithm=algorithm).alias('findings'),
    )

    findings = result['findings'][0]

    assert findings[0]['temporality'] == 'historical'
    assert findings[1]['temporality'] == 'current'


def test_custom_algorithm_supports_fixed_window():
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\babsent\b',
                direction='forward',
                effect=ContextEffect(assertion='negated'),
            ),
        ]),
        window=2,
    )

    df = pl.DataFrame({'note_text': [
        'Absent severe pneumonia.',
        'Absent very severe pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(PNEUMONIA, algorithm=algorithm).alias('affirmed'),
    )

    assert result['affirmed'].to_list() == [False, True]


def test_custom_algorithm_integrates_with_multi_term_calls(terms):
    algorithm = RuleBased(
        rules=RuleSet([
            ContextRule(
                pattern=r'\babsent\b',
                direction='forward',
                effect=ContextEffect(assertion='negated'),
            ),
        ]),
    )

    df = pl.DataFrame({'note_text': [
        'Absent pneumonia. Anaphylaxis present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_any(terms, algorithm=algorithm).alias('any'),
        pl.col('note_text').cnlp.affirmed_all(terms, algorithm=algorithm).alias('all'),
        pl.col('note_text').cnlp.affirmed_each(terms, algorithm=algorithm).alias('each'),
    )

    assert result['any'].to_list() == [True]
    assert result['all'].to_list() == [False]
    assert result['each'].to_list() == [{
        'pneumonia': False,
        'anaphylaxis': True,
    }]
