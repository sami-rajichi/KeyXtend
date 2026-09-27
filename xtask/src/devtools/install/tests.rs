use super::*;
use crate::test_support::{TempDir, devtools_config, make_dir_link};

const TP: &str = "0123456789ABCDEF0123456789ABCDEF01234567";

#[test]
fn plain_uses_the_shared_name_rule() {
    assert_eq!(plain("slint-kb"), Ok("slint-kb".to_string()));
    assert!(matches!(plain(".."), Err(DevError::BadName(_))));
    assert!(matches!(plain("kb[1]"), Err(DevError::BadName(_))));
}

#[test]
fn sign_args_use_the_user_store_and_digest() {
    let args = sign_args(&devtools_config(), TP, Path::new("kb.exe"));
    let expected = ["sign", "/s", "My", "/sha1", TP, "/fd", "SHA256", "kb.exe"];
    assert_eq!(args, expected);
}

#[test]
fn only_top_level_exes_are_signed() {
    let dir = TempDir::new("exes");
    std::fs::create_dir_all(dir.path().join("sub")).unwrap();
    for file in ["a.exe", "B.EXE", "c.dll"] {
        std::fs::write(dir.path().join(file), b"").unwrap();
    }
    std::fs::write(dir.path().join("sub").join("d.exe"), b"").unwrap();
    let found = exes(dir.path()).unwrap();
    assert_eq!(
        found,
        vec![dir.path().join("B.EXE"), dir.path().join("a.exe")]
    );
}

#[test]
fn a_folder_without_programs_is_refused() {
    let dir = TempDir::new("no-exe");
    std::fs::write(dir.path().join("c.dll"), b"").unwrap();
    assert!(matches!(exes(dir.path()), Err(DevError::NoExe(_))));
}

#[test]
fn a_source_with_a_folder_link_is_refused() {
    let dir = TempDir::new("link-src");
    let outside = TempDir::new("link-outside");
    let source = dir.path().join("kb");
    std::fs::create_dir_all(&source).unwrap();
    make_dir_link(&source.join("big"), outside.path());
    let target = dir.path().join("target");
    assert!(matches!(
        check_source(&source, &target, 1),
        Err(DevError::Link(_))
    ));
}

#[test]
fn a_source_that_is_the_target_is_refused() {
    let dir = TempDir::new("overlap");
    let kb = dir.path().join("kb");
    std::fs::create_dir_all(&kb).unwrap();
    assert!(matches!(
        check_source(&kb, &kb, 1),
        Err(DevError::Overlap(_))
    ));
    assert!(matches!(
        check_source(dir.path(), &kb, 1),
        Err(DevError::Overlap(_))
    ));
}

#[test]
fn a_source_over_the_limit_is_refused() {
    let dir = TempDir::new("big");
    let source = dir.path().join("kb");
    std::fs::create_dir_all(&source).unwrap();
    let size = usize::try_from(BYTES_PER_MB).unwrap() + 1;
    std::fs::write(source.join("big.bin"), vec![0u8; size]).unwrap();
    let target = dir.path().join("target");
    assert_eq!(
        check_source(&source, &target, 1),
        Err(DevError::TooBig { mb: 2, max: 1 })
    );
    assert_eq!(check_source(&source, &target, 2), Ok(()));
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn install_name_is_the_last_folder() {
        assert_eq!(
            install_name(Path::new("D:\\x\\slint-kb\\")),
            Ok("slint-kb".to_string())
        );
    }

    #[test]
    fn a_parent_folder_path_has_no_install_name() {
        assert!(install_name(Path::new("D:\\x\\..")).is_err());
    }

    #[test]
    fn target_is_inside_the_install_folder() {
        let target = target_dir(Path::new("C:\\PF"), &devtools_config(), "qt-kb");
        assert_eq!(target, PathBuf::from("C:\\PF\\KeyXtend-dev\\qt-kb"));
    }
}
