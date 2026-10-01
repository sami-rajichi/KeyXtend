use super::*;
use crate::test_support::{devtools_config as config, fake_env as env};

/// A made-up user profile, absolute on every system.
fn profile() -> PathBuf {
    std::env::temp_dir().join("Users").join("u")
}

/// The profile's roaming folder.
fn roaming() -> PathBuf {
    paths::join_rel(profile(), "AppData/Roaming")
}

#[test]
fn the_link_is_in_the_users_own_start_menu() {
    let link = link(
        &config(),
        env(vec![("APPDATA", roaming()), ("USERPROFILE", profile())]),
    )
    .unwrap();
    let menu = paths::join_rel(roaming(), "Microsoft/Windows/Start Menu/Programs");
    assert_eq!(link, menu.join("KeyXtend (dev).lnk"));
    assert!(paths::is_under(&link, &profile()));
}

#[test]
fn without_the_users_folder_there_is_no_link() {
    let all_users = env(vec![
        ("ProgramData", std::env::temp_dir()),
        ("USERPROFILE", profile()),
    ]);
    assert_eq!(
        link(&config(), all_users),
        Err(DevError::MissingEnv("APPDATA".to_string()))
    );
    let relative = env(vec![
        ("APPDATA", PathBuf::from("Roaming")),
        ("USERPROFILE", profile()),
    ]);
    assert!(matches!(
        link(&config(), relative),
        Err(DevError::MissingEnv(_))
    ));
}

#[test]
fn a_link_outside_the_users_profile_is_refused() {
    let shared = std::env::temp_dir().join("ProgramData");
    let outside = env(vec![("APPDATA", shared), ("USERPROFILE", profile())]);
    assert!(matches!(
        link(&config(), outside),
        Err(DevError::NotInProfile(_))
    ));
    let no_profile = env(vec![("APPDATA", roaming())]);
    assert_eq!(
        link(&config(), no_profile),
        Err(DevError::MissingEnv("USERPROFILE".to_string()))
    );
}

#[test]
fn params_name_the_action_link_and_program() {
    let p = params(script::ADD, Path::new("m.lnk"), Path::new("k.exe"));
    let expected = [
        (script::ACTION, script::ADD.to_string()),
        (script::LINK, "m.lnk".to_string()),
        (script::EXE, "k.exe".to_string()),
    ];
    assert_eq!(p, expected);
}

/// Runs the real shortcut script on links in a temp folder, never the Start menu.
#[cfg(windows)]
mod script_runs {
    use super::*;
    use crate::test_support::TempDir;

    fn setup() -> DevSetup {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        DevSetup {
            root: root.to_path_buf(),
            config: config(),
        }
    }

    /// Runs one action on `link` for `exe`, and returns the script's output.
    fn act(action: &str, link: &Path, exe: &Path) -> String {
        call(&setup(), &params(action, link, exe)).unwrap()
    }

    /// The program `link` starts, or nothing.
    fn target(link: &Path) -> String {
        act(script::FIND, link, Path::new("")).trim().to_string()
    }

    /// A temp folder with two programs and the link path.
    fn folder() -> (TempDir, PathBuf, PathBuf, PathBuf) {
        let dir = TempDir::new("shortcut");
        let (a, b) = (dir.path().join("a.exe"), dir.path().join("b.exe"));
        std::fs::write(&a, b"").unwrap();
        std::fs::write(&b, b"").unwrap();
        let link = dir.path().join("KeyXtend (dev).lnk");
        (dir, a, b, link)
    }

    #[test]
    fn a_second_add_replaces_the_shortcut() {
        let (_dir, a, b, link) = folder();
        act(script::ADD, &link, &a);
        assert_eq!(target(&link), paths::text(&a));
        act(script::ADD, &link, &b);
        assert_eq!(target(&link), paths::text(&b));
    }

    #[test]
    fn remove_deletes_the_shortcut_to_the_program() {
        let (_dir, a, _b, link) = folder();
        add(&setup(), &link, &a).unwrap();
        assert_eq!(remove(&setup(), &link, &a), Ok(true));
        assert!(!link.exists());
        assert_eq!(remove(&setup(), &link, &a), Ok(false));
    }

    #[test]
    fn remove_keeps_a_shortcut_to_another_program() {
        let (_dir, a, b, link) = folder();
        act(script::ADD, &link, &b);
        assert_eq!(remove(&setup(), &link, &a), Ok(false));
        assert_eq!(target(&link), paths::text(&b));
    }
}
