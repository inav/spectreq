"""
Basic HTTP request example for Spectre Python bindings

This example demonstrates how to make a simple GET request using
the Spectre client with browser impersonation.

Run with: python examples/basic_request.py
"""

import asyncio
from spectreq import Profile, Client


async def main():
    # Create a client with Chrome 143 on Windows profile
    profile = Profile.chrome_143_windows()
    print(f"Using profile: {profile}")
    print(f"User-Agent: {profile.user_agent}")
    print()

    client = Client(profile)

    # Make a GET request
    print("Making GET request to https://httpbin.org/get")
    response = await client.get("https://httpbin.org/get")

    print(f"Status Code: {response.status_code}")
    print(f"Wire Size: {response.wire_size} bytes")
    print(f"From Cache: {response.from_cache}")
    print()

    # Print response body
    print("Response Body:")
    print(response.text())


if __name__ == "__main__":
    asyncio.run(main())
