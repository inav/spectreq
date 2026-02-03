# Spectre Python Examples

This directory contains example scripts demonstrating how to use the Spectre HTTP client with browser fingerprinting, proxy support, and custom headers.

## Examples

| Example | Description |
|---------|-------------|
| `basic_request.py` | Simple GET request demonstration |
| `post_request.py` | POST request with JSON data |
| `profiles.py` | Display all available browser profiles |
| `streaming.py` | File download and save to disk |
| `full_features.py` | Caching, cookies, and custom headers |

## Running Examples

```bash
# Basic request
python examples/basic_request.py

# POST request
python examples/post_request.py

# View all profiles
python examples/profiles.py

# Download file
python examples/streaming.py

# Full features demo
python examples/full_features.py
```

## Browser Profiles

Spectre supports various browser profiles for fingerprinting:

```python
from spectreq import Profile

# Chrome profiles
profile = Profile.chrome_120_windows()
profile = Profile.chrome_120_macos()
profile = Profile.chrome_120_linux()
profile = Profile.chrome_120_android()
profile = Profile.chrome_131_windows()
profile = Profile.chrome_143_windows()

# Firefox
profile = Profile.firefox_121_windows()

# Safari
profile = Profile.safari_17_macos()

# Edge
profile = Profile.edge_120_windows()
```

## Client Construction

```python
from spectreq import Client, Profile

# Basic usage
client = Client(profile=Profile.chrome_120_windows())

# With proxy
client = Client(
    profile=Profile.chrome_120_windows(),
    proxy="http://proxy.example.com:8080"
)

# With custom headers
client = Client(
    profile=Profile.chrome_120_windows(),
    headers={
        "Authorization": "Bearer token",
        "X-Custom-Header": "value"
    }
)

# With both proxy and headers
client = Client(
    profile=Profile.chrome_120_windows(),
    proxy="http://proxy.example.com:8080",
    headers={"X-API-Key": "secret"}
)
```

## Making Requests

```python
# GET request
resp = await client.get("https://api.example.com/data")
if resp.ok():
    data = resp.json()  # Parse JSON response
    text = resp.text()  # Get response as text
    content = resp.content()  # Get raw bytes

# POST request with JSON body
resp = await client.post(
    "https://api.example.com/create",
    json.dumps(payload).encode()
)

# Other HTTP methods
resp = await client.put(url, body)
resp = await client.patch(url, body)
resp = await client.delete(url)
resp = await client.head(url)
```

## Response Object

```python
resp.status_code      # HTTP status code (200, 404, etc.)
resp.ok()             # True if status is 2xx
resp.text()           # Response body as string
resp.content()        # Response body as bytes
resp.json()           # Parse response as JSON
resp.headers_dict()   # All headers as dict
resp.get_header(name) # Get specific header
resp.wire_size        # Compressed size from network
resp.from_cache       # True if response was from cache
```

## Proxy Support

Set proxy via environment variable or client parameter:

```bash
# Environment variable
export HTTP_PROXY=http://proxy.example.com:8080
export HTTP_PROXY=http://user:pass@proxy.example.com:8080
export HTTP_PROXY=socks5://proxy.example.com:1080
```

```python
# Client parameter
client = Client(
    profile=Profile.chrome_120_windows(),
    proxy="http://proxy.example.com:8080"
)
```

## Async/Await Pattern

All Spectre client methods are async. Use `asyncio.run()` to execute:

```python
import asyncio
from spectreq import Client, Profile

async def main():
    profile = Profile.chrome_120_windows()
    client = Client(profile=profile)
    resp = await client.get("https://example.com")
    print(resp.status_code)

if __name__ == "__main__":
    asyncio.run(main())
```

## Concurrent Requests

Use `asyncio.gather()` for concurrent requests:

```python
async def fetch_multiple():
    profile = Profile.chrome_120_windows()
    client = Client(profile=profile)

    results = await asyncio.gather(
        client.get("https://api.example.com/1"),
        client.get("https://api.example.com/2"),
        client.get("https://api.example.com/3"),
    )
    return results
```
