"""Shared pytest fixtures for CCF journey tests."""
import pytest


@pytest.fixture(scope="session")
def ccf_core():
    """Session-scoped ccf_core import. Skip entire session if not installed."""
    try:
        import ccf_core as _ccf_core
        return _ccf_core
    except ImportError:
        pytest.skip("ccf_core not installed. Run: pip install ccf-core")
