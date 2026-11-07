//! The Windows probe on a packages tree built in a temp folder: a probe found or not, its file always deleted, the
//! copies listed and deleted with links deleted as links, and the shipped table.

use std::fs;

use kx_platform::APP_DIR_NAME;
use kx_test_support::tempdir::TempDir;

use super::*;

#[cfg(windows)]
mod links;
#[cfg(windows)]
mod live;

/// The package whose mirror the tests fill.
const PKG: &str = "Claude_pzs8sxrjxfjjc";
/// A package with an empty mirror.
const OTHER: &str = "Other_8wekyb3d8bbwe";

/// A local app-data folder in a temp folder, with a packages root and each package's empty mirror.
struct Tree {
    _dir: TempDir,
    top: PathBuf,
    local: PathBuf,
    root: PathBuf,
}

impl Tree {
    /// A fresh tree named after `label`.
    fn new(label: &str) -> Self {
        let dir = TempDir::new(label).unwrap();
        let top = dir.path().to_path_buf();
        let local = top.join("Local");
        let root = local.join(&table().unwrap().packages);
        let tree = Self {
            _dir: dir,
            top,
            local,
            root,
        };
        for pkg in [PKG, OTHER] {
            fs::create_dir_all(tree.mirror(pkg)).unwrap();
        }
        tree
    }

    /// `pkg`'s private copy of the local folder.
    fn mirror(&self, pkg: &str) -> PathBuf {
        self.root.join(pkg).join(steps())
    }

    /// `pkg`'s hidden copy of our data folder.
    fn copy_in(&self, pkg: &str) -> PathBuf {
        self.mirror(pkg).join(APP_DIR_NAME)
    }

    /// The installed data folder.
    fn data(&self) -> PathBuf {
        self.local.join(APP_DIR_NAME)
    }

    /// The port over the tree.
    fn hidden(&self) -> WinHidden {
        WinHidden::at(self.local.clone(), self.root.clone())
    }

    /// The port over a local folder that is `pkg`'s mirror itself, so every write lands there as if redirected.
    fn hidden_in(&self, pkg: &str) -> WinHidden {
        WinHidden::at(self.mirror(pkg), self.root.clone())
    }

    /// The copy the list gives for `pkg`.
    fn listed(&self, pkg: &str, app: &str) -> HiddenCopy {
        HiddenCopy::new(self.copy_in(pkg), pkg.into(), app.into())
    }
}

/// The file a filled folder holds.
const SETTINGS: &str = "settings.toml";

/// Makes `dir` with a settings file in it.
fn fill(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join(SETTINGS), b"x").unwrap();
}

/// The shipped mirror inside a package folder.
fn steps() -> PathBuf {
    table().unwrap().mirror.iter().collect()
}

/// The names in `dir`.
fn names(dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name());
    entries.map(|n| n.to_string_lossy().into_owned()).collect()
}

#[test]
fn the_shipped_table_parses_into_plain_names() {
    let t = table().unwrap();
    let names = t.mirror.iter().chain([&t.packages, &t.probe]);
    assert!(names.into_iter().all(|n| one_name(n)));
    assert!(!t.app_end.is_empty() && t.probe_bytes > 0);
}

/// A table means the same on every system, though some read a backslash or a colon as plain letters.
#[test]
fn a_separator_or_a_drive_is_no_plain_name_on_any_system() {
    for bad in [
        "", ".", "..", "/x", "a/b", "a\\b", "\\x", "C:", "C:x", "ab:c",
    ] {
        assert!(!one_name(bad), "{bad}");
    }
    for good in ["Box", "kx-probe-", "~-1-2.tmp", "a b"] {
        assert!(one_name(good), "{good}");
    }
}

#[test]
fn a_table_with_an_empty_value_or_a_path_is_refused() {
    let good = [
        ("packages", "\"Packages\""),
        ("mirror", "[\"LocalCache\", \"Local\"]"),
        ("app_end", "\"_\""),
        ("probe", "\"kx-probe-\""),
        ("probe_bytes", "8"),
    ];
    let text = |key: &str, bad: &str| -> String {
        let line = |(k, v): &(&str, &str)| format!("{k} = {}\n", if *k == key { bad } else { v });
        good.iter().map(line).collect()
    };
    assert!(Table::parse(&text("", "")).is_ok());
    let bad = [
        ("packages", "\"\""),
        ("packages", "\"a/b\""),
        ("mirror", "[]"),
        ("mirror", "[\"LocalCache\", \"..\"]"),
        ("app_end", "\"\""),
        ("probe", "\"x/\""),
        ("probe_bytes", "0"),
    ];
    for (key, value) in bad {
        let got = Table::parse(&text(key, value));
        assert_eq!(got.err(), Some(TableError::Bad), "{key} = {value}");
    }
    let unknown = format!("{}extra = 1\n", text("", ""));
    assert!(matches!(Table::parse(&unknown), Err(TableError::Parse(_))));
}

#[test]
fn a_probe_name_is_fresh_each_time_and_starts_as_the_table_says() {
    let t = table().unwrap();
    let (a, b) = (t.probe_name().unwrap(), t.probe_name().unwrap());
    assert_ne!(a, b);
    let hex = a.strip_prefix(t.probe.as_str()).unwrap();
    assert_eq!(hex.len(), t.probe_bytes * 2);
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()) && one_name(&a));
}

