use super::*;
use crate::test_support::{TempDir, devtools_config, fake_env, make_dir_link, noise, stage_config};

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
    assert!(
        matches!(result, Err(DevError::Stage(StageError::Banned(_)))),
        "{result:?}"
    );
}

#[test]
fn dev_install_holds_an_app_folder_to_the_download_budget() {
    let mut stage = stage_config();
    stage.max_mb = 1;
    let dir = TempDir::new("vet-budget");
    let size = usize::try_from(paths::BYTES_PER_MB).unwrap();
    let program = [stage::bare_image(), noise(size)].concat();
    std::fs::write(dir.path().join(stage.program()), program).unwrap();
    let setup = DevSetup {
        root: dir.path().to_path_buf(),
        config: devtools_config(),
    };
    let windows = |_: &str| Some(paths::text(dir.path()));
    let over = StageError::OverBudget { mb: 2, max: 1 };
    assert_eq!(vet(dir.path(), &setup, &stage, windows), Err(over.into()));
}

#[test]
fn dev_install_refuses_any_program_it_would_sign_that_holds_the_gate_flag() {
    let dir = TempDir::new("gate-exes");
    let flag = stage_config().gate_flag;
    let (tool, gate) = (dir.path().join("tool.exe"), dir.path().join("other.exe"));
    std::fs::write(&tool, b"MZ --show").unwrap();
    std::fs::write(&gate, format!("MZ {flag} x")).unwrap();
    let programs = exes(dir.path()).unwrap();
    let exe = paths::text(&gate);
    let refused = StageError::GateFlag {
        exe,
        flag: flag.clone(),
    };
    assert_eq!(refuse_gate_builds(&programs, &flag), Err(refused.into()));
    std::fs::write(&gate, b"MZ --quit").unwrap();
    assert_eq!(refuse_gate_builds(&programs, &flag), Ok(()));
}

#[test]
fn a_folder_without_the_app_is_not_stage_checked() {
    let (dir, setup) = tls_folder("vet-other", "other.exe");
    let windows = |_: &str| Some(paths::text(dir.path()));
    assert_eq!(vet(dir.path(), &setup, &stage_config(), windows), Ok(()));
}

/// A folder holding the app, and an environment whose roaming folder is `menu` and whose profile is `u`, both in it.
fn app_with_menu(label: &str, menu: &str) -> (TempDir, impl Fn(&str) -> Option<String>) {
    let dir = TempDir::new(label);
    std::fs::write(dir.path().join(stage_config().program()), b"").unwrap();
    let at = |rel: &str| paths::join_rel(dir.path().to_path_buf(), rel);
    let env = fake_env(vec![("APPDATA", at(menu)), ("USERPROFILE", at("u"))]);
    (dir, env)
}

#[test]
fn an_app_folder_gets_its_shortcut_worked_out_up_front() {
    let (dir, env) = app_with_menu("link-app", "u/AppData/Roaming");
    let (config, program) = (devtools_config(), stage_config().program());
    let link = link_for(dir.path(), &program, &config, &env);
    assert_eq!(link, Ok(Some(shortcut::link(&config, &env).unwrap())));
}

#[test]
fn an_app_folder_whose_shortcut_leaves_the_profile_is_refused() {
    let (dir, env) = app_with_menu("link-out", "ProgramData");
    let program = stage_config().program();
    let link = link_for(dir.path(), &program, &devtools_config(), env);
    assert!(matches!(link, Err(DevError::NotInProfile(_))), "{link:?}");
}

#[test]
fn a_folder_without_the_app_needs_no_shortcut() {
    let dir = TempDir::new("link-none");
    let program = stage_config().program();
    let link = link_for(dir.path(), &program, &devtools_config(), fake_env(vec![]));
    assert_eq!(link, Ok(None));
}

#[test]
fn a_folder_without_programs_is_refused() {
    let dir = TempDir::new("no-exe");
    std::fs::write(dir.path().join("c.dll"), b"").unwrap();
    assert!(matches!(
        exes(dir.path()),
        Err(DevError::Stage(StageError::NoExe(_)))
    ));
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
