"""Tests for HTTP client functionality."""

import pytest
from spectreq import Client, Profile


class TestClientCreation:
    """Tests for client initialization."""

    def test_create_client_basic(self, chrome_profile):
        """Test basic client creation."""
        client = Client(profile=chrome_profile)
        assert client is not None

    def test_create_client_with_headers(self, chrome_profile):
        """Test client creation with custom headers."""
        headers = {
            "X-Custom-Header": "test-value",
            "Authorization": "Bearer token123"
        }
        client = Client(profile=chrome_profile, headers=headers)
        assert client is not None
        assert client.headers.get("X-Custom-Header") == "test-value"

    def test_create_client_with_proxy(self, chrome_profile):
        """Test client creation with proxy."""
        client = Client(
            profile=chrome_profile,
            proxy="http://proxy.example.com:8080"
        )
        assert client is not None
        assert client.proxy == "http://proxy.example.com:8080"

    def test_create_client_with_socks_proxy(self, chrome_profile):
        """Test client creation with SOCKS5 proxy."""
        client = Client(
            profile=chrome_profile,
            proxy="socks5://127.0.0.1:1080"
        )
        assert client.proxy == "socks5://127.0.0.1:1080"

    def test_client_repr(self, chrome_profile):
        """Test client string representation."""
        client = Client(profile=chrome_profile)
        repr_str = repr(client)
        assert "Client" in repr_str


class TestClientProperties:
    """Tests for client property access."""

    def test_get_proxy_none(self, chrome_profile):
        """Test getting proxy when none set."""
        client = Client(profile=chrome_profile)
        assert client.proxy is None

    def test_get_headers_empty(self, chrome_profile):
        """Test getting headers when none set."""
        client = Client(profile=chrome_profile)
        assert isinstance(client.headers, dict)

    def test_get_custom_headers(self, chrome_profile):
        """Test getting custom headers."""
        client = Client(
            profile=chrome_profile,
            headers={"X-Test": "value"}
        )
        assert client.headers["X-Test"] == "value"


@pytest.mark.asyncio
class TestHTTPMethods:
    """Tests for HTTP methods."""

    async def test_get_request(self, client, httpbin_url):
        """Test basic GET request."""
        response = await client.get(f"{httpbin_url}/get")
        assert response.status_code == 200
        assert response.ok()

    async def test_get_request_with_json(self, client, httpbin_url):
        """Test GET request and parse JSON response."""
        response = await client.get(f"{httpbin_url}/get")
        data = response.json()
        assert isinstance(data, dict)
        assert "headers" in data

    async def test_post_request(self, client, httpbin_url):
        """Test POST request with body."""
        body = b'{"key": "value"}'
        response = await client.post(f"{httpbin_url}/post", body=body)
        assert response.status_code == 200

    async def test_post_request_empty_body(self, client, httpbin_url):
        """Test POST request without body."""
        response = await client.post(f"{httpbin_url}/post")
        assert response.status_code == 200

    async def test_put_request(self, client, httpbin_url):
        """Test PUT request."""
        body = b"update data"
        response = await client.put(f"{httpbin_url}/put", body=body)
        assert response.status_code == 200

    async def test_patch_request(self, client, httpbin_url):
        """Test PATCH request."""
        body = b'{"patch": "data"}'
        response = await client.patch(f"{httpbin_url}/patch", body=body)
        assert response.status_code == 200

    async def test_delete_request(self, client, httpbin_url):
        """Test DELETE request."""
        response = await client.delete(f"{httpbin_url}/delete")
        assert response.status_code == 200

    async def test_head_request(self, client, httpbin_url):
        """Test HEAD request."""
        response = await client.head(f"{httpbin_url}/get")
        assert response.status_code == 200
        # HEAD response should have empty body
        assert len(response.content()) == 0


