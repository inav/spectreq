import pytest
import multiprocessing
from spectreq import CookieJar

class TestCookieJar:
    """Test CookieJar functionality."""

    def test_new_jar(self):
        """Test creating a new cookie jar."""
        jar = CookieJar()
        assert len(jar) == 0
        assert jar.is_empty()

    def test_set_get_cookies(self):
        """Test setting and getting cookies."""
        jar = CookieJar()
        url = "https://example.com"
        
        # Set cookies
        jar.set_cookies(url, ["session=123; Path=/", "user=test"])
        assert len(jar) == 2
        assert not jar.is_empty()
        
        # Get cookies
        cookies = jar.get_cookie_value(url)
        assert cookies is not None
        assert "session=123" in cookies
        assert "user=test" in cookies

    def test_domain_matching(self):
        """Test cookie domain matching."""
        jar = CookieJar()
        
        # Set cookie for example.com
        jar.set_cookies("https://example.com", ["session=123; Domain=example.com"])
        
        # Should match example.com
        assert jar.get_cookie_value("https://example.com") is not None
        
        # Should NOT match google.com
        assert jar.get_cookie_value("https://google.com") is None

    def test_clear(self):
        """Test clearing cookies."""
        jar = CookieJar()
        jar.set_cookies("https://example.com", ["session=123"])
        assert len(jar) == 1
        
        jar.clear()
        assert len(jar) == 0

    def test_remove_for_domain(self):
        """Test removing cookies for a domain."""
        jar = CookieJar()
        jar.set_cookies("https://example.com", ["session=123; Domain=example.com"])
        jar.set_cookies("https://google.com", ["auth=xyz; Domain=google.com"])
        
        assert len(jar) == 2
        
        jar.remove_for_domain("example.com")
        assert len(jar) == 1
        assert jar.get_cookie_value("https://example.com") is None
        assert jar.get_cookie_value("https://google.com") is not None
