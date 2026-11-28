//! Example paths written once for Windows that read as this system's own (`PathRoot::NATIVE`), so a test that hands
//! one to the path rules runs on every system (ADR-0062).

use kx_module_api::ports::PathRoot;
use std::path::PathBuf;

/// `windows`, such as `C:\Users\someone`, in this system's style: as written on Windows, `/c/Users/someone` elsewhere.
/// A refusal example must not lean on a drive being missing: `\x` turns into `/x`, which Unix takes.
#[must_use]
pub fn native(windows: &str) -> String {
    match PathRoot::NATIVE {
        PathRoot::Drive => windows.to_owned(),
        PathRoot::Slash => unix(windows),
    }
}

/// `native` as a path.
#[must_use]
pub fn native_path(windows: &str) -> PathBuf {
    PathBuf::from(native(windows))
}

/// `windows` as a Unix path, as Git Bash spells it: a drive's root `C:\` becomes `/c/` and each `\` a `/`, so drives
/// stay apart; anything else, `C:x` too, only turns its backslashes, so a relative path stays relative.
fn unix(windows: &str) -> String {
    let turned = windows.replace('\\', "/");
    match (
        PathRoot::Drive.len_of(windows.as_bytes()),
        windows.chars().next(),
    ) {
        (Some(root), Some(letter)) => {
            format!("/{}/{}", letter.to_ascii_lowercase(), &turned[root..])
        }
        _ => turned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drive_path_becomes_its_letters_folder_and_turns_its_backslashes() {
        let cases = [
            (r"C:\Users\someone", "/c/Users/someone"),
            ("d:/a/b.png", "/d/a/b.png"),
            (r"D:\", "/d/"),
            (r"\\server\share", "//server/share"),
            ("C:x", "C:x"),
            ("C:", "C:"),
            ("x", "x"),
        ];
        for (windows, want) in cases {
            assert_eq!(unix(windows), want, "{windows}");
        }
    }

    #[test]
    fn a_path_is_kept_on_windows_and_turned_elsewhere() {
        let want = match PathRoot::NATIVE {
            PathRoot::Drive => r"C:\a\b",
            PathRoot::Slash => "/c/a/b",
        };
        assert_eq!(native(r"C:\a\b"), want);
        assert_eq!(native_path(r"C:\a\b"), PathBuf::from(want));
    }
}
