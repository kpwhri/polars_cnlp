---
layout: page
title: Clinical NLP API
---

# Clinical NLP API: `.cnlp`

[Home](index.md) · [API](api.md) · [Clinical NLP](cnlp.md) · [RDW](rdw.md) · [Algorithms](algorithms.md) · [Custom rules](custom-rules.md) · [Advanced usage](advanced.md) · [How it works](how-it-works.md)

---

The `.cnlp` namespace provides clinical concept matching and contextual interpretation as Polars expressions.

## API overview

> [!IMPORTANT]
> Import `polars_cnlp` before using `.cnlp` or `.rdw`. The import registers these Polars expression namespaces and may
appear unused to editors or linters.
>
> ```python
> import polars as pl
> import polars_cnlp  # noqa: F401  # registers .cnlp and .rdw namespaces
> ```


| Method                                  | Input               | Output              | Typical use                        |
|-----------------------------------------|---------------------|---------------------|------------------------------------|
| [`contains()`](#cnlpcontains)           | regex               | `Boolean`           | Presence check                     |
| [`count()`](#cnlpcount)                 | regex               | `UInt32`            | Count one concept                  |
| [`count_all()`](#cnlpcount_all)         | mapping             | `Struct[UInt32]`    | Count named concepts               |
| [`affirmed()`](#cnlpaffirmed)           | regex               | `Boolean/null`      | Contextual Boolean for one concept |
| [`affirmed_any()`](#cnlpaffirmed_any)   | sequence or mapping | `Boolean/null`      | Any requested concept affirmed     |
| [`affirmed_all()`](#cnlpaffirmed_all)   | sequence or mapping | `Boolean/null`      | All requested concepts affirmed    |
| [`affirmed_each()`](#cnlpaffirmed_each) | mapping             | `Struct[Boolean]`   | Status of each named concept       |
| [`find_best()`](#cnlpfind_best)         | regex or mapping    | `Struct/null`       | Best finding                       |
| [`find_all()`](#cnlpfind_all)           | regex or mapping    | `List[Struct]/null` | All findings                       |

The contextual methods use ConText by default. See [Algorithms](algorithms.md) to select NegEx or configure algorithm
behavior.

## `cnlp.contains`

```python
contains(pattern: str) -> pl.Expr
```

Check whether the text contains a regular expression.

```python
df.select(
    pl.col('note_text')
    .cnlp.contains(r'\bpneumonia\b')
    .alias('has_pneumonia'),
)
```

`contains()` performs regex presence matching only. It does not apply assertion, temporality, or experiencer rules.

## `cnlp.count`

```python
count(term: str) -> pl.Expr
```

Count occurrences of one concept.

```python
df.select(
    pl.col('note_text')
    .cnlp.count(r'\bpneumonia\b')
    .alias('pneumonia_count'),
)
```

For non-null text, an absent concept returns `0`. Null input remains null.

## `cnlp.count_all`

```python
count_all(terms: Mapping[str, str]) -> pl.Expr
```

Count occurrences of several named concepts in one expression.

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
    'asthma': r'\basthma\b',
}

result = df.select(
    pl.col('note_text')
    .cnlp.count_all(terms)
    .alias('counts'),
)
```

The result is a Struct with one `UInt32` field per mapping key:

```text
{
    pneumonia: 2,
    anaphylaxis: 0,
    asthma: 1
}
```

Extract fields using normal Polars Struct expressions:

```python
result.select(
    pl.col('counts').struct.field('pneumonia'),
    pl.col('counts').struct.field('anaphylaxis'),
)
```

## Contextual interpretation

Contextual findings have three independent dimensions:

| Dimension   | Values                                  |
|-------------|-----------------------------------------|
| assertion   | `affirmed`, `possible`, `negated`       |
| temporality | `current`, `historical`, `hypothetical` |
| experiencer | `patient`, `other`                      |

For Boolean affirmation methods, `True` means the concept has an occurrence that is affirmed, current, and attributed to
the patient.

## `cnlp.affirmed`

```python
affirmed(
    term: str,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

Determine whether a concept is affirmed.

```python
result = df.select(
    pl.col('note_text')
    .cnlp.affirmed(r'\bpneumonia\b')
    .alias('pneumonia'),
)
```

Return semantics:

```text
affirmed/current/patient occurrence -> True
concept mentioned but none affirmed -> False
concept absent                      -> null
input text null                     -> null
```

## `cnlp.affirmed_any`

```python
affirmed_any(
    terms: TermPatterns,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

`TermPatterns` may be a sequence of regexes or a mapping from labels to regexes.

```python
patterns = [
    r'\bpneumonia\b',
    r'\banaphylaxis\b',
]

df.select(
    pl.col('note_text')
    .cnlp.affirmed_any(patterns)
    .alias('any_affirmed'),
)
```

Semantics:

```text
at least one concept affirmed                 -> True
at least one concept mentioned, none affirmed -> False
none of the requested concepts mentioned      -> null
null input                                     -> null
```

## `cnlp.affirmed_all`

```python
affirmed_all(
    terms: TermPatterns,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

Return whether every requested concept is affirmed.

```python
patterns = [
    r'\bpneumonia\b',
    r'\banaphylaxis\b',
]

df.select(
    pl.col('note_text')
    .cnlp.affirmed_all(patterns)
    .alias('all_affirmed'),
)
```

The expression uses three-valued logic:

```text
True  + True  -> True
True  + False -> False
False + null  -> False
True  + null  -> null
null  + null  -> null
```

## `cnlp.affirmed_each`

```python
affirmed_each(
    terms: Mapping[str, str],
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

Return independent affirmation status for every named concept.

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}

result = df.select(
    pl.col('note_text')
    .cnlp.affirmed_each(terms)
    .alias('status'),
)
```

A result may resemble:

```python
{
    'pneumonia': False,
    'anaphylaxis': True,
}
```

Each field is `True`, `False`, or null using the same concept-level semantics as `affirmed()`.

## Finding structure

`find_best()` and `find_all()` return contextual findings with these fields:

```text
label        String/null
start        UInt64
end          UInt64
assertion    String
temporality  String
experiencer  String
```

`start` and `end` are UTF-8 byte offsets into the original text.

## `cnlp.find_best`

```python
find_best(
    terms: FindPatterns,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

`FindPatterns` may be one regex or a mapping from labels to regexes.

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
    'asthma': r'\basthma\b',
}

df.select(
    pl.col('note_text')
    .cnlp.find_best(terms)
    .alias('finding'),
)
```

When one regex is supplied, `label` is null. When several findings are available, selection is deterministic:

1. patient before other experiencer;
2. current before historical before hypothetical;
3. affirmed before possible before negated;
4. earlier text occurrence;
5. mapping order as the final tie-breaker.

## `cnlp.find_all`

```python
find_all(
    terms: FindPatterns,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

Return every contextualized finding in text order.

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}

df.select(
    pl.col('note_text')
    .cnlp.find_all(terms)
    .alias('findings'),
)
```

Semantics:

```text
non-null note, matches found -> list of findings
non-null note, no matches     -> []
null note                     -> null
```

Use `find_all()` for mention-level datasets, annotation review, NLP validation, and inspecting repeated mentions with
different context.

## See also

- [Algorithms](algorithms.md)
- [Custom rules](custom-rules.md)
- [Advanced usage](advanced.md)
