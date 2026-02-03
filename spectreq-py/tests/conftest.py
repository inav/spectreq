"""Pytest configuration and fixtures for spectreq tests."""

import pytest
import asyncio
from typing import AsyncGenerator


@pytest.fixture(scope="session")
def event_loop():
    """Create an event loop for the test session."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()


@pytest.fixture
def chrome_profile():
    """Create a Chrome 143 Windows profile for testing."""
    from spectreq import Profile
    return Profile.chrome_143_windows()


@pytest.fixture
def firefox_profile():
    """Create a Firefox 121 Windows profile for testing."""
    from spectreq import Profile
    return Profile.firefox_121_windows()


@pytest.fixture
def safari_profile():
    """Create a Safari 17 macOS profile for testing."""
    from spectreq import Profile
    return Profile.safari_17_macos()


@pytest.fixture
def client(chrome_profile):
    """Create a test client with Chrome profile."""
    from spectreq import Client
    return Client(profile=chrome_profile)


@pytest.fixture
def client_with_headers(chrome_profile):
    """Create a client with custom headers."""
    from spectreq import Client
    return Client(
        profile=chrome_profile,
        headers={
            "X-Test-Header": "test-value",
            "Authorization": "Bearer test-token"
        }
    )


# Test URLs
TEST_URLS = {
    "httpbin": "https://httpbin.org",
    "example": "https://example.com",
}


@pytest.fixture
def httpbin_url():
    """Base URL for httpbin testing."""
    return TEST_URLS["httpbin"]


@pytest.fixture
def example_url():
    """Base URL for example.com."""
    return TEST_URLS["example"]
