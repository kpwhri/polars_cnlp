from typing import Mapping, Sequence

TermPatterns = Mapping[str, str] | Sequence[str]
FindPatterns = str | Mapping[str, str]


def mapping_terms_kwargs(terms: Mapping[str, str]) -> dict[str, object]:
    if not isinstance(terms, Mapping):
        raise TypeError('terms must be a mapping of label to regex pattern')

    if not terms:
        raise ValueError('terms must contain at least one regular expression')

    normalized = []

    for label, pattern in terms.items():
        if not isinstance(label, str):
            raise TypeError('term labels must be strings')

        if not label.strip():
            raise ValueError('term labels must not be empty')

        if not isinstance(pattern, str):
            raise TypeError(f'regex for term {label!r} must be a string')

        if not pattern:
            raise ValueError(f'regex for term {label!r} must not be empty')

        normalized.append({
            'label': label,
            'pattern': pattern,
        })

    return {'terms': normalized}


def predicate_terms_kwargs(terms: TermPatterns) -> dict[str, object]:
    if isinstance(terms, Mapping):
        return mapping_terms_kwargs(terms)

    if isinstance(terms, (str, bytes)) or not isinstance(terms, Sequence):
        raise TypeError(
            'terms must be a mapping of label to regex pattern '
            'or a sequence of regex patterns'
        )

    if not terms:
        raise ValueError('terms must contain at least one regular expression')

    normalized = []

    for index, pattern in enumerate(terms):
        if not isinstance(pattern, str):
            raise TypeError(f'regex at index {index} must be a string')

        if not pattern:
            raise ValueError(f'regex at index {index} must not be empty')

        normalized.append({
            'label': f'term_{index}',
            'pattern': pattern,
        })

    return {'terms': normalized}


def find_terms_kwargs(terms: FindPatterns) -> dict[str, object]:
    if isinstance(terms, str):
        if not terms:
            raise ValueError('regular expression must not be empty')

        return {
            'terms': [{
                'label': None,
                'pattern': terms,
            }],
        }

    if not isinstance(terms, Mapping):
        raise TypeError(
            'terms must be a regular expression or a mapping '
            'of label to regex pattern'
        )

    return mapping_terms_kwargs(terms)
