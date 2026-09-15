---
layout: page
title: How polars-cnlp works
---

# How polars-cnlp works

[Home](index.md) · [API](api.md) · [Clinical NLP](cnlp.md) · [RDW](rdw.md) · [Algorithms](algorithms.md) · [Custom rules](custom-rules.md) · [Advanced usage](advanced.md) · [How it works](how-it-works.md)

---

`polars-cnlp` is designed as a Polars expression extension rather than a separate note-processing framework. Users stay
inside ordinary Polars eager or lazy expressions while the computational work is implemented by the package's native
plugin.

## 1. Import registers expression namespaces

```python
import polars as pl
import polars_cnlp
```

After import, Polars expressions expose:

```python
pl.col('note_text').cnlp
pl.col('value').rdw
```

This keeps clinical NLP composable with `select`, `with_columns`, `filter`, lazy queries, and other Polars expressions.

## 2. Concepts are regex targets

Clinical concepts are supplied as regular expressions:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}
```

The package identifies target spans in the source text. Named mappings preserve the concept label so one note can be
processed for several concepts together.

Presence/count methods stop at this stage. Contextual methods continue into contextual interpretation.

## 3. Contextual methods resolve modifiers around targets

Contextual methods use a configurable algorithm:

```python
algorithm = 'context'  # default
algorithm = 'negex'
```

The rule engine distinguishes three lexical rule behaviors:

- **context modifiers** — assign assertion, temporality, and/or experiencer;
- **termination rules** — stop a modifier's scope;
- **pseudo rules** — suppress context modifiers that occur inside a known pseudo phrase.

Context modifiers can apply forward, backward, or bidirectionally and can limit scope by intervening tokens or number of
targets.

## 4. ConText produces independent dimensions

A finding has independent context dimensions:

```text
assertion   = affirmed | possible | negated
temporality = current | historical | hypothetical
experiencer = patient | other
```

This allows distinctions such as:

```text
Possible pneumonia.
    possible / current / patient

If pneumonia develops...
    affirmed / hypothetical / patient

Family history of pneumonia.
    affirmed / historical / other
```

Boolean `affirmed*` methods reduce this richer representation to a patient/current/affirmed result. `find_best()` and
`find_all()` retain the dimensions.

## 5. NegEx uses fixed-window mechanics

NegEx uses its own default rules and fixed directional target window. Its default window is six target positions,
equivalent to zero through five intervening terms.

ConText and NegEx share lower-level rule-resolution machinery, but their vocabulary and scope behavior are intentionally
not identical.

## 6. Modifier/target overlap is resolved per target

A lexical expression may be both a concept and a modifier. For example, a project may treat `kratom` as a modifier of
subsequent drug concepts while also searching for `kratom` itself.

A modifier does not contextualize a target whose span overlaps the modifier. While that overlapping target is resolved,
the modifier is also ignored as a modifier boundary so it does not block another valid modifier from reaching the
target.

That produces intuitive behavior:

```text
Kratom.
    kratom remains affirmed

Kratom and morphine.
    kratom can modify morphine

No kratom.
    "no" can still modify kratom
```

The exclusion is target-specific. The same modifier remains active for other non-overlapping targets.

## 7. Optional prefiltering avoids unnecessary context work

With:

```python
prefilter = True
```

a contextual expression first checks whether any requested concept occurs. If no concept matches, the rule engine is
skipped and the normal no-match value is returned.

This is useful for sparse concepts and intentionally remains an execution optimization rather than part of algorithm
configuration.

## 8. Results return as native Polars values

Depending on the expression, results are returned as:

- `Boolean`;
- `UInt32`;
- `Struct`;
- `List[Struct]`.

This makes the results directly usable in normal Polars pipelines.

## Finding offsets

Finding `start` and `end` fields are UTF-8 byte offsets into the original text. This matches the native regex/plugin
representation and avoids changing source-span coordinates during contextual processing.

## Design goals

The implementation aims to keep:

- the Python API expression-oriented and Polars-like;
- contextual algorithms configurable without changing expression syntax;
- multi-concept processing reusable within one note;
- rule customization declarative;
- common processing in native Rust code;
- algorithm semantics separate from execution optimizations such as `prefilter`.