#[test]
fn a_real_probe_found_in_no_mirror_is_normal_and_leaves_nothing() {
    let tree = Tree::new("redirect-normal");
    assert_eq!(tree.hidden().probe(&tree.data()), Ok(Probe::Normal));
    assert!(!tree.data().exists(), "the folder it made is gone");
    fs::create_dir(tree.data()).unwrap();
    assert_eq!(tree.hidden().probe(&tree.data()), Ok(Probe::Normal));
    assert!(names(&tree.data()).is_empty(), "the probe file is gone");
}

#[test]
fn a_real_probe_whose_writes_land_in_a_mirror_is_hidden() {
    let tree = Tree::new("redirect-hidden");
    let hidden = tree.hidden_in(PKG);
    fill(&tree.copy_in(PKG));
    let want = Probe::Hidden {
        package: PKG.into(),
        app: "Claude".into(),
    };
    assert_eq!(hidden.probe(&tree.copy_in(PKG)), Ok(want));
    assert_eq!(
        names(&tree.copy_in(PKG)),
        [SETTINGS],
        "the probe file is gone"
    );
}

#[test]
fn the_probe_file_is_deleted_when_the_scan_fails() {
    let tree = Tree::new("redirect-scan");
    let root = tree.top.join("packages-file");
    fs::write(&root, b"no folder").unwrap();
    let hidden = WinHidden::at(tree.local.clone(), root);
    fs::create_dir(tree.data()).unwrap();
    assert_eq!(hidden.probe(&tree.data()), Err(HiddenError::Unreadable));
    assert!(names(&tree.data()).is_empty(), "the probe file is gone");
    fs::remove_dir(tree.data()).unwrap();
    assert_eq!(hidden.probe(&tree.data()), Err(HiddenError::Unreadable));
    assert!(!tree.data().exists(), "the folder it made is gone");
}

#[test]
fn a_folder_outside_app_data_is_normal_and_never_made() {
    let tree = Tree::new("redirect-outside");
    for dir in ["Elsewhere", "LocalLow"] {
        let outside = tree.top.join(dir).join(APP_DIR_NAME);
        assert_eq!(tree.hidden().probe(&outside), Ok(Probe::Normal));
        assert!(!outside.exists() && !outside.parent().unwrap().exists());
    }
}

#[test]
fn real_copies_list_only_mirrors_whose_app_folder_holds_something() {
    let tree = Tree::new("redirect-copies");
    fill(&tree.copy_in(PKG));
    fs::write(tree.copy_in(OTHER), b"a file, no folder").unwrap();
    fs::create_dir_all(tree.root.join("Empty_x").join(steps()).join(APP_DIR_NAME)).unwrap();
    let sibling = tree.mirror(OTHER).join(format!("{APP_DIR_NAME}X"));
    fill(&sibling);
    fs::create_dir(tree.data()).unwrap();
    let want = vec![tree.listed(PKG, "Claude")];
    assert_eq!(tree.hidden().copies(), Ok(want));
    let forged = HiddenCopy::new(sibling.clone(), OTHER.into(), "Other".into());
    assert_eq!(tree.hidden().remove(&forged), Err(HiddenError::Refused));
    assert_eq!(names(&sibling), [SETTINGS]);
}

#[test]
fn remove_deletes_a_listed_copy_with_everything_in_it() {
    let tree = Tree::new("redirect-remove");
    let words = tree.copy_in(PKG).join("words");
    fs::create_dir_all(&words).unwrap();
    fs::write(words.join("sealed"), b"x").unwrap();
    let kept = tree.copy_in(PKG).join("settings.toml");
    fs::write(&kept, b"y").unwrap();
    let mut readonly = fs::metadata(&kept).unwrap().permissions();
    readonly.set_readonly(true);
    fs::set_permissions(&kept, readonly).unwrap();
    assert_eq!(tree.hidden().remove(&tree.listed(PKG, "Claude")), Ok(()));
    assert!(!tree.copy_in(PKG).exists() && tree.mirror(PKG).is_dir());
    assert_eq!(tree.hidden().copies(), Ok(vec![]));
}

#[test]
fn remove_refuses_the_data_folder_and_anything_not_listed() {
    let tree = Tree::new("redirect-refuse");
    fill(&tree.copy_in(PKG));
    fs::create_dir(tree.data()).unwrap();
    let forged = [
        HiddenCopy::new(tree.data(), PKG.into(), "Claude".into()),
        HiddenCopy::new(tree.mirror(PKG), PKG.into(), "Claude".into()),
        tree.listed(OTHER, "Other"),
    ];
    for copy in &forged {
        assert_eq!(tree.hidden().remove(copy), Err(HiddenError::Refused));
    }
    assert!(tree.data().is_dir() && tree.copy_in(PKG).is_dir());
}

#[test]
fn a_port_without_a_local_folder_finds_every_run_normal_and_cannot_list_or_delete() {
    let tree = Tree::new("redirect-none");
    let none = WinHidden {
        local: None,
        root: None,
    };
    assert_eq!(none.probe(&tree.data()), Ok(Probe::Normal));
    assert_eq!(none.copies(), Err(HiddenError::Unreadable));
    let copy = tree.listed(PKG, "Claude");
    assert_eq!(none.remove(&copy), Err(HiddenError::Unreadable));
    assert!(!tree.data().exists());
}

#[test]
fn a_port_without_a_packages_root_still_finds_an_outside_folder_normal() {
    let tree = Tree::new("redirect-noroot");
    let no_root = WinHidden {
        local: Some(tree.local.clone()),
        root: None,
    };
    let outside = tree.top.join("Elsewhere");
    assert_eq!(no_root.probe(&outside), Ok(Probe::Normal));
    assert_eq!(no_root.probe(&tree.data()), Err(HiddenError::Unreadable));
    assert!(!outside.exists() && !tree.data().exists());
}
