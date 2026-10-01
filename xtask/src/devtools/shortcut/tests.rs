use super::*;
use crate::test_support::devtools_config as config;

/// A made-up roaming folder, absolute on every system.
fn roaming() -> PathBuf {
    std::env::temp_dir().join("Roaming")
}

/// The environment with only `key` set to `value`.
fn only(key: &'static str, value: PathBuf) -> impl Fn(&str) -> Option<String> {
    move |k| (k == key).then(|| paths::text(&value))
}

#[test]
fn the_link_is_in_the_users_own_start_menu() {
    let link = link(&config(), only("APPDATA", roaming())).unwrap();
    let menu = paths::join_rel(roaming(), "Microsoft/Windows/Start Menu/Programs");
    assert_eq!(link, menu.join("KeyXtend (dev).lnk"));
    assert!(paths::is_under(&link, &roaming()));
}

#[test]
fn without_the_users_folder_there_is_no_link() {
    let all_users = only("ProgramData", std::env::temp_dir());
    assert_eq!(
        link(&config(), all_users),
        Err(DevError::MissingEnv("APPDATA".to_string()))
    );
    let relative = only("APPDATA", PathBuf::from("Roaming"));
    assert!(matches!(
        link(&config(), relative),
        Err(DevError::MissingEnv(_))
    ));
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
        act(script::ADD, &link, &a);
        act(script::REMOVE, &link, &a);
        assert!(!link.exists());
        act(script::REMOVE, &link, &a);
    }

    #[test]
    fn remove_keeps_a_shortcut_to_another_program() {
        let (_dir, a, b, link) = folder();
        act(script::ADD, &link, &b);
        act(script::REMOVE, &link, &a);
        assert_eq!(target(&link), paths::text(&b));
    }
}
