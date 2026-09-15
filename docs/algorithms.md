---
layout: page
title: Context Algorithms
---

The contextual `.cnlp` methods support interchangeable algorithms. ConText is the default; NegEx is available when
fixed-window NegEx behavior is desired.

The `algorithm=` argument is available on:

```text
affirmed
affirmed_any
affirmed_all
affirmed_each
find_best
find_all
```

`contains`, `count`, and `count_all` do not perform contextual analysis.

## ConText

```python
from polars_cnlp.algorithms import ConText
```

Default use:

```python
pl.col('note_text').cnlp.affirmed(r'\bpneumonia\b')
```

is equivalent to:

```python
pl.col('note_text').cnlp.affirmed(
    r'\bpneumonia\b',
    algorithm=ConText(),
)
```

ConText tracks three independent dimensions:

```text
assertion:    affirmed | possible | negated
temporality:  current | historical | hypothetical
experiencer:  patient | other
```

Examples:

```text
No pneumonia.
    assertion   = negated
    temporality = current
    experiencer = patient

Possible pneumonia.
    assertion   = possible
    temporality = current
    experiencer = patient

History of pneumonia.
    assertion   = affirmed
    temporality = historical
    experiencer = patient

If pneumonia develops...
    assertion   = affirmed
    temporality = hypothetical
    experiencer = patient

Family history of pneumonia.
    assertion   = affirmed
    temporality = historical
    experiencer = other
```

### Constructor

```python
ConText(
    rules: RuleSet | None = None,
additional_rules: Sequence[Rule] = (),
)
```

- `ConText()` uses the package defaults.
- `ConText(additional_rules=[...])` keeps the defaults and adds rules.
- `ConText(rules=RuleSet([...]))` replaces the defaults.
- `rules` and `additional_rules` cannot be used together.

## NegEx

```python
from polars_cnlp.algorithms import NegEx
```

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed(
        r'\bpneumonia\b',
        algorithm=NegEx(),
    ),
)
```

### Constructor

```python
NegEx(
    rules: RuleSet | None = None,
    additional_rules: Sequence[Rule] = (),
    window: int = 6,
    propagate_same_concept: bool = True,
)
```

`window=6` represents six possible target positions, equivalent to zero through five intervening terms.

Customize the window:

```python
algorithm = NegEx(window=8)
```

`propagate_same_concept=True` controls whether a negated or possible occurrence propagates to other occurrences of the
same concept according to NegEx behavior.

As with ConText:

```text
NegEx()                         -> built-in defaults
NegEx(additional_rules=[...])  -> built-in defaults + additions
NegEx(rules=RuleSet([...]))    -> replacement rule set
```

## ConText versus NegEx

The two algorithms intentionally do not have identical vocabulary or scope mechanics.

| Behavior           | ConText                             | NegEx                                   |
|--------------------|-------------------------------------|-----------------------------------------|
| default algorithm  | yes                                 | no                                      |
| context dimensions | assertion, temporality, experiencer | NegEx-style assertion-oriented behavior |
| scope              | rule-defined/sentence-style         | fixed target window                     |
| custom rules       | yes                                 | yes                                     |
| additional rules   | yes                                 | yes                                     |
| replacement rules  | yes                                 | yes                                     |

For research pipelines that reproduce an existing NegEx phenotype, use NegEx explicitly. For richer contextual
interpretation, ConText is the default.

## Finding ranking

`find_best()` ranks findings deterministically:

1. patient before other experiencer;
2. current before historical before hypothetical;
3. affirmed before possible before negated;
4. earlier text occurrence;
5. mapping order as the final tie-breaker.

This ranking affects only which finding `find_best()` returns. `find_all()` preserves every finding in text order.

## Next steps

- [Extend or replace rules](custom-rules.md)
- [Advanced usage and performance](advanced.md)
- [How contextual processing works](how-it-works.md)
