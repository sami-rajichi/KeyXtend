use proptest::prelude::*;

use super::*;
use crate::test_support::{TempDir, make_dir_link};

#[test]
fn plain_names_are_accepted() {
    for good in [
        "slint-kb", "qt_kb.1", "COM", "COM10", "COM⁴", "console", "Nulls",
    ] {
        assert!(is_plain_name(good), "{good:?} was refused");
    }
}

#[test]
fn separators_dots_and_wildcards_are_refused() {
    for bad in [
        "", " ", ".", "..", "a\\b", "a/b", "c:x", "a*b", "kb[1]", "a`b",
    ] {
        assert!(!is_plain_name(bad), "{bad:?} was accepted");
    }
}

#[test]
fn names_windows_would_change_are_refused() {
    for bad in ["x.", "x ", ". ", ".. ", " ."] {
        assert!(!is_plain_name(bad), "{bad:?} was accepted");
    }
}

#[test]
fn device_names_are_refused() {
    for bad in [
        "NUL",
        "nul.txt",
        "Con",
        "COM1",
        "lpt9.log",
        "aux.x.y",
        "COM¹",
        "lpt³.txt",
    ] {
        assert!(!is_plain_name(bad), "{bad:?} was accepted");
    }
}

#[test]
fn relative_settings_paths_are_checked() {
    assert!(is_relative_plain("target/dev-tools"));
    assert!(is_relative_plain("xtask"));
    for bad in ["", "../x", "a/../b", "C:/x", "/x", "a//b", "a/"] {
        assert!(!is_relative_plain(bad), "{bad:?} was accepted");
    }
}

#[test]
fn settings_paths_join_part_by_part() {
    let joined = join_rel(PathBuf::from("ws"), "a/b");
    assert_eq!(joined, Path::new("ws").join("a").join("b"));
}

#[test]
fn paths_with_parent_steps_count_as_overlapping() {
    assert!(overlaps(Path::new("/a/../b"), Path::new("/c")));
    assert!(overlaps(Path::new("/c"), Path::new("./c")));
}

#[test]
fn folder_size_adds_nested_files() {
    let dir = TempDir::new("size");
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    std::fs::write(dir.path().join("a.bin"), [0u8; 3]).unwrap();
    std::fs::write(dir.path().join("sub").join("b.bin"), [0u8; 5]).unwrap();
    assert_eq!(folder_size(dir.path()), Ok(8));
}

#[test]
fn folder_size_refuses_a_folder_link() {
    let dir = TempDir::new("size-link");
    let outside = TempDir::new("size-outside");
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    make_dir_link(&dir.path().join("sub").join("link"), outside.path());
    assert!(matches!(folder_size(dir.path()), Err(DevError::Link(_))));
}

#[cfg(windows)]
mod windows {
    use super::*;

    const PF: &str = "C:\\Program Files";

    #[test]
    fn a_file_inside_is_under() {
        let exe = Path::new("C:\\Program Files\\KeyXtend-dev\\kb\\kb.exe");
        assert!(is_under(exe, Path::new(PF)));
    }

    #[test]
    fn letter_case_is_ignored() {
        assert!(is_under(
            Path::new("c:\\program files\\kb.exe"),
            Path::new(PF)
        ));
    }

    #[test]
    fn a_lookalike_folder_is_not_under() {
        let exe = Path::new("C:\\Program Files Evil\\kb.exe");
        assert!(!is_under(exe, Path::new(PF)));
    }

    #[test]
    fn the_folder_itself_is_not_under() {
        assert!(!is_under(Path::new(PF), Path::new(PF)));
    }

    #[test]
    fn a_parent_step_is_never_under() {
        let exe = Path::new("C:\\Program Files\\..\\Users\\kb.exe");
        assert!(!is_under(exe, Path::new(PF)));
    }

    #[test]
    fn an_empty_drive_or_relative_root_contains_nothing() {
        let exe = Path::new("C:\\Users\\kb.exe");
        for root in ["", "C:", "C:\\", "Program Files"] {
            assert!(!is_under(exe, Path::new(root)), "{root:?} contained it");
        }
    }

    #[test]
    fn overlapping_folders_are_detected() {
        let a = Path::new("C:\\PF\\KeyXtend-dev\\kb");
        assert!(overlaps(a, Path::new("c:\\pf\\keyxtend-dev\\KB")));
        assert!(overlaps(a, Path::new("C:\\PF\\KeyXtend-dev\\kb\\bin")));
        assert!(overlaps(Path::new("C:\\PF"), a));
        assert!(!overlaps(a, Path::new("C:\\PF\\KeyXtend-dev\\qt")));
    }

    #[test]
    fn normalize_drops_a_trailing_separator() {
        let path = normalize(Path::new("D:\\x\\kb\\"));
        assert_eq!(text(&path), "D:\\x\\kb");
    }

    proptest! {
        #[test]
        fn a_path_with_a_parent_step_is_never_under(tail in "[a-z]{1,8}") {
            let exe = PathBuf::from(format!("{PF}\\..\\{tail}\\kb.exe"));
            prop_assert!(!is_under(&exe, Path::new(PF)));
        }

        #[test]
        fn any_plain_file_inside_is_under(name in "[A-Za-z0-9_-]{1,12}") {
            let exe = Path::new(PF).join(format!("{name}.exe"));
            prop_assert!(is_under(&exe, Path::new(PF)));
        }
    }
}

proptest! {
    #[test]
    fn a_name_with_a_separator_is_never_plain(a in ".*", b in ".*", sep in "[/\\\\]") {
        let name = format!("{a}{sep}{b}");
        prop_assert!(!is_plain_name(&name));
    }
}
