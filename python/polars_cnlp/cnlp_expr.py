from collections.abc import Mapping, Sequence
from pathlib import Path

import polars as pl
from polars.plugins import register_plugin_function

from polars_cnlp.algorithms import AlgorithmLike, algorithm_kwargs
from polars_cnlp.mapping import TermPatterns, FindPatterns, mapping_terms_kwargs, predicate_terms_kwargs, \
    find_terms_kwargs

_PLUGIN_PATH = Path(__file__).parent


@pl.api.register_expr_namespace('cnlp')
class ClinicalNlpExpr:
    """Clinical NLP expressions."""

    def __init__(self, expr: pl.Expr):
        self._expr = expr

    def count(self, term: str) -> pl.Expr:
        """Count occurrences of a concept.

        Parameters
        ----------
        term
            Regular expression defining the concept.

        Returns
        -------
        Expr
            Expression of data type `UInt32`.

        See Also
        --------
        count_all

        Examples
        --------
        >>> df.select(
        ...     pl.col('note_text').cnlp.count(r'\\bpneumonia\\b'),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='count_concept',
            args=[self._expr],
            kwargs={'pattern': term},
            is_elementwise=True,
        )

    def count_all(self, terms: Mapping[str, str]) -> pl.Expr:
        """
        Count occurrences of each named concept.

        Parameters
        ----------
        terms
            Mapping from output field name to regular expression.

        Returns
        -------
        Expr
            Struct expression with one `UInt32` field per concept.

        See Also
        --------
        count

        Examples
        --------
        >>> terms = {
        ...     'pneumonia': r'\\bpneumonia\\b',
        ...     'asthma': r'\\basthma\\b',
        ... }
        >>> df.select(
        ...     pl.col('note_text').cnlp.count_all(terms).alias('counts'),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='count_all_concepts',
            args=[self._expr],
            kwargs=mapping_terms_kwargs(terms),
            is_elementwise=True,
        )

    def affirmed(
            self,
            term: str,
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """ Check whether a concept is affirmed.

        Returns `True` when at least one occurrence is affirmed, `False`
        when the concept occurs but no occurrence is affirmed, and null
        when the concept is not mentioned.

        Parameters
        ----------
        term
            Regular expression defining the concept.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            Expression of data type `Boolean`.

        See Also
        --------
        affirmed_any, affirmed_all, affirmed_each

        Examples
        --------
        >>> df.select(
        ...     pl.col('note_text').cnlp.affirmed(r'\\bpneumonia\\b'),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='affirmed_concept',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                {'pattern': term},
                algorithm,
            ),
            is_elementwise=True,
        )

    def affirmed_any(
            self,
            terms: TermPatterns,
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """
        Check whether any requested concept is affirmed.

        Parameters
        ----------
        terms
            Regular expressions supplied either as a sequence or as a
            mapping from labels to regular expressions. Labels are used
            only for diagnostics.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            Boolean expression. Returns `True` when at least one concept
            is affirmed, `False` when at least one concept is mentioned
            but none are affirmed, and null when none are mentioned.

        See Also
        --------
        affirmed, affirmed_all, affirmed_each

        Examples
        --------
        >>> terms = [
        ...     r'\\bpneumonia\\b',
        ...     r'\\banaphylaxis\\b',
        ... ]
        >>> df.filter(
        ...     pl.col('note_text').cnlp.affirmed_any(terms),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='affirmed_any_concepts',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                predicate_terms_kwargs(terms),
                algorithm,
            ),
            is_elementwise=True,
        )

    def affirmed_all(
            self,
            terms: TermPatterns,
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """
        Check whether all requested concepts are affirmed.

        Parameters
        ----------
        terms
            Regular expressions supplied either as a sequence or as a
            mapping from labels to regular expressions. Labels are used
            only for diagnostics.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            Boolean expression. Returns `True` when every concept is
            affirmed, `False` when any mentioned concept is non-affirmed,
            and null when no concept is non-affirmed but one or more
            concepts are not mentioned.

        See Also
        --------
        affirmed, affirmed_any, affirmed_each

        Examples
        --------
        >>> terms = [
        ...     r'\\bpneumonia\\b',
        ...     r'\\banaphylaxis\\b',
        ... ]
        >>> df.filter(
        ...     pl.col('note_text').cnlp.affirmed_all(terms),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='affirmed_all_concepts',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                predicate_terms_kwargs(terms),
                algorithm,
            ),
            is_elementwise=True,
        )

    def affirmed_each(
            self,
            terms: Mapping[str, str],
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """
        Check affirmation status for each named concept.

        Parameters
        ----------
        terms
            Mapping from output field name to regular expression.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            Struct expression with one Boolean field per concept. A field
            is `True` when the concept is affirmed, `False` when it is
            mentioned but non-affirmed, and null when it is not mentioned.

        See Also
        --------
        affirmed, affirmed_any, affirmed_all

        Examples
        --------
        >>> terms = {
        ...     'pneumonia': r'\\bpneumonia\\b',
        ...     'anaphylaxis': r'\\banaphylaxis\\b',
        ... }
        >>> df.select(
        ...     pl.col('note_text').cnlp.affirmed_each(terms).alias('status'),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='affirmed_each_concepts',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                mapping_terms_kwargs(terms),
                algorithm,
            ),
            is_elementwise=True,
        )

    def find_best(
            self,
            terms: FindPatterns,
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """
        Return the highest-ranked contextualized finding.

        Findings are ranked by experiencer, temporality, assertion, and
        then text position. Patient findings are preferred over other
        experiencers, current over historical over hypothetical, and
        affirmed over possible over negated.

        Parameters
        ----------
        terms
            A regular expression or mapping from finding label to regular
            expression. A single regular expression produces a null
            `label` field.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            Nullable Struct expression with fields `label`, `start`,
            `end`, `assertion`, `temporality`, and `experiencer`.
            `start` and `end` are UTF-8 byte offsets.

        See Also
        --------
        find_all, affirmed

        Examples
        --------
        >>> df.select(
        ...     pl.col('note_text')
        ...     .cnlp.find_best(r'\\bpneumonia\\b')
        ...     .alias('finding'),
        ... )
        """

        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='find_best_concept',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                find_terms_kwargs(terms),
                algorithm,
            ),
            is_elementwise=True,
        )

    def find_all(
            self,
            terms: FindPatterns,
            *,
            algorithm: AlgorithmLike = 'context',
    ) -> pl.Expr:
        """
        Return all contextualized findings.

        Findings are returned in text order.

        Parameters
        ----------
        terms
            A regular expression or mapping from finding label to regular
            expression. A single regular expression produces null values
            in the `label` field.
        algorithm
            Context algorithm. Use `'context'`, `'negex'`, or an
            `Algorithm` instance.

        Returns
        -------
        Expr
            List of Structs with fields `label`, `start`, `end`,
            `assertion`, `temporality`, and `experiencer`. `start` and
            `end` are UTF-8 byte offsets. A non-null note with no findings
            returns an empty list; null input returns null.

        See Also
        --------
        find_best, affirmed_each

        Examples
        --------
        >>> terms = {
        ...     'pneumonia': r'\\bpneumonia\\b',
        ...     'anaphylaxis': r'\\banaphylaxis\\b',
        ... }
        >>> df.select(
        ...     pl.col('note_text').cnlp.find_all(terms).alias('findings'),
        ... )
        """
        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='find_all_concepts',
            args=[self._expr],
            kwargs=algorithm_kwargs(
                find_terms_kwargs(terms),
                algorithm,
            ),
            is_elementwise=True,
        )
