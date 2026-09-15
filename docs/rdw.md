---
layout: page
title: Research data-wrangling API
---

# Research Data Wrangling API: `.rdw`

[Home](index.md) · [API](api.md) · [Clinical NLP](cnlp.md) · [RDW](rdw.md) · [Algorithms](algorithms.md) · [Custom rules](custom-rules.md) · [Advanced usage](advanced.md) · [How it works](how-it-works.md)

---

The `.rdw` namespace contains general-purpose helpers that are useful in research datasets but are not specific to
clinical NLP.

Both current methods use **literal** string matching rather than regular expressions.

## API overview

> [!IMPORTANT]
> Import `polars_cnlp` before using `.cnlp` or `.rdw`. The import registers these Polars expression namespaces and may
appear unused to editors or linters.
>
> ```python
> import polars as pl
> import polars_cnlp  # noqa: F401  # registers .cnlp and .rdw namespaces
> ```

| Method                                     | Input                        | Output    | Typical use                           |
|--------------------------------------------|------------------------------|-----------|---------------------------------------|
| [`starts_with_any()`](#rdwstarts_with_any) | sequence of literal prefixes | `Boolean` | Code families and identifier prefixes |
| [`ends_with_any()`](#rdwends_with_any)     | sequence of literal suffixes | `Boolean` | Code suffixes and file extensions     |

## `rdw.starts_with_any`

```python
starts_with_any(prefixes: Sequence[str]) -> pl.Expr
```

Check whether a string starts with any supplied literal prefix.

```python
df = pl.DataFrame({'code': [
    'A123',
    'B456',
    'C789',
    'X999',
    None,
]})

result = df.select(
    pl.col('code')
    .rdw.starts_with_any(['A', 'B', 'C'])
    .alias('selected'),
)
```

Expected values:

```python
[True, True, True, False, None]
```

Values are literal. A prefix such as `'a.'` matches the period itself rather than treating `.` as a regex wildcard.

### Code-family example

```python
respiratory = df.filter(
    pl.col('icd10').rdw.starts_with_any([
        'J12', 'J13', 'J14', 'J15', 'J16', 'J17', 'J18',
    ]),
)
```

### Behavior

```text
empty prefix collection -> False for non-null input
empty-string prefix      -> True for non-null input
null input               -> null
```

## `rdw.ends_with_any`

```python
ends_with_any(suffixes: Sequence[str]) -> pl.Expr
```

Check whether a string ends with any supplied literal suffix.

```python
df = pl.DataFrame({'filename': [
    'notes.parquet',
    'patients.csv',
    'readme.md',
    None,
]})

result = df.select(
    pl.col('filename')
    .rdw.ends_with_any(['.parquet', '.csv'])
    .alias('data_file'),
)
```

Expected values:

```python
[True, True, False, None]
```

### Procedure-code example

```python
selected = df.filter(
    pl.col('procedure_code').rdw.ends_with_any([
        '01',
        '02',
        '03',
    ]),
)
```

### Behavior

```text
empty suffix collection -> False for non-null input
empty-string suffix      -> True for non-null input
null input               -> null
```

## Why `.rdw` is separate from `.cnlp`

These operations are optimized convenience expressions for ordinary string/data-wrangling tasks. They do not perform
concept extraction or contextual interpretation, so keeping them under `.rdw` makes the distinction from clinical NLP
explicit.
