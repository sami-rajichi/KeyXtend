//! The test build's portable data folder around a clipboard gate, never the owner's: a folder the gate makes carries
//! its marker and goes at the end, a marked leftover of a hard stop is made afresh, and an unmarked one is never removed.
#![cfg(windows)]

use std::fs;
use std::io::ErrorKind;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};

use kx_platform::{CLIP_DIR, PORTABLE_DIR, SEALED_DIR};
use kx_settings::Files;
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

use crate::product;

/// The first of `path` and its parent folders that `link` finds is a junction or link, which could lead elsewhere.
pub(crate) fn first_link(path: &Path, link: impl Fn(&Path) -> bool) -> Option<&Path> {
    path.ancestors().find(|p| link(p))
}

/// True when `path` is a reparse point, such as a junction or symlink.
fn reparse(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|m| m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0)
}

/// Deletes folder `dir`, when there is one.
pub(crate) fn remove(dir: &Path) -> Result<(), String> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(format!("{}: {e}", dir.display())),
        _ => Ok(()),
    }
}

/// True when `name` is, in any case, one of the app's own folders, the history's among them, or a name the product's
/// settings layer writes in its data folder, so a marker by that name would mark a folder the gate never made.
pub(crate) fn taken(name: &str) -> bool {
    let f = Files::new(Path::new(""));
    let product = [&f.settings, &f.previous, &f.broken, &f.temp, &f.logs];
    let app = [CLIP_DIR, SEALED_DIR];
    let mut names = product.iter().filter_map(|p| p.to_str()).chain(app);
    names.any(|n| n.eq_ignore_ascii_case(name))
}

/// True when data folder `data` holds the gate's `marker`, so the gate made it.
pub(crate) fn marked(data: &Path, marker: &str) -> bool {
    data.join(marker).is_file()
}

/// Readies data folder `data`: a marked leftover is deleted, a missing folder is made with `marker` in it, and an
/// unmarked one stays as it is. True when this call made it.
pub(crate) fn prepare(data: &Path, marker: &str) -> Result<bool, String> {
    if marked(data, marker) {
        remove(data)?;
    }
    if data.exists() {
        return Ok(false);
    }
    let fail = |e: std::io::Error| format!("{}: {e}", data.display());
    fs::create_dir_all(data).map_err(fail)?;
    fs::write(data.join(marker), b"").map_err(fail)?;
    Ok(true)
}

/// Error `e`, after deleting data folder `data` when it carries `marker`.
pub(crate) fn undone(data: &Path, marker: &str, e: String) -> String {
    if !marked(data, marker) {
        return e;
    }
    match remove(data) {
        Err(left) => format!("{e}; {left}"),
        Ok(()) => e,
    }
}

/// Ends a run on data folder `data`: a marked folder is deleted, an unmarked one only loses the app's history folder.
pub(crate) fn finish(data: &Path, marker: &str) -> Result<(), String> {
    if marked(data, marker) {
        remove(data)
    } else {
        remove(&data.join(CLIP_DIR))
    }
}

/// The data folder beside `program`, readied with `marker`, so the copy is portable, once the product reads it there;
/// refused when it or a folder above it is a junction or link, which could lead to the owner's folder.
pub(crate) fn own(program: &Path, marker: &str) -> Result<PathBuf, String> {
    let data = program
        .parent()
        .ok_or("the test build has no folder")?
        .join(PORTABLE_DIR);
    if let Some(link) = first_link(&data, reparse) {
        return Err(format!(
            "{} is a junction or link, so the copy's data folder could be another",
            link.display()
        ));
    }
    prepare(&data, marker)?;
    match product::files(program) {
        Ok(f) if f.dir == data => Ok(data),
        Ok(_) => {
            let e = format!(
                "{} is not the copy's data folder, so it would use another",
                data.display()
            );
            Err(undone(&data, marker, e))
        }
        Err(e) => Err(undone(&data, marker, e)),
    }
}

#[cfg(test)]
mod tests {
    use kx_test_support::tempdir::TempDir;

    use super::*;

    const MARK: &str = "made-by-test";
    /// A test build's program, never started.
    const PROGRAM: &str = "test-build.exe";

    /// A temp folder and its `data` folder path, not made yet.
    fn data() -> (TempDir, PathBuf) {
        let dir = TempDir::new("gates-clipdata").expect("a temp folder");
        let data = dir.path().join(PORTABLE_DIR);
        (dir, data)
    }

