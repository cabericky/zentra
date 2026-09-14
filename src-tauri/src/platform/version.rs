#[repr(C)]
#[allow(non_snake_case)]
struct OSVERSIONINFOW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
}

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut OSVERSIONINFOW) -> i32;
}

/// Retrieves the Windows OS major version, minor version, and build number via RtlGetVersion.
pub fn get_windows_version() -> Option<(u32, u32, u32)> {
    let mut info: OSVERSIONINFOW = unsafe { std::mem::zeroed() };
    info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
    let status = unsafe { RtlGetVersion(&mut info) };
    if status == 0 {
        Some((info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber))
    } else {
        None
    }
}

/// Checks if the running operating system is at least the specified Windows build number.
/// Accounts for Windows 10 (major == 10) build thresholds as well as Windows 11 / future versions (major > 10).
pub fn is_windows_build_at_least(required_build: u32) -> bool {
    if let Some((major, _minor, build)) = get_windows_version() {
        major > 10 || (major == 10 && build >= required_build)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_version_detection() {
        let version = get_windows_version();
        assert!(version.is_some(), "Should retrieve Windows OS version");
        let (major, _minor, build) = version.unwrap();
        assert!(major >= 10, "Expected Windows 10 or later");
        assert!(build > 0, "Build number should be positive");
    }

    #[test]
    fn test_is_windows_build_at_least() {
        // Checking against a baseline Windows 10 build should return true
        assert!(is_windows_build_at_least(10240));
        // Checking against an improbable future build should return false
        assert!(!is_windows_build_at_least(999_999));
    }
}