@pytest.mark.asyncio
class TestResponseHandling:
    """Tests for response parsing and handling."""

    async def test_response_text(self, client, httpbin_url):
        """Test getting response as text."""
        response = await client.get(f"{httpbin_url}/get")
        text = response.text()
        assert isinstance(text, str)
        assert len(text) > 0

    async def test_response_content(self, client, httpbin_url):
        """Test getting response as bytes."""
        response = await client.get(f"{httpbin_url}/get")
        content = response.content()
        assert isinstance(content, bytes)
        assert len(content) > 0

    async def test_response_json(self, client, httpbin_url):
        """Test parsing JSON response."""
        response = await client.get(f"{httpbin_url}/json")
        data = response.json()
        assert isinstance(data, dict)

    async def test_response_headers(self, client, httpbin_url):
        """Test getting response headers."""
        response = await client.get(f"{httpbin_url}/get")
        headers = response.headers_dict()
        assert isinstance(headers, dict)
        assert "content-type" in headers or "Content-Type" in headers

    async def test_get_specific_header(self, client, httpbin_url):
        """Test getting a specific header."""
        response = await client.get(f"{httpbin_url}/get")
        content_type = response.get_header("content-type")
        assert content_type is not None
        assert "json" in content_type.lower()

    async def test_wire_size(self, client, httpbin_url):
        """Test wire size tracking."""
        response = await client.get(f"{httpbin_url}/get")
        assert response.wire_size >= 0

    async def test_response_ok(self, client, httpbin_url):
        """Test response ok() method."""
        response = await client.get(f"{httpbin_url}/get")
        assert response.ok() is True

        response_404 = await client.get(f"{httpbin_url}/status/404")
        assert response_404.ok() is False

    async def test_response_repr(self, client, httpbin_url):
        """Test response string representation."""
        response = await client.get(f"{httpbin_url}/get")
        repr_str = repr(response)
        assert "Response" in repr_str
        assert "200" in repr_str


@pytest.mark.asyncio
class TestStatusCodes:
    """Tests for various HTTP status codes."""

    async def test_status_200(self, client, httpbin_url):
        """Test 200 OK status."""
        response = await client.get(f"{httpbin_url}/status/200")
        assert response.status_code == 200

    async def test_status_201(self, client, httpbin_url):
        """Test 201 Created status."""
        response = await client.get(f"{httpbin_url}/status/201")
        assert response.status_code == 201

    async def test_status_301(self, client, httpbin_url):
        """Test 301 redirect (auto-follow)."""
        response = await client.get(f"{httpbin_url}/status/301")
        # httpbin returns 301 without Location header, so it stays 301
        assert response.status_code in [200, 301]

    async def test_status_404(self, client, httpbin_url):
        """Test 404 Not Found status."""
        response = await client.get(f"{httpbin_url}/status/404")
        assert response.status_code == 404
        assert not response.ok()

    async def test_status_500(self, client, httpbin_url):
        """Test 500 Internal Server Error status."""
        response = await client.get(f"{httpbin_url}/status/500")
        assert response.status_code == 500
        assert not response.ok()


@pytest.mark.asyncio
class TestCustomHeaders:
    """Tests for custom header handling."""

    async def test_custom_headers_sent(self, client_with_headers, httpbin_url):
        """Test that custom headers are sent with requests."""
        response = await client_with_headers.get(f"{httpbin_url}/headers")
        data = response.json()
        headers = data.get("headers", {})
        
        # Header names might be normalized to different cases
        assert any(
            k.lower() == "x-test-header" 
            for k in headers.keys()
        )

    async def test_authorization_header(self, client_with_headers, httpbin_url):
        """Test that authorization header is sent."""
        response = await client_with_headers.get(f"{httpbin_url}/headers")
        data = response.json()
        headers = data.get("headers", {})
        
        auth_header = headers.get("Authorization") or headers.get("authorization")
        assert auth_header == "Bearer test-token"


@pytest.mark.asyncio
class TestClientFeatures:
    """Tests for advanced client features."""

    async def test_response_timing(self, client, httpbin_url):
        """Test that response includes timing metrics."""
        response = await client.get(f"{httpbin_url}/get")
        timing = response.timing
        
        # Timing Metrics should be floats >= 0
        assert isinstance(timing.dns_lookup, float)
        assert timing.dns_lookup >= 0
        assert isinstance(timing.total, float)
        assert timing.total > 0
        
        # Check connection time components
        assert timing.tcp_connect >= 0
        assert timing.tls_handshake >= 0
        assert timing.ttfb >= 0

    async def test_client_cookie_jar(self, client, httpbin_url):
        """Test accessing and using the client's cookie jar."""
        jar = client.cookie_jar()
        assert jar is not None
        assert jar.len() == 0
        
        # Make a request that sets a cookie
        await client.get(f"{httpbin_url}/cookies/set/testcookie/testvalue")
        
        # Verify cookie is in the jar
        cookies = jar.get_cookie_value(f"{httpbin_url}")
        assert "testcookie=testvalue" in cookies
        assert jar.len() > 0

