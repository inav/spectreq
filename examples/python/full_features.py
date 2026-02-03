"""
Full-featured client example for Spectre Python bindings

This example demonstrates caching, cookies, custom headers, and proxy configuration.

Run with: python examples/full_features.py
"""

import asyncio
from spectreq import Profile, Client


async def main():
    # Create profile
    profile = Profile.chrome_143_windows()

    print("Creating client with custom configuration:")
    print("  Profile: Chrome 143 on Windows")
    print("  Custom Headers: X-Custom-Header, X-API-Key")
    print()

    # Create client with custom headers
    client = Client(
        profile,
        headers={"X-Custom-Header": "custom-value", "X-API-Key": "secret-key-12345"},
    )

    # First request - sets a cookie
    print("=" * 60)
    print("Request 1: Setting a cookie")
    print("=" * 60)

    response = await client.get("https://httpbin.org/cookies/set?name=value")
    print(f"Status: {response.status_code}")
    print(f"Body: {response.text()[:200]}...")
    print()

    # Second request - cookies should be included
    print("=" * 60)
    print("Request 2: Checking cookies (should include 'name=value')")
    print("=" * 60)

    response = await client.get("https://httpbin.org/cookies")
    print(f"Status: {response.status_code}")
    print(f"From Cache: {response.from_cache}")
    print("Cookies Response:")
    print(response.text())
    print()

    # Third request - may use cache
    print("=" * 60)
    print("Request 3: Same endpoint (may use cache)")
    print("=" * 60)

    response = await client.get("https://httpbin.org/cookies")
    print(f"Status: {response.status_code}")
    print(f"From Cache: {response.from_cache}")
    print()

    # Test custom headers
    print("=" * 60)
    print("Request 4: Checking custom headers")
    print("=" * 60)

    response = await client.get("https://httpbin.org/headers")
    print(f"Status: {response.status_code}")
    print("Response (showing custom headers):")
    print(response.text())
    print()

    # Print client configuration
    print("=" * 60)
    print("Client Configuration")
    print("=" * 60)

    print(f"Proxy: {client.proxy}")
    print(f"Headers: {client.headers}")
    print()


if __name__ == "__main__":
    asyncio.run(main())
