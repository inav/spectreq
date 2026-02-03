"""
POST request with JSON example for Spectre Python bindings

This example demonstrates how to make POST requests with JSON data.

Run with: python examples/post_request.py
"""

import asyncio
import json
from spectreq import Profile, Client


async def main():
    # Create client
    profile = Profile.chrome_143_windows()
    client = Client(profile)

    # POST JSON data
    data = {"name": "John Doe", "email": "john@example.com", "age": 30}

    print("Posting JSON data to https://httpbin.org/post")
    print(f"Data: {json.dumps(data, indent=2)}")
    print()

    # Convert to bytes
    json_data = json.dumps(data).encode("utf-8")

    # Make POST request
    response = await client.post("https://httpbin.org/post", json_data)

    print(f"Status Code: {response.status_code}")
    print(f"Content-Type: {response.get_header('content-type')}")
    print()

    # Parse and print JSON response
    result = response.json()
    print("Response JSON:")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    asyncio.run(main())
