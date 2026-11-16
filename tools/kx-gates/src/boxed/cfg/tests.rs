//! The box proofs' settings: the shipped ones pass, each bad key is refused, the worker is the real install's as the
//! root manifest names it, and the folder sits at a second drive's root.

use super::*;
use crate::config::load;

/// The root manifest, whose dev-install and stage settings name the installed worker's folder.
const ROOT_MANIFEST: &str = include_str!("../../../../../Cargo.toml");

fn shipped() -> BoxProofs {
    load().expect("the settings load").boxed
}

#[test]
fn the_shipped_proofs_pass_their_check_and_each_bad_key_is_refused() {
    let b = shipped();
    assert_eq!(b.check(), Ok(()));
    let bad: [fn(&mut BoxProofs); 9] = [
        |b| b.worker = r"keyxtend\keyxtend-worker.exe".into(),
        |b| b.worker = r"C:\Program Files\KeyXtend-dev\keyxtend\keyxtend.exe".into(),
        |b| b.drive = "D:".into(),
        |b| b.drive = r"D:\dev".into(),
        |b| b.drive = r"\server\share\".into(),
        |b| b.folder = "..".into(),
        |b| b.folder = r"a\b".into(),
        |b| b.lang = "xx".into(),
        |b| b.lang = String::new(),
    ];
    for (i, spoil) in bad.iter().enumerate() {
        let mut b = shipped();
        spoil(&mut b);
        assert!(b.check().is_err(), "case {i} passed");
    }
}

#[test]
fn the_worker_is_the_real_installs_as_the_root_manifest_names_it() {
    let root: toml::Table = ROOT_MANIFEST.parse().expect("the root manifest is TOML");
    let meta = &root["workspace"]["metadata"];
    let (dir, stage) = (meta["devtools"]["install_dir"].as_str(), &meta["stage"]);
    let (app, worker) = (stage["app"].as_str(), stage["worker"].as_str());
    let tail = format!(
        r"\{}\{}\{}.exe",
        dir.unwrap(),
        app.unwrap(),
        worker.unwrap()
    );
    assert!(shipped().worker.ends_with(&tail), "{tail}");
}

#[test]
fn only_a_drives_root_alone_is_a_root() {
    for root in [r"D:\", r"e:\"] {
        assert!(is_root(root), "{root}");
    }
    for not in [
        "D:",
        r"D:\dev",
        "",
        r"\",
        r"\server\share\",
        r"\?\D:\",
        "dev",
    ] {
        assert!(!is_root(not), "{not:?}");
    }
}

#[test]
fn the_folder_goes_on_a_second_drive_never_the_system_one() {
    assert_eq!(second(r"D:\", Some("C:")), Ok(()));
    let same = second(r"c:\", Some("C:")).expect_err("the system drive");
    assert!(same.contains("system drive"), "{same}");
    assert!(second(r"D:\", None).is_err(), "unknown");
}

#[test]
fn the_guarded_folder_sits_at_the_drives_root() {
    let mut b = shipped();
    (b.drive, b.folder) = (r"E:\".into(), "kx-proof".into());
    assert_eq!(b.guarded(), PathBuf::from(r"E:\kx-proof"));
}
