import pytest


@pytest.fixture
def terms():
    return {
        'pneumonia': r'\bpneumonia\b',
        'anaphylaxis': r'\banaphylaxis\b',
    }


@pytest.fixture
def patterns():
    return [
        r'\bpneumonia\b',
        r'\banaphylaxis\b',
    ]
