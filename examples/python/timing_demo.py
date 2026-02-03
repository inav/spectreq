"""
Performance Metrics Example for Spectre Python bindings.

Demonstrates how to access request timing metrics (TTFB, Total Time, DNS, etc.)
to analyze performance.

Run with: python examples/python/timing_demo.py
"""

import asyncio
from spectreq import Profile, Client

async def main():
    profile = Profile.chrome_143_windows()
    client = Client(profile)
    
    urls = [
        "https://httpbin.org/get",
        "https://httpbin.org/delay/1",  # Test TTFB tracking
        "https://www.google.com"
    ]
    
    print("--- Request Timing and Metrics ---")
    
    for url in urls:
        print(f"\nRequesting: {url}")
        try:
            response = await client.get(url)
            timing = response.timing
            
            print(f"Status: {response.status_code}")
            print(f"Total Time: {timing.total:.4f}s")
            print(f"TTFB (Time to First Byte): {timing.ttfb:.4f}s")
            print(f"DNS Lookup: {timing.dns_lookup:.4f}s")
            print(f"TCP Connect: {timing.tcp_connect:.4f}s")
            print(f"TLS Handshake: {timing.tls_handshake:.4f}s")
            
            # Note: DNS/TCP/TLS might be 0 if connection was reused (Keep-Alive)
            # or if the metrics are not yet fully instrumented in the core connector.
            
        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
