//! After `dll::pin`, a library named without a folder loads only from the app's folder or System32: a copy in the
//! working folder, which loaded before, no longer does. It changes the whole process, so this binary holds one test.
#![cfg(windows)]
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    reason = "loads and frees a library by name; a failed setup should fail the test"
)]

use kx_platform_windows::dll;
use kx_test_support::tempdir::TempDir;
use std::env;
use std::path::PathBuf;
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::LoadLibraryW;
use windows::core::HSTRING;

/// A small System32 library, which the probe copies.
const SOURCE: &str = "version.dll";
/// The copy's name, which no folder Windows searches holds.
const PROBE: &str = "kx-dll-probe.dll";
/// The variable naming the Windows folder.
const SYSTEM_ROOT: &str = "SystemRoot";

/// True when library `name` loads by name alone; it is freed at once.
fn loads(name: &str) -> bool {
    // SAFETY: loads a library by name; only a copy of a System32 library or that library itself can match.
    let Ok(module) = (unsafe { LoadLibraryW(&HSTRING::from(name)) }) else {
        return false;
    };
    // SAFETY: frees the handle the load above returned, once.
    let _ = unsafe { FreeLibrary(module) };
    true
}

#[test]
fn after_pin_the_working_folder_lends_no_library() {
    let dir = TempDir::new("dll-probe").unwrap();
    let system = PathBuf::from(env::var_os(SYSTEM_ROOT).unwrap()).join("System32");
    std::fs::copy(system.join(SOURCE), dir.path().join(PROBE)).unwrap();
    let home = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();
    let before = loads(PROBE);
    dll::pin().unwrap();
    let after = loads(PROBE);
    let system_still = loads(SOURCE);
    env::set_current_dir(home).unwrap();
    assert!(before, "the working folder is searched before");
    assert!(!after, "and not after");
    assert!(system_still, "System32 still is");
}