    /// Fills `data` with a settings file and a history file, as a run leaves them.
    fn fill(data: &Path) {
        fs::create_dir_all(data.join(CLIP_DIR)).expect("history folder");
        fs::write(data.join("settings.toml"), "x = 1").expect("settings");
        fs::write(data.join(CLIP_DIR).join("list"), "entries").expect("history");
    }

    #[test]
    fn a_missing_folder_is_made_marked_and_goes_at_the_end() {
        let (_dir, data) = data();
        assert_eq!(prepare(&data, MARK), Ok(true));
        assert!(marked(&data, MARK));
        fill(&data);
        assert_eq!(finish(&data, MARK), Ok(()));
        assert!(!data.exists());
    }

    #[test]
    fn a_marked_leftover_is_made_afresh_and_still_marked() {
        let (_dir, data) = data();
        prepare(&data, MARK).expect("made");
        fill(&data);
        assert_eq!(prepare(&data, MARK), Ok(true));
        assert!(marked(&data, MARK));
        assert!(!data.join("settings.toml").exists() && !data.join(CLIP_DIR).exists());
    }

    #[test]
    fn an_unmarked_folder_stays_and_only_loses_its_history() {
        let (_dir, data) = data();
        fill(&data);
        assert_eq!(prepare(&data, MARK), Ok(false));
        assert!(!marked(&data, MARK));
        assert_eq!(undone(&data, MARK, "boom".into()), "boom");
        assert!(
            data.join("settings.toml").exists(),
            "undone never removes it"
        );
        assert_eq!(finish(&data, MARK), Ok(()));
        assert!(data.join("settings.toml").exists() && !data.join(CLIP_DIR).exists());
    }

    #[test]
    fn the_products_own_names_and_the_history_are_taken_in_any_case() {
        for name in [
            "settings.toml",
            "SETTINGS.previous.toml",
            "settings.broken.toml",
            "settings.toml.tmp",
            "Logs",
        ] {
            assert!(taken(name), "{name}");
        }
        assert!(taken(CLIP_DIR), "the app's own history folder");
        assert!(taken(&CLIP_DIR.to_uppercase()), "in any case");
        assert!(taken("Words"), "the learned words' folder");
        assert!(!taken(MARK));
        assert!(!taken("settings"), "only whole names");
    }

    #[test]
    fn the_first_junction_on_the_way_is_found_from_the_folder_up() {
        let data = Path::new(r"D:\build\debug\data");
        let at = |hit: &'static str| move |p: &Path| p == Path::new(hit);
        assert_eq!(first_link(data, at(r"D:\build\debug\data")), Some(data));
        let up = Some(Path::new(r"D:\build"));
        assert_eq!(first_link(data, at(r"D:\build")), up, "a folder above");
        assert_eq!(first_link(data, |_| false), None);
    }

    /// Links folder `link` to `target` with a junction, which Windows allows without admin rights.
    fn junction(link: &Path, target: &Path) {
        let out = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .expect("cmd runs");
        assert!(out.status.success(), "mklink /J failed: {out:?}");
    }

    #[test]
    fn a_data_folder_through_a_junction_is_refused_and_nothing_is_made() {
        let dir = TempDir::new("gates-clipdata").expect("a temp folder");
        let (real, link, plain) = (
            dir.path().join("real"),
            dir.path().join("build"),
            dir.path().join("plain"),
        );
        fs::create_dir(&real).expect("the real folder");
        fs::create_dir(&plain).expect("a plain build folder");
        junction(&link, &real);
        junction(&plain.join(PORTABLE_DIR), &real);
        for (build, at) in [(&link, link.clone()), (&plain, plain.join(PORTABLE_DIR))] {
            let e = own(&build.join(PROGRAM), MARK).expect_err("a junction on the way");
            assert!(e.contains(&at.display().to_string()), "{e}");
        }
        assert!(
            !real.join(MARK).exists() && !real.join(PORTABLE_DIR).exists(),
            "nothing made"
        );
        assert_eq!(first_link(&real, reparse), None, "a plain folder");
        assert_eq!(own(&real.join(PROGRAM), MARK), Ok(real.join(PORTABLE_DIR)));
    }

    #[test]
    fn undone_deletes_only_a_marked_folder() {
        let (_dir, data) = data();
        prepare(&data, MARK).expect("made");
        fill(&data);
        assert_eq!(undone(&data, MARK, "boom".into()), "boom");
        assert!(!data.exists());
        assert_eq!(remove(&data), Ok(()), "a missing folder is no error");
    }
}
