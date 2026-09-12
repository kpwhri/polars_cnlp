import pytest

from polars_cnlp.algorithms import (
    ContextEffect,
    ContextRule,
    NegEx,
    RuleSet,
)


def test_rule_set_is_immutable():
    rules = RuleSet([
        ContextRule(
            pattern=r'\babsent\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ])

    assert isinstance(rules.rules, tuple)
    assert len(rules) == 1


def test_rule_set_extend_returns_new_rule_set():
    original = RuleSet([
        ContextRule(
            pattern=r'\babsent\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ])

    extended = original.extend([
        ContextRule(
            pattern=r'\bremote\b',
            direction='forward',
            effect=ContextEffect(temporality='historical'),
        ),
    ])

    assert len(original) == 1
    assert len(extended) == 2


@pytest.mark.parametrize('tag, expected_direction, expected_assertion', [
    ('PREN', 'forward', 'negated'),
    ('POST', 'backward', 'negated'),
    ('PREP', 'forward', 'possible'),
    ('POSP', 'backward', 'possible'),
])
def test_negex_modifier_tags(tag, expected_direction, expected_assertion):
    rules = RuleSet.from_negex_lines([
        f'example trigger [{tag}]',
    ])

    rule = rules.rules[0]

    assert rule.direction == expected_direction
    assert rule.effect.assertion == expected_assertion


def test_negex_pseudo_tag():
    rules = RuleSet.from_negex_lines([
        'not only [PSEU]',
    ])

    rule = rules.rules[0]

    assert rule.pattern
    assert rule.effects == (
        ContextEffect(assertion='negated'),
        ContextEffect(assertion='possible'),
    )


def test_negex_conjunction_tag():
    rules = RuleSet.from_negex_lines([
        'but [CONJ]',
    ])

    rule = rules.rules[0]

    assert rule.pattern
    assert rule.effects == (
        ContextEffect(assertion='negated'),
        ContextEffect(assertion='possible'),
    )


def test_negex_trigger_parser_handles_multiple_rules():
    rules = RuleSet.from_negex_lines([
        'no [PREN]',
        'ruled out [POST]',
        'rule out [PREP]',
        'not only [PSEU]',
        'but [CONJ]',
    ])

    assert len(rules) == 5


def test_negex_trigger_parser_ignores_blank_lines_and_comments():
    rules = RuleSet.from_negex_lines([
        '',
        '# comment',
        'no [PREN]',
        '   ',
    ])

    assert len(rules) == 1


def test_negex_trigger_parser_can_exclude_possible_rules():
    rules = RuleSet.from_negex_lines(
        [
            'no [PREN]',
            'rule out [PREP]',
            'not ruled out [POSP]',
        ],
        include_possible=False,
    )

    assert len(rules) == 1


@pytest.mark.parametrize('line', [
    'this has no trigger tag',
    'no [UNKNOWN]',
    '[PREN]',
])
def test_invalid_negex_trigger_line_is_rejected(line):
    with pytest.raises(ValueError, match='line 1'):
        RuleSet.from_negex_lines([line])


def test_negex_accepts_replacement_rule_set():
    rules = RuleSet.from_negex_lines([
        'no [PREN]',
    ])

    algorithm = NegEx(rules=rules)

    assert algorithm.rules is rules


@pytest.mark.parametrize('kwargs', [
    {'assertion': 'invalid'},
    {'temporality': 'invalid'},
    {'experiencer': 'invalid'},
])
def test_invalid_context_effect_value_is_rejected(kwargs):
    with pytest.raises(ValueError):
        ContextEffect(**kwargs)


def test_empty_context_effect_is_rejected():
    with pytest.raises(ValueError, match='at least one'):
        ContextEffect()


@pytest.mark.parametrize('direction', [
    'left',
    'right',
    'around',
])
def test_invalid_context_rule_direction_is_rejected(direction):
    with pytest.raises(ValueError, match='direction'):
        ContextRule(
            pattern=r'\babsent\b',
            direction=direction,
            effect=ContextEffect(assertion='negated'),
        )


@pytest.mark.parametrize('max_scope, max_targets', [
    (-1, None),
    (None, 0),
    (None, -1),
])
def test_invalid_context_rule_scope_is_rejected(max_scope, max_targets):
    with pytest.raises(ValueError):
        ContextRule(
            pattern=r'\babsent\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
            max_scope=max_scope,
            max_targets=max_targets,
        )
