---
layout: page
title: Custom rules and algorithms
---

# Custom rules and algorithms

[Home](index.md) · [API](api.md) · [Clinical NLP](cnlp.md) · [RDW](rdw.md) · [Algorithms](algorithms.md) · [Custom rules](custom-rules.md) · [Advanced usage](advanced.md) · [How it works](how-it-works.md)

---

Rule customization is useful when a project needs vocabulary that is not present in the built-in ConText or NegEx rules.

```python
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
```

## Extend the built-in defaults

Use `additional_rules=` when you want to keep the package defaults and add a small number of project-specific expressions.

### ConText

```python
algorithm = ConText(
    additional_rules=[
        ContextRule(
            pattern=r'\bfree\W+of\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ],
)
```

### NegEx

```python
algorithm = NegEx(
    additional_rules=[
        ContextRule(
            pattern=r'\bcovid\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ],
)
```

Use the configured algorithm normally:

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed_each(
        {
            'pneumonia': r'\bpneumonia\b',
            'anaphylaxis': r'\banaphylaxis\b',
        },
        algorithm=algorithm,
    ),
)
```

The built-in rules remain active.

## Replace the defaults

Use `rules=` when you want full control over the rule set:

```python
rules = RuleSet([
    ContextRule(
        pattern=r'\babsent\b',
        direction='forward',
        effect=ContextEffect(assertion='negated'),
    ),
])

algorithm = ConText(rules=rules)
```

Summary:

```text
ConText()                         -> built-in ConText rules
ConText(additional_rules=[...])  -> built-in rules + additions
ConText(rules=RuleSet([...]))    -> replacement rules

NegEx()                          -> built-in NegEx rules
NegEx(additional_rules=[...])    -> built-in rules + additions
NegEx(rules=RuleSet([...]))      -> replacement rules
```

`rules=` and `additional_rules=` are mutually exclusive.

## `ContextEffect`

```python
ContextEffect(
    assertion: Literal['affirmed', 'possible', 'negated'] | None = None,
    temporality: Literal['current', 'historical', 'hypothetical'] | None = None,
    experiencer: Literal['patient', 'other'] | None = None,
)
```

An effect may modify one or several context dimensions:

```python
family_history = ContextEffect(
    temporality='historical',
    experiencer='other',
)
```

At least one dimension must be supplied.

## `ContextRule`

```python
ContextRule(
    pattern: str,
    direction: Literal['forward', 'backward', 'bidirectional'],
    effect: ContextEffect,
    max_scope: int | None = None,
    max_targets: int | None = None,
    terminated_by: Sequence[ContextEffect] = (),
)
```

### Direction

```python
ContextRule(
    pattern=r'\bno\b',
    direction='forward',
    effect=ContextEffect(assertion='negated'),
)
```

```python
ContextRule(
    pattern=r'\bexcluded\b',
    direction='backward',
    effect=ContextEffect(assertion='negated'),
)
```

```python
ContextRule(
    pattern=r'\bsuspected\b',
    direction='bidirectional',
    effect=ContextEffect(assertion='possible'),
)
```

### Limit scope

`max_scope` is the maximum number of intervening tokens:

```python
ContextRule(
    pattern=r'\bremote\b',
    direction='forward',
    effect=ContextEffect(temporality='historical'),
    max_scope=3,
)
```

`max_targets` limits how many target occurrences one modifier occurrence may affect:

```python
ContextRule(
    pattern=r'\bstatus\W+post\b',
    direction='forward',
    effect=ContextEffect(temporality='historical'),
    max_targets=1,
)
```

## `TerminateRule`

```python
TerminateRule(
    pattern: str,
    effects: Sequence[ContextEffect] = (),
)
```

A termination rule stops modifier scope.

```python
negated = ContextEffect(assertion='negated')

rules = RuleSet([
    ContextRule(
        pattern=r'\babsent\b',
        direction='forward',
        effect=negated,
    ),
    TerminateRule(
        pattern=r'\bbut\b',
        effects=[negated],
    ),
])
```

For:

```text
Absent fever but pneumonia present.
```

`but` prevents the earlier modifier from negating `pneumonia`.

An empty `effects` sequence makes the terminator apply globally.

## `PseudoRule`

```python
PseudoRule(
    pattern: str,
    effects: Sequence[ContextEffect] = (),
)
```

A pseudo rule suppresses an overlapping context modifier.

```python
negated = ContextEffect(assertion='negated')

rules = RuleSet([
    ContextRule(
        pattern=r'\bnegative\b',
        direction='forward',
        effect=negated,
    ),
    PseudoRule(
        pattern=r'\bnegative\W+attitude\b',
        effects=[negated],
    ),
])
```

For:

```text
Negative attitude regarding pneumonia.
```

the pseudo rule prevents `negative` from acting as a negation trigger.

## `RuleSet`

```python
RuleSet(rules: Sequence[Rule])
```

A `RuleSet` is an immutable collection of context, termination, and pseudo rules.

Extend an existing rule set:

```python
extended = rules.extend([
    ContextRule(
        pattern=r'\bwithout\W+evidence\W+of\b',
        direction='forward',
        effect=ContextEffect(assertion='negated'),
    ),
])
```

### Load a NegEx trigger file

```python
rules = RuleSet.from_negex_file(
    'negex_triggers.txt',
)
```

Supported trigger categories are:

```text
PREN
POST
PREP
POSP
PSEU
CONJ
```

Use `include_possible=False` if PREP/POSP possible triggers should be excluded.

## Fully custom rule-based algorithms

```python
RuleBased(
    rules: RuleSet,
    window: int | None = None,
)
```

`window=None` uses ConText-style rule-defined scope. Supplying a positive integer uses a fixed directional target window.

```python
algorithm = RuleBased(
    rules=rules,
    window=6,
)
```

## Concepts that also match modifier rules

A concept may also match a contextual modifier rule. This can be useful when one vocabulary term should modify other concepts.

If a modifier's text span overlaps a target span, the modifier is not applied to that same target. The overlapping modifier is also ignored as a modifier boundary while that particular target is resolved.

For example, suppose `kratom` is both a searched concept and a forward negation modifier:

```text
Kratom.
```

`kratom` does not negate itself.

```text
Kratom and morphine.
```

`kratom` may still negate `morphine`.

```text
No kratom.
```

the overlapping `kratom` modifier does not prevent the separate `no` modifier from negating `kratom`.

This behavior is based on **overlapping text spans**, not on whether the concept regex and modifier regex are identical. If a concept pattern intentionally includes text matched by a modifier, that modifier will not contextualize that concept occurrence.

This overlap behavior applies to contextual modifiers. Explicit `TerminateRule` and `PseudoRule` behavior remains controlled by those rule types.
