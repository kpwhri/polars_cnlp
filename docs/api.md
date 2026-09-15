---
layout: page
title: Public API
---


The public API is organized around two Polars expression namespaces plus configurable clinical-context algorithms.

## Contents

> [!IMPORTANT]
> Import `polars_cnlp` before using `.cnlp` or `.rdw`. The import registers these Polars expression namespaces and may
appear unused to editors or linters.
>
> ```python
> import polars as pl
> import polars_cnlp  # noqa: F401  # registers .cnlp and .rdw namespaces
> ```


### Clinical NLP expressions

| Method                                            | Input               | Output              | Typical use                          |
|---------------------------------------------------|---------------------|---------------------|--------------------------------------|
| [`cnlp.contains`](cnlp.md#cnlpcontains)           | regex               | `Boolean`           | Does the note mention a pattern?     |
| [`cnlp.count`](cnlp.md#cnlpcount)                 | regex               | `UInt32`            | Count one concept                    |
| [`cnlp.count_all`](cnlp.md#cnlpcount_all)         | mapping             | `Struct[UInt32]`    | Count named concepts                 |
| [`cnlp.affirmed`](cnlp.md#cnlpaffirmed)           | regex               | `Boolean/null`      | Is one concept affirmed?             |
| [`cnlp.affirmed_any`](cnlp.md#cnlpaffirmed_any)   | sequence or mapping | `Boolean/null`      | Is any requested concept affirmed?   |
| [`cnlp.affirmed_all`](cnlp.md#cnlpaffirmed_all)   | sequence or mapping | `Boolean/null`      | Are all requested concepts affirmed? |
| [`cnlp.affirmed_each`](cnlp.md#cnlpaffirmed_each) | mapping             | `Struct[Boolean]`   | Status for every named concept       |
| [`cnlp.find_best`](cnlp.md#cnlpfind_best)         | regex or mapping    | `Struct/null`       | Best contextualized finding          |
| [`cnlp.find_all`](cnlp.md#cnlpfind_all)           | regex or mapping    | `List[Struct]/null` | All contextualized findings          |

### Algorithms and rules

| Class                                                             | Purpose                                             |
|-------------------------------------------------------------------|-----------------------------------------------------|
| [`ConText`](algorithms.md#context)                                | Default multi-dimensional contextual interpretation |
| [`NegEx`](algorithms.md#negex)                                    | NegEx-style fixed-window contextual interpretation  |
| [`RuleBased`](custom-rules.md#fully-custom-rule-based-algorithms) | Custom rule-based algorithm                         |
| [`ContextEffect`](custom-rules.md#contexteffect)                  | Assertion, temporality, and/or experiencer change   |
| [`ContextRule`](custom-rules.md#contextrule)                      | Directional contextual modifier                     |
| [`TerminateRule`](custom-rules.md#terminaterule)                  | Stop modifier scope                                 |
| [`PseudoRule`](custom-rules.md#pseudorule)                        | Suppress an overlapping modifier                    |
| [`RuleSet`](custom-rules.md#ruleset)                              | Immutable collection of rules                       |

### Research-data-wrangling expressions

| Method                                             | Input            | Output    | Typical use                         |
|----------------------------------------------------|------------------|-----------|-------------------------------------|
| [`rdw.starts_with_any`](rdw.md#rdwstarts_with_any) | literal prefixes | `Boolean` | Match code or identifier families   |
| [`rdw.ends_with_any`](rdw.md#rdwends_with_any)     | literal suffixes | `Boolean` | Match suffix families or file types |

## Import

```python
import polars as pl
import polars_cnlp
```

Algorithm configuration classes are imported separately:

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

The `.cnlp` and `.rdw` namespaces are expression APIs, so their methods compose with ordinary Polars `select`,
`with_columns`, `filter`, and lazy pipelines.
