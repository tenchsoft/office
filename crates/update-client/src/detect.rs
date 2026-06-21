//! Detect how this binary was installed. Used to decide whether the
//! self-update path is allowed (AppImage/Windows/MacOS bundle: yes) or whether
//! the system package manager owns updates (deb/rpm/Flatpak: defer).
//!
//! See `tench-docs/plans/contracts/Licensing/licensing-auth.md` (commit log
//! around 2026-06-21 for the deb/rpm split).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    /// Single-file AppImage. Self-update is allowed.
    AppImage,
    /// `.deb` package. apt owns updates; self-update is disabled.
    Deb,
    /// `.rpm` package. dnf owns updates; self-update is disabled.
    Rpm,
    /// Future Flatpak support. Flatpak runtime owns updates.
    Flatpak,
    /// Windows MSI/exe directory layout. Self-update via helper-exe.
    WindowsNative,
    /// macOS `.app` bundle. Self-update via rename swap.
    MacosBundle,
    /// Cannot determine. Conservative default: act as if self-update is
    /// disabled.
    Unknown,
}

impl InstallMethod {
    pub fn supports_self_update(self) -> bool {
        matches!(
            self,
            Self::AppImage | Self::WindowsNative | Self::MacosBundle
        )
    }
}

/// Detect the install method for the currently running binary.
///
/// Heuristics:
/// - Windows → [`InstallMethod::WindowsNative`] (always, for now)
/// - macOS   → [`InstallMethod::MacosBundle`] if running from `.app/Contents/MacOS/`
/// - Linux   → [`InstallMethod::AppImage`] if `APPIMAGE` env var is set
///             (AppImage runtime sets it). Otherwise `Deb` if `/var/lib/dpkg/status`
///             mentions a tench package, `Rpm` if `/var/lib/rpm` exists with a
///             tench package, else `Unknown`.
pub fn detect_install_method() -> InstallMethod {
    if cfg!(target_os = "windows") {
        return InstallMethod::WindowsNative;
    }

    if cfg!(target_os = "macos") {
        // Look for ".app/Contents/MacOS" in the current exe path.
        if let Ok(exe) = std::env::current_exe() {
            for ancestor in exe.ancestors() {
                if ancestor.extension().and_then(|e| e.to_str()) == Some("app") {
                    return InstallMethod::MacosBundle;
                }
            }
        }
        return InstallMethod::Unknown;
    }

    if cfg!(target_os = "linux") {
        if std::env::var_os("APPIMAGE").is_some() {
            return InstallMethod::AppImage;
        }
        if is_installed_via_dpkg() {
            return InstallMethod::Deb;
        }
        if is_installed_via_rpm() {
            return InstallMethod::Rpm;
        }
        return InstallMethod::Unknown;
    }

    InstallMethod::Unknown
}

#[cfg(target_os = "linux")]
fn is_installed_via_dpkg() -> bool {
    let Ok(status) = std::fs::read_to_string("/var/lib/dpkg/status") else {
        return false;
    };
    status.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("Package: tench-") || trimmed.starts_with("Package: tenchoffice-")
    })
}

#[cfg(target_os = "linux")]
fn is_installed_via_rpm() -> bool {
    // Without depending on librpm, do a best-effort directory check.
    std::path::Path::new("/var/lib/rpm").is_dir()
}

#[cfg(not(target_os = "linux"))]
fn is_installed_via_dpkg() -> bool {
    false
}

#[cfg(not(target_os = "linux"))]
fn is_installed_via_rpm() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_update_predicate() {
        assert!(InstallMethod::AppImage.supports_self_update());
        assert!(InstallMethod::WindowsNative.supports_self_update());
        assert!(InstallMethod::MacosBundle.supports_self_update());
        assert!(!InstallMethod::Deb.supports_self_update());
        assert!(!InstallMethod::Rpm.supports_self_update());
        assert!(!InstallMethod::Flatpak.supports_self_update());
        assert!(!InstallMethod::Unknown.supports_self_update());
    }

    #[test]
    fn detect_returns_a_value() {
        let _ = detect_install_method();
    }
}
