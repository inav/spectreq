# spectreq

Python bindings for the Spectre HTTP client, designed for browser impersonation and high-efficiency web scraping.

## Features

- **Browser Impersonation**: Mimic Chrome, Firefox, Safari, and Edge fingerprints.
- **Async API**: Built on `tokio` and `pyo3-asyncio` for high performance.
- **TLS/HTTP2 Fingerprinting**: Uses `boring` (BoringSSL) for realistic TLS handshakes.
- **Profile Randomization**: Easily rotate user agents and fingerprints.

## Installation

```bash
pip install spectreq-py
```

## Usage

### Basic Request

```python
import asyncio
from spectreq import Client, Profile

async def main():
    # various profile options
    profile = Profile.chrome_143_windows()
    # or
    # profile = Profile.random_desktop()
    
    client = Client(profile)
    
    response = await client.get("https://httpbin.org/get")
    print(response.status_code)
    print(response.json())
    
    # Access headers
    print(response.headers_dict())

asyncio.run(main())
```

### Authentication

Spectre supports various authentication helpers.

```python
from spectreq import Client, Profile, BearerToken, BasicAuth

async def main():
    profile = Profile.random_desktop()
    
    # Bearer Token
    token = BearerToken("my-secret-token")
    client = Client(profile, headers=token.headers())
    
    # Basic Auth
    auth = BasicAuth("username", "password")
    client_auth = Client(profile, headers=auth.headers())
    
    await client.get("https://api.example.com/protected")

asyncio.run(main())
```

### Custom Headers & Proxies

```python
client = Client(
    Profile.chrome_143_windows(),
    proxy="socks5://127.0.0.1:1080",
    headers={"X-Custom-ID": "12345"}
)
```

## Response Object

The `Response` object exposes:
- `status_code`: HTTP status (int)
- `text()`: Body as string
- `content()`: Body as bytes
- `json()`: Parse body as JSON
- `headers_dict()`: All headers
- `get_header(name)`: Single header lookup
- `wire_size`: Compressed size of the response
- `ok()`: True if status is 2xx

