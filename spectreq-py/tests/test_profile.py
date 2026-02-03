"""Tests for browser profiles."""

import pytest
from spectreq import Profile


class TestChromeProfiles:
    """Tests for Chrome browser profiles."""

    def test_chrome_120_windows(self):
        """Test Chrome 120 Windows profile."""
        profile = Profile.chrome_120_windows()
        assert profile.browser == "Chrome"
        assert profile.os == "Windows"
        assert "120" in profile.version
        assert "Chrome/120" in profile.user_agent
        assert "Windows NT" in profile.user_agent

    def test_chrome_120_macos(self):
        """Test Chrome 120 macOS profile."""
        profile = Profile.chrome_120_macos()
        assert profile.browser == "Chrome"
        assert profile.os == "MacOS"
        assert "Macintosh" in profile.user_agent

    def test_chrome_120_linux(self):
        """Test Chrome 120 Linux profile."""
        profile = Profile.chrome_120_linux()
        assert profile.browser == "Chrome"
        assert profile.os == "Linux"
        assert "Linux" in profile.user_agent

    def test_chrome_120_android(self):
        """Test Chrome 120 Android profile."""
        profile = Profile.chrome_120_android()
        assert profile.browser == "Chrome"
        assert profile.os == "Android"
        assert "Android" in profile.user_agent

    def test_chrome_131_windows(self):
        """Test Chrome 131 Windows profile."""
        profile = Profile.chrome_131_windows()
        assert profile.browser == "Chrome"
        assert "131" in profile.version

    def test_chrome_133_windows(self):
        """Test Chrome 133 Windows profile."""
        profile = Profile.chrome_133_windows()
        assert "133" in profile.version

    def test_chrome_141_windows(self):
        """Test Chrome 141 Windows profile."""
        profile = Profile.chrome_141_windows()
        assert "141" in profile.version

    def test_chrome_143_windows(self):
        """Test Chrome 143 Windows profile."""
        profile = Profile.chrome_143_windows()
        assert profile.browser == "Chrome"
        assert profile.os == "Windows"
        assert "143" in profile.version
        assert "Chrome/143" in profile.user_agent

    def test_chrome_143_macos(self):
        """Test Chrome 143 macOS profile."""
        profile = Profile.chrome_143_macos()
        assert profile.os == "MacOS"
        assert "143" in profile.version

    def test_chrome_143_linux(self):
        """Test Chrome 143 Linux profile."""
        profile = Profile.chrome_143_linux()
        assert profile.os == "Linux"
        assert "143" in profile.version

    def test_chrome_143_android(self):
        """Test Chrome 143 Android profile."""
        profile = Profile.chrome_143_android()
        assert profile.os == "Android"
        assert "143" in profile.version


class TestFirefoxProfiles:
    """Tests for Firefox browser profiles."""

    def test_firefox_121_windows(self):
        """Test Firefox 121 Windows profile."""
        profile = Profile.firefox_121_windows()
        assert profile.browser == "Firefox"
        assert profile.os == "Windows"
        assert "121" in profile.version
        assert "Firefox/121" in profile.user_agent


class TestSafariProfiles:
    """Tests for Safari browser profiles."""

    def test_safari_17_macos(self):
        """Test Safari 17 macOS profile."""
        profile = Profile.safari_17_macos()
        assert profile.browser == "Safari"
        assert profile.os == "MacOS"
        assert "17" in profile.version
        assert "Safari" in profile.user_agent


class TestEdgeProfiles:
    """Tests for Edge browser profiles."""

    def test_edge_120_windows(self):
        """Test Edge 120 Windows profile."""
        profile = Profile.edge_120_windows()
        assert profile.browser == "Edge"
        assert profile.os == "Windows"
        assert "120" in profile.version
        assert "Edg/" in profile.user_agent


class TestProfileProperties:
    """Tests for profile property access."""

    def test_profile_to_dict(self):
        """Test converting profile to dictionary."""
        profile = Profile.chrome_143_windows()
        info = profile.to_dict()
        
        assert isinstance(info, dict)
        assert "browser" in info
        assert "os" in info
        assert "version" in info
        assert info["browser"] == "Chrome"

    def test_profile_properties_not_empty(self):
        """Test that all profiles have required properties."""
        profiles = [
            Profile.chrome_120_windows(),
            Profile.chrome_143_windows(),
            Profile.firefox_121_windows(),
            Profile.safari_17_macos(),
            Profile.edge_120_windows(),
        ]
        
        for profile in profiles:
            assert profile.browser, "Browser should not be empty"
            assert profile.os, "OS should not be empty"
            assert profile.version, "Version should not be empty"
            assert profile.user_agent, "User agent should not be empty"
