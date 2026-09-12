import re
from collections.abc import Iterable, Iterator, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

Assertion = Literal['affirmed', 'possible', 'negated']
Temporality = Literal['current', 'historical', 'hypothetical']
Experiencer = Literal['patient', 'other']
Direction = Literal['forward', 'backward', 'bidirectional']


@dataclass(frozen=True)
class ContextEffect:
    """Define changes to one or more context dimensions.

    Parameters
    ----------
    assertion
        Assertion value assigned by the rule.
    temporality
        Temporality value assigned by the rule.
    experiencer
        Experiencer value assigned by the rule.
    """

    assertion: Assertion | None = None
    temporality: Temporality | None = None
    experiencer: Experiencer | None = None

    def __post_init__(self):
        valid_assertions = {None, 'affirmed', 'possible', 'negated'}
        valid_temporalities = {None, 'current', 'historical', 'hypothetical'}
        valid_experiencers = {None, 'patient', 'other'}

        if self.assertion not in valid_assertions:
            raise ValueError(f'Invalid assertion: {self.assertion!r}')

        if self.temporality not in valid_temporalities:
            raise ValueError(f'Invalid temporality: {self.temporality!r}')

        if self.experiencer not in valid_experiencers:
            raise ValueError(f'Invalid experiencer: {self.experiencer!r}')

        if self.assertion is None and self.temporality is None and self.experiencer is None:
            raise ValueError('Context effect must modify at least one context dimension')

    def _to_spec(self) -> dict:
        return {'assertion': self.assertion, 'temporality': self.temporality, 'experiencer': self.experiencer}


@dataclass(frozen=True)
class ContextRule:
    """Define a contextual modifier rule.

    Parameters
    ----------
    pattern
        Regular expression matching the modifier.
    direction
        Direction in which the modifier applies.
    effect
        Context values assigned by the modifier.
    max_scope
        Maximum number of intervening tokens. `None` uses the algorithm's
        normal scope.
    max_targets
        Maximum number of targets modified by one rule occurrence.
    terminated_by
        Other context effects which terminate this modifier's scope.
    """
    pattern: str
    direction: Direction
    effect: ContextEffect
    max_scope: int | None = None
    max_targets: int | None = None
    terminated_by: Sequence[ContextEffect] = ()

    def __post_init__(self):
        if not self.pattern:
            raise ValueError('pattern must not be empty')

        if self.direction not in {'forward', 'backward', 'bidirectional'}:
            raise ValueError(f'Invalid direction: {self.direction!r}')

        if self.max_scope is not None and self.max_scope < 0:
            raise ValueError('max_scope must be >= 0 or None')

        if self.max_targets is not None and self.max_targets < 1:
            raise ValueError('max_targets must be >= 1 or None')

        object.__setattr__(
            self,
            'terminated_by',
            tuple(self.terminated_by),
        )

    def _to_spec(self) -> dict:
        return {
            'kind': 'context',
            'pattern': self.pattern,
            'direction': self.direction,
            'effect': self.effect._to_spec(),
            'max_scope': self.max_scope,
            'max_targets': self.max_targets,
            'terminated_by': [effect._to_spec() for effect in self.terminated_by],
        }


@dataclass(frozen=True)
class TerminateRule:
    """Define a modifier-scope termination rule.

    Parameters
    ----------
    pattern
        Regular expression matching the termination phrase.
    effects
        Context effects terminated by this rule. An empty sequence means all
        context effects.
    """

    pattern: str
    effects: Sequence[ContextEffect] = ()

    def __post_init__(self):
        if not self.pattern:
            raise ValueError('Pattern must not be empty')

        object.__setattr__(self, 'effects', tuple(self.effects), )

    def _to_spec(self) -> dict:
        return {
            'kind': 'terminate',
            'pattern': self.pattern,
            'effects': [effect._to_spec() for effect in self.effects],
        }


@dataclass(frozen=True)
class PseudoRule:
    """Define a pseudo-trigger rule.

    Parameters
    ----------
    pattern
        Regular expression whose span suppresses overlapping modifiers.
    effects
        Context effects suppressed by this rule. An empty sequence means all
        effects.
    """

    pattern: str
    effects: Sequence[ContextEffect] = ()

    def __post_init__(self):
        if not self.pattern:
            raise ValueError('Pattern must not be empty')

        object.__setattr__(self, 'effects', tuple(self.effects))

    def _to_spec(self) -> dict:
        return {
            'kind': 'pseudo',
            'pattern': self.pattern,
            'effects': [effect._to_spec() for effect in self.effects],
        }


Rule = ContextRule | TerminateRule | PseudoRule


