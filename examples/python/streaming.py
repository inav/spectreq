"""
File download example for Spectre Python bindings

This example demonstrates how to download files and save them to disk.

Run with: python examples/streaming.py
"""

import asyncio
from spectreq import Profile, Client


async def main():
    # Create client
    profile = Profile.chrome_143_windows()
    client = Client(profile)

    # Download a small test file
    url = "https://httpbin.org/bytes/1024"
    filename = "download.bin"

    print(f"Downloading from {url}")
    response = await client.get(url)

    print(f"Status Code: {response.status_code}")
    print(f"Wire Size: {response.wire_size} bytes")
    print(f"Content Size: {len(response.content)} bytes")
    print()

    # Save to file
    with open(filename, "wb") as f:
        f.write(response.content)

    print(f"Saved {len(response.content)} bytes to {filename}")

    # Clean up
    import os

    os.remove(filename)
    print(f"Cleaned up {filename}")


if __name__ == "__main__":
    asyncio.run(main())
