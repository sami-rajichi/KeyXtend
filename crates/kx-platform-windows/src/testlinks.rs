//! Test helpers that make junctions and set file attributes with Windows' own tools, each started with no window.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use windows::Win32::System::Threading::CREATE_NO_WINDOW;

/// Makes a junction at `link` to the folder `target`; a junction needs no admin rights.
pub(crate) fn junction(link: &Path, target: &Path) {
    run(Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target));
}

/// Sets the attribute `flag` (`+h` hidden, `+s` system) on `path`.
pub(crate) fn attrib(flag: &str, path: &Path) {
    run(Command::new("attrib").arg(flag).arg(path));
}

/// Runs `command` with no console window; it must succeed.
fn run(command: &mut Command) {
    let done = command.creation_flags(CREATE_NO_WINDOW.0).output().unwrap();
    assert!(done.status.success(), "{command:?} failed: {done:?}");
}
