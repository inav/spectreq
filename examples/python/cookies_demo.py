"""
Cookie Management Example for Spectre Python bindings.

Demonstrates how to manually manage cookies using the CookieJar API,
including setting cookies, retrieving them, and persisting session state.

Run with: python examples/python/cookies_demo.py
"""

import asyncio
from spectreq import Profile, Client, CookieJar

async def main():
    # 1. Initialize client
    profile = Profile.chrome_143_windows()
    client = Client(profile)
    
    print("--- Cookie Management Demo ---")
    
    # 2. Access the CookieJar
    jar = client.cookie_jar()
    print(f"Initial Jar Size: {len(jar)}")
    
    # 3. Manually set a cookie
    # Useful for restoring sessions or setting consents
    url = "https://httpbin.org"
    print(f"\nSetting cookie 'session_id' for {url}...")
    jar.set_cookies(url, ["session_id=xyz789; Path=/; Secure"])
    
    # 4. Verify cookie was set
    cookie_header = jar.get_cookie_value(url)
    print(f"Cookie Header for {url}: {cookie_header}")
    
    # 5. Make a request verifying the cookie is sent
    print("\nMaking request to check cookies sent...")
    response = await client.get(f"{url}/cookies")
    data = response.json()
    print(f"Server received cookies: {data.get('cookies')}")
    
    # 6. Receive new cookies from server
    # httpbin/cookies/set endpoint sets cookies in the response
    print("\nRequesting server to set 'server_cookie'...")
    await client.get(f"{url}/cookies/set/server_cookie/fresh_value")
    
    # 7. Check if new cookie is in jar
    print(f"Jar Size after response: {len(jar)}")
    print(f"All cookies for {url}: {jar.get_cookie_value(url)}")
    
    # 8. Clear cookies
    print("\nClearing cookies...")
    jar.clear()
    print(f"Final Jar Size: {len(jar)}")

if __name__ == "__main__":
    asyncio.run(main())
