import spectreq
import asyncio
import sys

print(f"Python: {sys.version}")
print(f"Exports: {dir(spectreq)}")

def verify():
    print("Verifying exports...")
    assert hasattr(spectreq, "Client"), "Client missing"
    assert hasattr(spectreq, "Profile"), "Profile missing"
    assert hasattr(spectreq, "Response"), "Response missing"
    assert hasattr(spectreq, "BearerToken"), "BearerToken missing"
    
    print("Verifying instantiation...")
    p = spectreq.Profile.chrome_143_windows()
    print(f"Profile: {p}")
    
    c = spectreq.Client(p)
    print(f"Client: {c}")
    
    print("Verification success!")

if __name__ == "__main__":
    verify()
