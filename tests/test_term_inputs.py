import polars as pl
import pytest

import polars_cnlp


def test_affirmed_any_rejects_empty_dict():
    with pytest.raises(ValueError, match='at least one'):
        pl.col('note_text').cnlp.affirmed_any({})


def test_affirmed_any_rejects_empty_list():
    with pytest.raises(ValueError, match='at least one'):
        pl.col('note_text').cnlp.affirmed_any([])


def test_affirmed_all_rejects_empty_dict():
    with pytest.raises(ValueError, match='at least one'):
        pl.col('note_text').cnlp.affirmed_all({})


def test_affirmed_all_rejects_empty_list():
    with pytest.raises(ValueError, match='at least one'):
        pl.col('note_text').cnlp.affirmed_all([])


def test_string_is_not_treated_as_predicate_term_sequence():
    with pytest.raises(TypeError, match='mapping.*sequence'):
        pl.col('note_text').cnlp.affirmed_any(r'\bpneumonia\b')


def test_non_string_sequence_item_is_rejected():
    with pytest.raises(TypeError, match='index 1'):
        pl.col('note_text').cnlp.affirmed_all([
            r'\bpneumonia\b',
            123,
        ])


def test_empty_sequence_pattern_is_rejected():
    with pytest.raises(ValueError, match='index 1'):
        pl.col('note_text').cnlp.affirmed_any([
            r'\bpneumonia\b',
            '',
        ])


def test_affirmed_each_requires_mapping():
    with pytest.raises(TypeError, match='mapping'):
        pl.col('note_text').cnlp.affirmed_each([
            r'\bpneumonia\b',
            r'\banaphylaxis\b',
        ])


def test_count_all_requires_mapping():
    with pytest.raises(TypeError, match='mapping'):
        pl.col('note_text').cnlp.count_all([
            r'\bpneumonia\b',
            r'\banaphylaxis\b',
        ])


def test_find_best_accepts_single_regex():
    expr = pl.col('note_text').cnlp.find_best(
        r'\bpneumonia\b',
    )

    assert isinstance(expr, pl.Expr)


def test_find_all_accepts_single_regex():
    expr = pl.col('note_text').cnlp.find_all(
        r'\bpneumonia\b',
    )

    assert isinstance(expr, pl.Expr)


def test_find_best_rejects_list():
    with pytest.raises(TypeError, match='regular expression.*mapping'):
        pl.col('note_text').cnlp.find_best([
            r'\bpneumonia\b',
        ])


def test_find_all_rejects_list():
    with pytest.raises(TypeError, match='regular expression.*mapping'):
        pl.col('note_text').cnlp.find_all([
            r'\bpneumonia\b',
        ])


def test_find_rejects_empty_regex():
    with pytest.raises(ValueError, match='must not be empty'):
        pl.col('note_text').cnlp.find_best('')


def test_find_rejects_empty_mapping():
    with pytest.raises(ValueError, match='at least one'):
        pl.col('note_text').cnlp.find_all({})


def test_empty_mapping_label_is_rejected():
    with pytest.raises(ValueError, match='labels'):
        pl.col('note_text').cnlp.affirmed_each({
            '': r'\bpneumonia\b',
        })


def test_empty_mapping_pattern_is_rejected():
    with pytest.raises(ValueError, match='must not be empty'):
        pl.col('note_text').cnlp.count_all({
            'pneumonia': '',
        })


def test_invalid_find_regex_from_mapping_identifies_label():
    df = pl.DataFrame(
        {'note_text': ['Pneumonia.']},
    )

    with pytest.raises(pl.exceptions.ComputeError, match='bad_pattern'):
        df.select(
            pl.col('note_text').cnlp.find_best({
                'bad_pattern': r'[invalid',
            }),
        )


def test_invalid_find_single_regex_raises_compute_error():
    df = pl.DataFrame(
        {'note_text': ['Pneumonia.']},
    )

    with pytest.raises(pl.exceptions.ComputeError, match='invalid regex'):
        df.select(
            pl.col('note_text').cnlp.find_all(
                r'[invalid',
            ),
        )
