# polars cNLP

`polars-cnlp` adds clinical NLP (cnlp) and research-data-wrangling (rdw) expressions to Polars.

Importing `polars_cnlp` registers two expression namespaces:

- `.cnlp` — clinical concept matching and contextual interpretation
- `.rdw` — general research data-wrangling string helpers

```python
import polars as pl
import polars_cnlp
```

Once imported, the namespaces are available directly on Polars expressions:

```python
pl.col('note_text').cnlp.affirmed(r'\bpneumonia\b')
pl.col('code').rdw.starts_with_any(['A', 'B', 'C'])
```

## Contents

- [Installation](#installation)
- [Clinical NLP Namespace: `.cnlp`](#clinical-nlp-namespace-cnlp)
- [Choosing a context algorithm](#choosing-a-context-algorithm)
- [Custom rules and algorithms](#custom-rules-and-algorithms)
- [Research Data-Wrangling Namespace: `.rdw`](#research-data-wrangling-namespace-rdw)
- [Lazy queries](#lazy-queries)
- [Choosing the right method](#choosing-the-right-method)
- [Regex patterns](#regex-patterns)
- [Null behavior](#null-behavior)

## Installation

```bash
pip install polars-cnlp
```

Then:

```python
import polars as pl
import polars_cnlp
```

## Quick start

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
    pl.col('note_text').cnlp.affirmed(PNEUMONIA).alias('pneumonia'),
)

print(result)
```

`affirmed()` returns:

- `True` when at least one matching finding is affirmed, current, and experienced by the patient.
- `False` when the concept is mentioned but all matching findings are non-affirmed, historical, hypothetical, or
  attributed to another experiencer.
- `null` when the concept is not mentioned or the input text is null.

For the example above:

```python
[True, False, False, False, False, None, None]
```

---

# Clinical NLP namespace: `.cnlp`

## API overview

| Method                                  | Input               | Output              | Typical Usage                                |
|-----------------------------------------|---------------------|---------------------|----------------------------------------------|
| [`contains()`](#cnlpcontains)           | regex               | `Boolean`           | Does the note mention a pattern?             |
| [`count()`](#cnlpcount)                 | regex               | `UInt32`            | How many times does one concept occur?       |
| [`count_all()`](#cnlpcount_all)         | `{label: regex}`    | `Struct[UInt32]`    | Count several concepts at once               |
| [`affirmed()`](#cnlpaffirmed)           | regex               | `Boolean/null`      | Is one concept affirmed?                     |
| [`affirmed_any()`](#cnlpaffirmed_any)   | sequence or mapping | `Boolean/null`      | Is any requested concept affirmed?           |
| [`affirmed_all()`](#cnlpaffirmed_all)   | sequence or mapping | `Boolean/null`      | Are all requested concepts affirmed?         |
| [`affirmed_each()`](#cnlpaffirmed_each) | `{label: regex}`    | `Struct[Boolean]`   | Get affirmation status for every concept     |
| [`find_best()`](#cnlpfind_best)         | regex or mapping    | `Struct/null`       | Return the highest-ranked contextual finding |
| [`find_all()`](#cnlpfind_all)           | regex or mapping    | `List[Struct]/null` | Return every contextualized finding          |

The contextual methods use ConText by default. NegEx and custom algorithms can be selected with the `algorithm=`
argument.

## `cnlp.contains`

```python
contains(pattern: str) -> pl.Expr
```

Check whether text contains a regular expression.

```python
df = pl.DataFrame({'note_text': [
    'Pneumonia present.',
    'No respiratory findings.',
    None,
]})

result = df.select(
    pl.col('note_text').cnlp.contains(r'\bpneumonia\b').alias('has_pneumonia'),
)
```

Use cases:

- build broad candidate cohorts before contextual analysis;
- identify notes containing a disease, medication, procedure, or symptom;
- combine regex presence checks with normal Polars filters.

```python
candidates = df.filter(
    pl.col('note_text').cnlp.contains(r'\banaphyl(?:axis|actic)\b'),
)
```

`contains()` only checks whether the regex occurs. It does not determine whether the concept is negated, historical,
hypothetical, or attributed to another experiencer.

## `cnlp.count`

```python
count(term: str) -> pl.Expr
```

Count occurrences of one concept.

```python
PNEUMONIA = r'\bpneumonia\b'

df = pl.DataFrame({'note_text': [
    'Pneumonia. Pneumonia improved.',
    'No pneumonia.',
    'Asthma only.',
    None,
]})

result = df.select(
    pl.col('note_text').cnlp.count(PNEUMONIA).alias('pneumonia_count'),
)
```

Expected counts:

```python
[2, 1, 0, None]
```

Use `count()` for mention frequency, simple NLP features, or identifying notes with repeated discussion of a concept.

Because `count()` does not perform contextual analysis, it is cheaper than `find_all()` or `affirmed()`.

## `cnlp.count_all`

```python
count_all(terms: Mapping[str, str]) -> pl.Expr
```

Count several named concepts in one expression.

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
    'asthma': r'\basthma\b',
}

df = pl.DataFrame({'note_text': [
    'Pneumonia and asthma. Pneumonia again.',
    'Anaphylaxis.',
]})

result = df.select(
    pl.col('note_text').cnlp.count_all(terms).alias('counts'),
)
```

The result is a Struct resembling:

```text
{
    pneumonia: 2,
    anaphylaxis: 0,
    asthma: 1
}
```

Struct fields can be selected with normal Polars expressions:

```python
result = (
    df.with_columns(
        pl.col('note_text').cnlp.count_all(terms).alias('counts'),
    )
    .select(
        pl.col('counts').struct.field('pneumonia'),
        pl.col('counts').struct.field('anaphylaxis'),
        pl.col('counts').struct.field('asthma'),
    )
)
```

Use `count_all()` when you need multiple count features for a known vocabulary.

---

# Contextual interpretation

Contextual findings have three independent dimensions.

### Assertion

```text
affirmed
possible
negated
```

### Temporality

```text
current
historical
hypothetical
```

### Experiencer

```text
patient
other
```

The dimensions are independent. For example:

```text
Possible pneumonia.
```

can be:

```text
assertion   = possible
temporality = current
experiencer = patient
```

while:

```text
If possible pneumonia develops...
```

can be:

```text
assertion   = possible
temporality = hypothetical
experiencer = patient
```

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
PNEUMONIA = r'\bpneumonia\b'

df = pl.DataFrame({'note_text': [
    'Patient has pneumonia.',
    'No pneumonia.',
    'Possible pneumonia.',
    'History of pneumonia.',
    'Family history of pneumonia.',
    'If pneumonia develops, return to clinic.',
    'Patient has asthma.',
    None,
]})

result = df.select(
    pl.col('note_text').cnlp.affirmed(PNEUMONIA).alias('pneumonia'),
)
```

Expected result:

```python
[
    True,  # affirmed, current, patient
    False,  # negated
    False,  # possible
    False,  # historical
    False,  # other experiencer
    False,  # hypothetical
    None,  # concept absent
    None,  # null input
]
```

A finding is considered affirmed only when:

```text
assertion   = affirmed
temporality = current
experiencer = patient
```

Filtering to affirmed notes:

```python
affirmed_notes = df.filter(
    pl.col('note_text').cnlp.affirmed(PNEUMONIA),
)
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

Return whether any requested concept is affirmed.

The input can be a sequence:

```python
patterns = [
    r'\bpneumonia\b',
    r'\banaphylaxis\b',
]

result = df.select(
    pl.col('note_text').cnlp.affirmed_any(patterns).alias('any_affirmed'),
)
```

or a mapping:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}

result = df.select(
    pl.col('note_text').cnlp.affirmed_any(terms).alias('any_affirmed'),
)
```

Semantics:

```text
at least one concept affirmed                 -> True
at least one concept mentioned, none affirmed -> False
none of the requested concepts mentioned      -> null
null input                                     -> null
```

Example:

```python
df = pl.DataFrame({'note_text': [
    'No pneumonia. Anaphylaxis present.',
    'No pneumonia.',
    'Asthma only.',
]})

result = df.select(
    pl.col('note_text').cnlp.affirmed_any(patterns).alias('result'),
)
```

Produces:

```python
[True, False, None]
```

Typical use case:

```python
cohort = df.filter(
    pl.col('note_text').cnlp.affirmed_any([
        r'\bpneumonia\b',
        r'\bbronchitis\b',
        r'\bbronchiolitis\b',
    ]),
)
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

df = pl.DataFrame({'note_text': [
    'Pneumonia and anaphylaxis are present.',
    'No pneumonia but anaphylaxis is present.',
    'Pneumonia present.',
    'Asthma only.',
]})

result = df.select(
    pl.col('note_text').cnlp.affirmed_all(patterns).alias('all_affirmed'),
)
```

The method uses three-valued logic:

```text
True  + True  -> True
True  + False -> False
False + null  -> False
True  + null  -> null
null  + null  -> null
```

Typical use cases:

- phenotypes requiring several co-occurring concepts;
- validation rules where every required concept must be affirmed;
- direct filtering with multiple clinical criteria.

```python
cohort = df.filter(
    pl.col('note_text').cnlp.affirmed_all(patterns),
)
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

df = pl.DataFrame({'note_text': [
    'No pneumonia. Anaphylaxis present.',
]})

result = df.select(
    pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
)
```

Produces a Struct resembling:

```python
{
    'pneumonia': False,
    'anaphylaxis': True,
}
```

Each field is:

```text
True  -> concept has an affirmed occurrence
False -> concept occurs but has no affirmed occurrence
null  -> concept is absent
```

Extract individual fields:

```python
result = (
    df.with_columns(
        pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
    )
    .with_columns(
        pl.col('status').struct.field('pneumonia').alias('pneumonia'),
        pl.col('status').struct.field('anaphylaxis').alias('anaphylaxis'),
    )
)
```

Use `affirmed_each()` when you need several contextual Boolean features while processing the note once.

---

# Rich contextual findings

## Finding structure

`find_best()` and `find_all()` return findings with:

```text
label        String/null
start        UInt64
end          UInt64
assertion    String
temporality  String
experiencer  String
```

`start` and `end` are UTF-8 byte offsets into the original text.

Example:

```python
{
    'label': 'pneumonia',
    'start': 3,
    'end': 12,
    'assertion': 'negated',
    'temporality': 'current',
    'experiencer': 'patient',
}
```

## `cnlp.find_best`

```python
find_best(
    terms: FindPatterns,
*,
algorithm: AlgorithmLike = 'context',
prefilter: bool = False,
) -> pl.Expr
```

Return the highest-ranked contextualized finding.

Single concept:

```python
result = df.select(
    pl.col('note_text')
    .cnlp.find_best(r'\bpneumonia\b')
    .alias('finding'),
)
```

When a single regex is passed, `label` is null.

Several named concepts:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
    'asthma': r'\basthma\b',
}

df = pl.DataFrame({'note_text': [
    'No pneumonia. Possible anaphylaxis. Asthma present.',
]})

result = df.select(
    pl.col('note_text').cnlp.find_best(terms).alias('finding'),
)

finding = result['finding'][0]
```

The selected finding is `asthma` because it is affirmed and current, while pneumonia is negated and anaphylaxis is
possible.

Ranking is deterministic:

1. patient before other experiencer;
2. current before historical before hypothetical;
3. affirmed before possible before negated;
4. earlier text occurrence;
5. mapping order as the final tie-breaker.

Use cases:

- select one representative finding;
- reduce multiple candidate concepts to one feature;
- return the preferred span for review or annotation.

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

df = pl.DataFrame({'note_text': [
    'No pneumonia. Possible anaphylaxis. Pneumonia later developed.',
]})

result = df.select(
    pl.col('note_text').cnlp.find_all(terms).alias('findings'),
)
```

Semantics:

```text
non-null note, matches found -> list of findings
non-null note, no matches     -> []
null note                     -> null
```

Use `find_all()` for:

- annotation review;
- NLP validation;
- exact source spans;
- mention-level datasets;
- comparing contextual status across repeated mentions.

---

# Choosing a context algorithm

All contextual methods use ConText by default:

```python
pl.col('note_text').cnlp.affirmed(PNEUMONIA)
```

is equivalent to:

```python
pl.col('note_text').cnlp.affirmed(
    PNEUMONIA,
    algorithm='context',
)
```

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

## Prefiltering

All contextual methods accept `prefilter=True`:

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed(
        PNEUMONIA,
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
    ).alias('status'),
)
```

When enabled, `polars-cnlp` first checks whether any requested concept regex matches the note. If none match, it skips
ConText, NegEx, or other contextual processing and returns the normal no-match result. This can improve performance when
the requested concepts are uncommon. When most notes contain a matching concept, the prefilter may add some overhead.

`prefilter` changes execution only; it does not change result semantics.

## ConText

```python
from polars_cnlp.algorithms import ConText

result = df.select(
    pl.col('note_text').cnlp.affirmed(
        PNEUMONIA,
        algorithm=ConText(),
    ),
)
```

ConText supports independent assertion, temporality, and experiencer dimensions.

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

## NegEx

Use NegEx when you specifically want NegEx-style trigger and fixed-window behavior.

```python
result = df.select(
    pl.col('note_text').cnlp.affirmed(
        PNEUMONIA,
        algorithm='negex',
    ),
)
```

Or:

```python
from polars_cnlp.algorithms import NegEx

result = df.select(
    pl.col('note_text').cnlp.affirmed(
        PNEUMONIA,
        algorithm=NegEx(),
    ),
)
```

Customize the window:

```python
algorithm = NegEx(window=8)
```

NegEx and ConText intentionally do not have identical rule vocabularies or scope behavior.

---

# Custom rules and algorithms

```python
from polars_cnlp.algorithms import (
    ConText,
    ContextEffect,
    ContextRule,
    PseudoRule,
    RuleBased,
    RuleSet,
    TerminateRule,
)
```

## Add rules to the defaults

Use `additional_rules=` when you want to keep the built-in ConText or NegEx rules and add only a few project-specific
expressions.

For example, extend ConText with an additional negation phrase:

```python
from polars_cnlp.algorithms import ConText, ContextEffect, ContextRule

algorithm = ConText(
    additional_rules=[
        ContextRule(
            pattern=r'\bfree\W+of\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ],
)

result = df.select(
    pl.col('note_text').cnlp.affirmed(
        r'\bpneumonia\b',
        algorithm=algorithm,
    ),
)
```

The built-in ConText rules remain active; the supplied rule is added to them.

The same approach works with NegEx:

```python
from polars_cnlp.algorithms import NegEx, ContextEffect, ContextRule

algorithm = NegEx(
    additional_rules=[
        ContextRule(
            pattern=r'\bcovid\b',
            direction='forward',
            effect=ContextEffect(assertion='negated'),
        ),
    ],
)

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

Use `additional_rules=` to extend the defaults. Use `rules=` instead when you want to replace the built-in rule set
completely.

```text
ConText()                         -> built-in ConText rules
ConText(additional_rules=[...])  -> built-in rules + additional rules
ConText(rules=RuleSet([...]))    -> replacement rule set

NegEx()                          -> built-in NegEx rules
NegEx(additional_rules=[...])    -> built-in rules + additional rules
NegEx(rules=RuleSet([...]))      -> replacement rule set
```

`rules=` and `additional_rules=` cannot be used together.

### Concepts that also match modifier rules

A concept may also match a ConText or NegEx modifier rule. This can be useful when a term acts as a modifier for other
concepts:

```text
Kratom and morphine.
^^^^^^     ^^^^^^^^
modifier   target
```

A modifier is not applied to a target whose text span overlaps the modifier itself. The overlapping modifier is also
ignored as a modifier boundary when resolving that target.

For example, if `kratom` is both a searched concept and a forward negation rule:

```text
Kratom.
```

`kratom` does not negate itself.

```text
Kratom and morphine.
```

`kratom` can still negate `morphine`.

```text
No kratom.
```

the overlapping `kratom` rule does not prevent the separate `no` rule from negating `kratom`.

This behavior is based on overlapping text spans, not on whether the concept and modifier use identical regular
expressions. If a concept pattern intentionally includes text matched by a modifier, that modifier will not
contextualize that concept occurrence.

## Replace the ConText rule set

Sometimes, we want a completely new set of rules. To do this, we'll create a new RuleSet and pass it to the algorithm
constructor.

```python
rules = RuleSet([
    ContextRule(
        pattern=r'\babsent\b',
        direction='forward',
        effect=ContextEffect(assertion='negated'),
    ),
])

algorithm = ConText(rules=rules)

result = df.select(
    pl.col('note_text').cnlp.affirmed(
        PNEUMONIA,
        algorithm=algorithm,
    ),
)
```

Supplying `rules=` replaces the built-in rule set rather than silently extending it.

## Multi-dimensional effects

```python
algorithm = ConText(
    rules=RuleSet([
        ContextRule(
            pattern=r'\bfamily\W+record\W+of\b',
            direction='forward',
            effect=ContextEffect(
                temporality='historical',
                experiencer='other',
            ),
        ),
    ]),
)
```

## Forward, backward, and bidirectional rules

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

## Limit scope

`max_scope` is the maximum number of intervening tokens:

```python
ContextRule(
    pattern=r'\bremote\b',
    direction='forward',
    effect=ContextEffect(temporality='historical'),
    max_scope=3,
)
```

`max_targets` limits the number of target occurrences modified:

```python
ContextRule(
    pattern=r'\bstatus\W+post\b',
    direction='forward',
    effect=ContextEffect(temporality='historical'),
    max_targets=1,
)
```

## Termination rules

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

`but` prevents the earlier modifier from negating pneumonia.

## Pseudo rules

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

the pseudo rule prevents `negative` from incorrectly negating pneumonia.

## Load a NegEx trigger file

```python
from polars_cnlp.algorithms import NegEx, RuleSet

rules = RuleSet.from_negex_file(
    'negex_triggers.txt',
)

algorithm = NegEx(
    rules=rules,
)
```

Supported trigger categories include:

```text
PREN
POST
PREP
POSP
PSEU
CONJ
```

This is useful when a project needs to reproduce a specific NegEx lexicon.

---

# Research data-wrangling namespace: `.rdw`

The `.rdw` namespace contains general-purpose helpers useful in research datasets but not specific to clinical NLP.

## API overview

| Method                                     | Input                          | Output    | Typical use                                                    |
|--------------------------------------------|--------------------------------|-----------|----------------------------------------------------------------|
| [`starts_with_any()`](#rdwstarts_with_any) | collection of literal prefixes | `Boolean` | Match code families, identifiers, or other prefix-based values |
| [`ends_with_any()`](#rdwends_with_any)     | collection of literal suffixes | `Boolean` | Match file types, code endings, or other suffix-based values   |

Both methods treat the supplied values as literal strings rather than regular expressions.

## `rdw.starts_with_any`

```python
starts_with_any(prefixes: Sequence[str]) -> pl.Expr
```

Check if a column starts with any of a collection/list of strings.

### Parameters

`prefixes`
: List of prefixes.

### Returns

`Expr`
: Expression of data type `Bool`.

### Example

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

Expected result:

```python
[True, True, True, False, None]
```

Values are treated as literal strings, not regular expressions.

```python
df = pl.DataFrame({'value': [
    'a.b',
    'axb',
]})

result = df.select(
    pl.col('value').rdw.starts_with_any(['a.']).alias('result'),
)
```

Produces:

```python
[True, False]
```

### Use case: ICD families

```python
respiratory = df.filter(
    pl.col('icd10').rdw.starts_with_any([
        'J12', 'J13', 'J14', 'J15', 'J16', 'J17', 'J18',
    ]),
)
```

### Use case: identifier families

```python
selected = df.filter(
    pl.col('study_id').rdw.starts_with_any([
        'CASE-',
        'CTRL-',
        'PILOT-',
    ]),
)
```

Behavior:

```text
empty prefix collection -> False for non-null input
empty-string prefix      -> True for non-null input
null input               -> null
```

## `rdw.ends_with_any`

```python
ends_with_any(suffixes: Sequence[str]) -> pl.Expr
```

Check if a column ends with any of a collection/list of strings.

### Parameters

`suffixes`
: List of suffixes.

### Returns

`Expr`
: Expression of data type `Bool`.

### Example

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

Expected result:

```python
[True, True, False, None]
```

Suffixes are literal strings, not regular expressions.

### Use case: procedure-code suffixes

```python
selected = df.filter(
    pl.col('procedure_code').rdw.ends_with_any([
        '01',
        '02',
        '03',
    ]),
)
```

### Use case: file filtering

```python
files = df.filter(
    pl.col('path').rdw.ends_with_any([
        '.parquet',
        '.csv',
        '.sas7bdat',
    ]),
)
```

Behavior:

```text
empty suffix collection -> False for non-null input
empty-string suffix      -> True for non-null input
null input               -> null
```

---

# Lazy queries

All namespace methods are designed for normal Polars lazy expressions.

```python
result = (
    pl.scan_parquet('notes/*.parquet')
    .filter(
        pl.col('note_text').cnlp.affirmed(
            r'\bpneumonia\b',
        ),
    )
    .select('patient_id', 'note_id', 'note_text')
    .collect()
)
```

Multi-concept processing:

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
        .alias('concept_status'),
    )
    .collect()
)
```

---

# Choosing the right method

Use `contains()` when:

- you only need regex presence;
- context is unnecessary;
- you are creating a broad candidate set.

Use `count()` or `count_all()` when:

- mention frequency matters;
- context does not;
- you want count features.

Use `affirmed()` when:

- you want a patient/current/affirmed Boolean phenotype for one concept.

Use `affirmed_any()` when:

- several alternative concepts can satisfy a phenotype.

Use `affirmed_all()` when:

- all required concepts must be affirmed.

Use `affirmed_each()` when:

- you need one contextual Boolean feature per concept.

Use `find_best()` when:

- one representative contextual finding is needed.

Use `find_all()` when:

- mention-level context and spans matter;
- you are validating NLP output;
- different mentions in the same note must remain distinguishable.

Use `.rdw.starts_with_any()` and `.rdw.ends_with_any()` when:

- matching literal prefixes or suffixes;
- a list is clearer than constructing a regular expression;
- the task is data wrangling rather than clinical NLP.

---

# Example: clinical phenotype

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
    'asthma': r'\basthma\b',
}

phenotype = (
    df.lazy()
    .with_columns(
        pl.col('note_text')
        .cnlp.affirmed_each(terms)
        .alias('nlp'),
    )
    .with_columns(
        pl.col('nlp').struct.field('pneumonia').alias('pneumonia'),
        pl.col('nlp').struct.field('anaphylaxis').alias('anaphylaxis'),
        pl.col('nlp').struct.field('asthma').alias('asthma'),
    )
    .filter(
        pl.col('pneumonia').fill_null(False)
        | pl.col('anaphylaxis').fill_null(False)
        | pl.col('asthma').fill_null(False),
    )
    .collect()
)
```

# Example: inspect findings during validation

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphylaxis\b',
}

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

This is useful for reviewing labels, spans, assertion, temporality, and experiencer.

# Example: compare ConText and NegEx

```python
comparison = df.select(
    'note_text',
    pl.col('note_text')
    .cnlp.affirmed(
        PNEUMONIA,
        algorithm='context',
    )
    .alias('context'),
    pl.col('note_text')
    .cnlp.affirmed(
        PNEUMONIA,
        algorithm='negex',
    )
    .alias('negex'),
)
```

This is useful when validating whether an existing NegEx-based phenotype changes under richer ConText-style
interpretation.

---

# Regex patterns

Concepts are regular expressions.

Prefer explicit boundaries when appropriate:

```python
r'\bpneumonia\b'
```

More flexible concepts can be written directly:

```python
ANAPHYLAXIS = r'\banaphyl(?:axis|actic)\b'
```

Named mappings are recommended when results contain multiple concepts:

```python
terms = {
    'pneumonia': r'\bpneumonia\b',
    'anaphylaxis': r'\banaphyl(?:axis|actic)\b',
}
```

The mapping key becomes the finding label or Struct field name.

# Null behavior

`polars-cnlp` preserves the distinction between concept absence, non-affirmed concepts, and null input.

For contextual Boolean methods:

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

This distinction is intentional and works naturally with Polars' nullable expressions.
