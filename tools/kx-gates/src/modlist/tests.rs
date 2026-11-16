//! The module list's pure parts: which processes are our own workers, and which modules sit in a folder.

use super::*;
use crate::g19chips::procs::Proc;

fn p(pid: u32, parent: u32, exe: &str) -> Proc {
    Proc {
        pid,
        parent,
        exe: exe.to_string(),
    }
}

#[test]
fn only_our_own_children_of_the_program_are_ours() {
    let all = [
        p(10, 1, "keyxtend-worker.exe"),
        p(11, 7, "KEYXTEND-WORKER.EXE"),
        p(12, 7, "keyxtend.exe"),
        p(13, 7, "keyxtend-worker.exe"),
    ];
    assert_eq!(kids(&all, 7, "keyxtend-worker.exe"), [11, 13]);
    assert!(kids(&all, 99, "keyxtend-worker.exe").is_empty());
}

#[test]
fn a_module_is_inside_a_folder_by_whole_parts_in_any_case() {
    let dir = Path::new(r"D:\dev\models\kx-plant");
    assert!(inside(
        Path::new(r"D:\dev\models\kx-plant\ggml-cpu.dll"),
        dir
    ));
    assert!(inside(Path::new(r"d:\DEV\Models\KX-PLANT\sub\a.dll"), dir));
    assert!(
        !inside(Path::new(r"D:\dev\models\kx-plant2\a.dll"), dir),
        "a longer name"
    );
    assert!(!inside(Path::new(r"D:\dev\models\a.dll"), dir));
    assert!(!inside(dir, dir), "the folder itself is no module");
}

#[test]
fn what_was_seen_counts_snapshots_and_modules_in_a_folder() {
    let mut seen = Seen::default();
    seen.add(Ok(vec![
        PathBuf::from(r"C:\t\copy\ggml-cpu-x64.dll"),
        PathBuf::from(r"C:\Windows\a.dll"),
    ]));
    seen.add(Err("denied".into()));
    seen.add(Ok(vec![PathBuf::from(r"C:\t\copy\ggml-cpu-x64.dll")]));
    assert_eq!((seen.snaps, seen.failed), (2, 1));
    assert_eq!(seen.under(Path::new(r"C:\t\copy")), 1, "each module once");
    assert_eq!(seen.under(Path::new(r"D:\plant")), 0);
    assert!(seen.has(Path::new(r"c:\T\COPY\GGML-CPU-X64.DLL")));
}
