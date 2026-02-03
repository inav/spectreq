"""
Browser profiles example for Spectre Python bindings

This example demonstrates all available pre-configured browser profiles.

Run with: python examples/profiles.py
"""

from spectreq import Profile


def main():
    print("=" * 60)
    print("Available Browser Profiles")
    print("=" * 60)
    print()

    # Chrome profiles
    print("Chrome 120 Series:")
    print("-" * 40)

    profiles_120 = [
        ("Windows", Profile.chrome_120_windows()),
        ("macOS", Profile.chrome_120_macos()),
        ("Linux", Profile.chrome_120_linux()),
        ("Android", Profile.chrome_120_android()),
    ]

    for os_name, profile in profiles_120:
        print(f"  Chrome 120 on {os_name}:")
        print(f"    Browser: {profile.browser}")
        print(f"    OS: {profile.os}")
        print(f"    Version: {profile.version}")
        print(f"    User-Agent: {profile.user_agent[:60]}...")
        print()

    # Chrome 131+ Series
    print("Chrome 131+ Series:")
    print("-" * 40)

    profiles_131 = [
        ("Chrome 131 Windows", Profile.chrome_131_windows()),
        ("Chrome 133 Windows", Profile.chrome_133_windows()),
        ("Chrome 141 Windows", Profile.chrome_141_windows()),
    ]

    for name, profile in profiles_131:
        print(f"  {name}:")
        print(f"    Version: {profile.version}")
        print(f"    User-Agent: {profile.user_agent[:60]}...")
        print()

    # Chrome 143 Series
    print("Chrome 143 Series:")
    print("-" * 40)

    profiles_143 = [
        ("Windows", Profile.chrome_143_windows()),
        ("macOS", Profile.chrome_143_macos()),
        ("Linux", Profile.chrome_143_linux()),
        ("Android", Profile.chrome_143_android()),
    ]

    for os_name, profile in profiles_143:
        print(f"  Chrome 143 on {os_name}:")
        print(f"    Version: {profile.version}")
        print(f"    User-Agent: {profile.user_agent[:60]}...")
        print()

    # Other browsers
    print("Other Browsers:")
    print("-" * 40)

    other_profiles = [
        ("Firefox 121 Windows", Profile.firefox_121_windows()),
        ("Safari 17 macOS", Profile.safari_17_macos()),
        ("Edge 120 Windows", Profile.edge_120_windows()),
    ]

    for name, profile in other_profiles:
        print(f"  {name}:")
        print(f"    Browser: {profile.browser}")
        print(f"    OS: {profile.os}")
        print(f"    Version: {profile.version}")
        print(f"    User-Agent: {profile.user_agent[:60]}...")
        print()

    # Profile as dict
    print("Profile as Dictionary:")
    print("-" * 40)
    profile = Profile.chrome_143_windows()
    profile_dict = profile.to_dict()
    for key, value in profile_dict.items():
        print(f"  {key}: {value}")


if __name__ == "__main__":
    main()
