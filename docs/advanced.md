---
layout: page
title: Advanced Usage
---


This page collects patterns that are useful in larger research pipelines and NLP validation work.

## Lazy queries

The namespace methods are normal Polars expressions and can be used in lazy pipelines.

```python
result = (
    pl.scan_parquet('notes/*.parquet')
    .filter(
        pl.col('note_text').cnlp.affirmed(
            r'\bpneumonia\b',
        ),
    )
    .select(
        'patient_id',
        'note_id',
        'note_text',
    )
    .collect()
)
```

For multiple concepts, compute the Struct once and then extract its fields:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}

result = (
    pl.scan_parquet('notes/*.parquet')
    .with_columns(
        pl.col('note_text')
        .cnlp.affirmed_each(terms)
        .alias('status'),
    )
    .with_columns(
        pl.col('status').struct.field('pneumonia').alias('pneumonia'),
        pl.col('status').struct.field('anaphylaxis').alias('anaphylaxis'),
    )
    .collect()
)
```

## Prefilter

All contextual methods accept `prefilter=True`:

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed(
        r'\bpneumonia\b',
        prefilter=True,
    ),
)
```

For multiple concepts:

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed_each(
        terms,
        prefilter=True,
    ),
)
```

When enabled, `polars-cnlp` first checks whether any requested concept regex matches the note. If none match, contextual
processing is skipped and the normal no-match result is returned.

This is most useful when target concepts are uncommon. When most notes contain at least one target, the extra concept
check can add overhead.

`prefilter` changes execution only; it does not change result semantics.

## Candidate filtering versus contextual filtering

Use `contains()` when you only need a broad regex-presence filter:

```python
candidates = df.filter(
    pl.col('note_text').cnlp.contains(r'\banaphyl(?:axis|actic)\b'),
)
```

Use `affirmed()` when context matters:

```python
affirmed = df.filter(
    pl.col('note_text').cnlp.affirmed(r'\banaphyl(?:axis|actic)\b'),
)
```

For very uncommon concepts, `prefilter=True` allows contextual methods to perform their own early target check without
requiring a separate pipeline stage.

## Inspect findings during validation

Boolean outputs are useful for phenotypes, but mention-level findings are often better for validation.

```python
review = (
    df.lazy()
    .with_columns(
        pl.col('note_text')
        .cnlp.find_all(terms)
        .alias('findings'),
    )
    .filter(
        pl.col('findings').list.len() > 0,
    )
    .collect()
)
```

Each finding includes label, byte offsets, assertion, temporality, and experiencer.

## Compare algorithms

```python
comparison = df.select(
    'note_text',
    pl.col('note_text')
    .cnlp.affirmed(
        r'\bpneumonia\b',
        algorithm='context',
    )
    .alias('context'),
    pl.col('note_text')
    .cnlp.affirmed(
        r'\bpneumonia\b',
        algorithm='negex',
    )
    .alias('negex'),
)
```

This is useful when validating whether an existing NegEx-based phenotype changes under richer ConText interpretation.

## Null behavior

The contextual Boolean methods distinguish absence from a non-affirmed mention:

```text
concept affirmed       -> True
concept non-affirmed   -> False
concept absent         -> null
input text null        -> null
```

For `count()`:

```text
concept absent         -> 0
input text null        -> null
```

For `find_all()`:

```text
concept absent         -> []
input text null        -> null
```

This distinction is intentional. Use Polars null-handling expressions explicitly when a downstream task wants to
collapse absence into `False`:

```python
df.filter(
    pl.col('note_text')
    .cnlp.affirmed(r'\bpneumonia\b')
    .fill_null(False),
)
```

## Regex guidance

Concepts are regular expressions. Prefer explicit boundaries where appropriate:

```python
r'\bpneumonia\b'
```

More flexible clinical concepts can be expressed directly:

```python
ANAPHYLAXIS = r'\banaphyl(?:axis|actic)\b'
```

For named multi-concept results, mapping keys become finding labels or Struct field names:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphyl(?:axis|actic)\b',
}
```

## Byte offsets

`find_best()` and `find_all()` return `start` and `end` as UTF-8 byte offsets, not Python character indices. Take this
into account when slicing text containing non-ASCII characters.