@dataclass(frozen=True)
class RuleSet:
    """Immutable collection of rules defining an algorithm's keyword set."""

    rules: Sequence[Rule]

    def __post_init__(self):
        object.__setattr__(
            self,
            'rules',
            tuple(self.rules),
        )

    def __iter__(self) -> Iterator[Rule]:
        return iter(self.rules)

    def __len__(self) -> int:
        return len(self.rules)

    def extend(self, rules: Iterable[Rule]) -> 'RuleSet':
        """Return a new rule set containing the additional rules."""
        return RuleSet((*self.rules, *tuple(rules)))

    def _to_spec(self) -> list[dict]:
        return [rule._to_spec() for rule in self.rules]

    @classmethod
    def from_negex_lines(cls, lines: Iterable[str], *, include_possible: bool = True) -> 'RuleSet':
        """Create a rule set from standard NegEx trigger-file lines.

        Supported tags are PREN, POST, PREP, POSP, PSEU, and CONJ.
        """
        rules: list[Rule] = []

        negated = ContextEffect(assertion='negated')
        possible = ContextEffect(assertion='possible')

        assertion_effects = ((negated, possible) if include_possible else (negated,))

        line_pattern = re.compile(r'^\s*(.*?)\s+\[(PREN|POST|PREP|POSP|PSEU|CONJ)\]\s*$')

        for line_number, line in enumerate(lines, start=1):
            stripped = line.strip()

            if not stripped or stripped.startswith('#'):
                continue

            match = line_pattern.fullmatch(stripped)

            if match is None:
                raise ValueError(f'Invalid NegEx rule at line {line_number}: {stripped!r}')

            literal, tag = match.groups()
            pattern = _literal_pattern(literal)

            if tag == 'PREN':
                rules.append(ContextRule(pattern=pattern, direction='forward', effect=negated))
            elif tag == 'POST':
                rules.append(ContextRule(pattern=pattern, direction='backward', effect=negated))
            elif tag == 'PREP':
                if include_possible:
                    rules.append(ContextRule(pattern=pattern, direction='forward', effect=possible))
            elif tag == 'POSP':
                if include_possible:
                    rules.append(ContextRule(pattern=pattern, direction='backward', effect=possible))
            elif tag == 'PSEU':
                rules.append(PseudoRule(pattern=pattern, effects=assertion_effects))
            elif tag == 'CONJ':
                rules.append(TerminateRule(pattern=pattern, effects=assertion_effects))

        return cls(rules)

    @classmethod
    def from_negex_file(cls, path: str | Path, *, include_possible: bool = True) -> 'RuleSet':
        """Create a rule set from a NegEx trigger file."""
        with Path(path).open(encoding='utf8') as file:
            return cls.from_negex_lines(file, include_possible=include_possible)


class Algorithm:
    """Base class for serializable CNLP algorithm specifications."""

    def _to_spec(self) -> dict:
        raise NotImplementedError


@dataclass(frozen=True)
class ConText(Algorithm):
    """Use ConText mechanics.

    Parameters
    ----------
    rules
        Replacement rule set. `None` uses the package default ConText rules.
    """

    rules: RuleSet | None = None

    def _to_spec(self) -> dict:
        return {
            'type': 'context',
            'rules': (
                None
                if self.rules is None
                else self.rules._to_spec()
            ),
        }


@dataclass(frozen=True)
class NegEx(Algorithm):
    """Use NegEx mechanics.

    Parameters
    ----------
    rules
        Replacement rule set. `None` uses the package default NegEx rules.
    window
        Number of directional target positions. The published default is six,
        equivalent to zero through five intervening terms.
    propagate_same_concept
        Whether a negated/possible occurrence propagates to other occurrences
        of the same concept in the sentence.
    """

    rules: RuleSet | None = None
    window: int = 6
    propagate_same_concept: bool = True

    def __post_init__(self):
        if self.window < 1:
            raise ValueError('window must be at least 1')

    def _to_spec(self) -> dict:
        return {
            'type': 'negex',
            'rules': (
                None
                if self.rules is None
                else self.rules._to_spec()
            ),
            'window': self.window,
            'propagate_same_concept': self.propagate_same_concept,
        }


@dataclass(frozen=True)
class RuleBased(Algorithm):
    """Define a custom rule-based algorithm.

    Parameters
    ----------
    rules
        Rules defining modifier, pseudo, and termination behavior.
    window
        Optional fixed directional target window. `None` uses rule-defined
        ConText-style sentence scope.
    """

    rules: RuleSet
    window: int | None = None

    def __post_init__(self):
        if self.window is not None and self.window < 1:
            raise ValueError('window must be at least 1 or None')

    def _to_spec(self) -> dict:
        return {
            'type': 'rule_based',
            'rules': self.rules._to_spec(),
            'window': self.window,
        }


AlgorithmLike = Literal['context', 'negex'] | Algorithm


def algorithm_kwargs(kwargs: dict, algorithm: AlgorithmLike, ) -> dict:
    """Add a serialized context algorithm to plugin keyword arguments."""
    if algorithm == 'context':
        spec = ConText()._to_spec()
    elif algorithm == 'negex':
        spec = NegEx()._to_spec()
    elif isinstance(algorithm, Algorithm):
        spec = algorithm._to_spec()
    else:
        raise ValueError('Algorithm must be "context", "negex", or an Algorithm instance')

    return {
        **kwargs,
        'algorithm': spec,
    }


def _literal_pattern(literal: str) -> str:
    parts = literal.split()

    if not parts:
        raise ValueError('NegEx trigger must not be empty')

    escaped = [
        re.escape(part)
        for part in parts
    ]

    return r'\b' + r'\W+'.join(escaped) + r'\b'
