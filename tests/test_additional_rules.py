import polars as pl
import pytest

import polars_cnlp
from polars_cnlp.algorithms import ConText, ContextEffect, ContextRule, NegEx, RuleSet


def test_negex_additional_rule_preserves_defaults(terms):
    algorithm = NegEx(additional_rules=[
        ContextRule(
            pattern=r'\bcovid\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ])

    df = pl.DataFrame({'note_text': [
        'Covid pneumonia.',
        'No anaphylaxis.',
        'Pneumonia and anaphylaxis present.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_each(terms, algorithm=algorithm).alias('status'),
    )

    assert result['status'].to_list() == [
        {'pneumonia': False, 'anaphylaxis': None},
        {'pneumonia': None, 'anaphylaxis': False},
        {'pneumonia': True, 'anaphylaxis': True},
    ]


def test_negex_additional_rule_uses_negex_scope():
    algorithm = NegEx(additional_rules=[
        ContextRule(
            pattern=r'\bcovid\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ])

    df = pl.DataFrame({'note_text': [
        'Covid one two three four five pneumonia.',
        'Covid one two three four five six pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(r'\bpneumonia\b', algorithm=algorithm).alias('pneumonia'),
    )

    assert result['pneumonia'].to_list() == [False, True]


def test_context_additional_rule_preserves_defaults(terms):
    algorithm = ConText(additional_rules=[
        ContextRule(
            pattern=r'\bquestionable\b',
            direction='forward',
            effect=ContextEffect(assertion='possible'),
        ),
    ])

    df = pl.DataFrame({'note_text': [
        'Questionable pneumonia.',
        'No anaphylaxis.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed_each(terms, algorithm=algorithm).alias('status'),
    )

    assert result['status'].to_list() == [
        {'pneumonia': False, 'anaphylaxis': None},
        {'pneumonia': None, 'anaphylaxis': False},
    ]


def test_replacement_rules_do_not_include_defaults():
    covid_rule = ContextRule(
        pattern=r'\bcovid\b',
        direction='forward',
        effect=ContextEffect(assertion='negated'),
    )

    algorithm = NegEx(rules=RuleSet([covid_rule]))

    df = pl.DataFrame({'note_text': [
        'No pneumonia.',
        'Covid pneumonia.',
    ]})

    result = df.select(
        pl.col('note_text').cnlp.affirmed(r'\bpneumonia\b', algorithm=algorithm).alias('pneumonia'),
    )

    assert result['pneumonia'].to_list() == [True, False]


@pytest.mark.parametrize('algorithm_class', [
    ConText,
    NegEx,
])
def test_rules_and_additional_rules_are_mutually_exclusive(algorithm_class):
    rule = ContextRule(
        pattern=r'\bcovid\b',
        direction='forward',
        effect=ContextEffect(assertion='negated'),
    )

    with pytest.raises(ValueError, match='cannot be used together'):
        algorithm_class(
            rules=RuleSet([rule]),
            additional_rules=[rule],
        )
