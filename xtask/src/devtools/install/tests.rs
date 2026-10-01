use super::*;
use crate::test_support::{TempDir, devtools_config, make_dir_link, stage_config};

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
fn only_a_folder_with_the_app_gets_a_shortcut() {
    let dir = TempDir::new("app");
    assert!(!holds_app(dir.path(), "keyxtend.exe"));
    std::fs::write(dir.path().join("keyxtend.exe"), b"").unwrap();
    assert!(holds_app(dir.path(), "keyxtend.exe"));
}

/// A folder holding `program` and a TLS plugin, with settings rooted there.
fn tls_folder(label: &str, program: &str) -> (TempDir, DevSetup) {
    let dir = TempDir::new(label);
    std::fs::write(dir.path().join(program), b"").unwrap();
    std::fs::create_dir_all(dir.path().join("tls")).unwrap();
    std::fs::write(dir.path().join("tls").join("x.dll"), b"").unwrap();
    let setup = DevSetup {
        root: dir.path().to_path_buf(),
        config: devtools_config(),
    };
    (dir, setup)
}

#[test]
fn dev_install_refuses_an_app_folder_that_fails_the_stage_check() {
    let stage = stage_config();
    let (dir, setup) = tls_folder("vet-app", &stage.program());
    let windows = |_: &str| Some(paths::text(dir.path()));
    let result = vet(dir.path(), &setup, &stage, windows);
    assert!(matches!(result, Err(DevError::Banned(_))), "{result:?}");
}

#[test]
fn a_folder_without_the_app_is_not_stage_checked() {
    let (dir, setup) = tls_folder("vet-other", "other.exe");
    let windows = |_: &str| Some(paths::text(dir.path()));
    assert_eq!(vet(dir.path(), &setup, &stage_config(), windows), Ok(()));
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
    let size = usize::try_from(paths::BYTES_PER_MB).unwrap() + 1;
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
