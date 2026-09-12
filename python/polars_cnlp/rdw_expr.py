from pathlib import Path
from typing import Collection

import polars as pl
from polars.plugins import register_plugin_function

_PLUGIN_PATH = Path(__file__).parent


@pl.api.register_expr_namespace('rdw')
class ResearchDataWranglingExpr:

    def __init__(self, expr: pl.Expr):
        self._expr = expr

    def starts_with_any(self, prefixes: Collection[str]) -> pl.Expr:
        """Check if column starts with any of a collection/list of strings.

        Parameters
        ----------
        prefixes
            List of prefixes.

        Returns
        -------
        Expr
            Expression of data type `Bool`.


        Examples
        --------
        >>> df.filter(
        ...     pl.col('dx').cnlp.starts_with_any([123, 456]),
        ... )
        """
        prefixes = list(prefixes)

        if not all(isinstance(prefix, str) for prefix in prefixes):
            raise TypeError('prefixes must contain only strings')

        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='starts_with_any',
            args=[self._expr],
            kwargs={
                'prefixes': prefixes,
            },
            is_elementwise=True,
        )

    def ends_with_any(self, prefixes: Collection[str]) -> pl.Expr:
        """Check whether each string ends with any literal pattern.

        Parameters
        ----------
        expr
            String expression to evaluate.
        prefixes
            Literal suffixes to match. Matching is case-sensitive.

        Returns
        -------
        pl.Expr
            Boolean expression. Null input values remain null.

        Examples
        --------
        >>> df = pl.DataFrame({
        ...     'value': ['running', 'walked', 'quickly', 'run'],
        ... })
        >>> df.filter(
        ...     ends_with_any(
        ...         pl.col('value'),
        ...         ['ing', 'ed'],
        ...     )
        ... )
        shape: (2, 1)
        ┌─────────┐
        │ value   │
        │ ---     │
        │ str     │
        ╞═════════╡
        │ running │
        │ walked  │
        └─────────┘

        Notes
        -----
        Patterns are interpreted literally, not as regular expressions.
        An empty pattern matches every non-null string.
        """
        prefixes = list(prefixes)

        if not all(isinstance(pattern, str) for pattern in prefixes):
            raise TypeError('patterns must contain only strings')

        return register_plugin_function(
            plugin_path=_PLUGIN_PATH,
            function_name='ends_with_any',
            args=self._expr,
            kwargs={'prefixes': prefixes},
            is_elementwise=True,
        )
