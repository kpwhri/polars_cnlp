---
layout: page
title: polars-cnlp
---


`polars-cnlp` adds clinical NLP and research-data-wrangling expressions directly to [Polars](https://pola.rs/).

Importing `polars_cnlp` registers two expression namespaces:

- `.cnlp` — regex-based clinical concept matching and contextual interpretation;
- `.rdw` — general research-data-wrangling string helpers.

The clinical NLP expressions can identify concepts, count mentions, determine whether concepts are affirmed, and return
mention-level context such as assertion, temporality, and experiencer. Contextual processing supports both ConText-style
and NegEx-style algorithms and can be customized with additional or replacement rules.

## Installation

```bash
pip install polars-cnlp
```

Then import the package once to register the expression namespaces:

```python
import polars as pl
import polars_cnlp  # noqa: F401  # registers .cnlp and .rdw namespaces
```

**Note:** Since we will not be using `polars_cnlp` directly, we're adding the `noqa` to avoid any linters removing this
import as it will appear 'unused'.

## Importing `polars-cnlp`

Import `polars_cnlp` before using the `.cnlp` or `.rdw` expression namespaces:

```python
import polars as pl
import polars_cnlp  # noqa: F401  # registers .cnlp and .rdw namespaces
```

The `polars_cnlp` import registers the package's Polars expression namespaces. After it has been imported, the
namespaces are available on Polars expressions:

```python
pl.col('note_text').cnlp.affirmed(r'\bpneumonia\b')
pl.col('code').rdw.starts_with_any(['J12', 'J13', 'J14'])
```

Although the `polars_cnlp` name does not need to be referenced directly afterward, the import itself is required for
namespace registration.

For scripts and modules that use `.cnlp` or `.rdw`, explicitly importing `polars_cnlp` is recommended rather than
relying on another module to have imported it previously.

## Quick example

```python
PNEUMONIA = r'\bpneumonia\b'

df = pl.DataFrame({'note_text': [
    'Patient has pneumonia.',
    'No pneumonia.',
    'Possible pneumonia.',
    'History of pneumonia.',
    'Family history of pneumonia.',
    'Patient has asthma.',
    None,
]})

result = df.with_columns(
    pl.col('note_text')
    .cnlp.affirmed(PNEUMONIA)
    .alias('pneumonia'),
)
```

`affirmed()` returns:

```python
[True, False, False, False, False, None, None]
```

A concept is considered affirmed when the selected finding is:

```text
assertion   = affirmed
temporality = current
experiencer = patient
```

Concept absence remains distinct from a non-affirmed mention: absent concepts return `null`, while mentioned but
negated, possible, historical, hypothetical, or other-experiencer concepts return `False`.

## Research-Data-Wrangling (rdw) example

```python
result = df_codes.select(
    pl.col('code')
    .rdw.starts_with_any(['J12', 'J13', 'J14', 'J15'])
    .alias('respiratory'),
)
```

The `.rdw` helpers use literal string matching rather than regular expressions.

## Documentation

| Page                                  | Contents                                                                   |
|---------------------------------------|----------------------------------------------------------------------------|
| [Public API](api.md)                  | Compact index of the public Python API                                     |
| [Clinical NLP API](cnlp.md)           | `.cnlp` expressions, return values, and examples                           |
| [Research Data Wrangling API](rdw.md) | `.rdw` expressions                                                         |
| [Algorithms](algorithms.md)           | ConText, NegEx, contextual dimensions, and finding ranking                 |
| [Custom rules](custom-rules.md)       | Extend defaults, replace rule sets, scope, pseudo, and termination rules   |
| [Advanced usage](advanced.md)         | Lazy queries, prefiltering, Struct outputs, null semantics, and validation |
| [How it works](how-it-works.md)       | High-level execution model and Rust-backed Polars plugin architecture      |

## Project links

- [GitHub repository](https://github.com/kpwhri/polars_cnlp)
- [Polars documentation](https://docs.pola.rs/)
