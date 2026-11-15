//! The worker's home, read from a real child started as the worker is: its own folder as its working folder, and
//! only the kept variables, with the system folders alone as its search path.

use super::*;
use std::io::Read;

/// What the system's command shell defines on its own when it starts.
const SHELL_OWN: [&str; 3] = ["COMSPEC", "PATHEXT", "PROMPT"];

#[test]
fn a_child_runs_in_its_own_folder_with_only_the_kept_variables() {
    let windows = PathBuf::from(env::var_os("SystemRoot").unwrap());
    let system = windows.join("System32");
    let args = ["/d", "/c", "cd&set"].map(String::from).to_vec();
    let launch = Launch {
        args,
        memory_mb: CFG.memory_mb,
    };
    let (input, mut output, mut child) = WinSpawner::at(system.join("cmd.exe"))
        .start(&launch)
        .unwrap();
    drop(input);
    let mut text = String::new();
    output.read_to_string(&mut text).unwrap();
    assert!(child.code(WAIT).is_some(), "the shell ended");
    let mut lines = text.lines();
    let here = lines.next().unwrap_or_default();
    assert!(
        here.eq_ignore_ascii_case(&system.to_string_lossy()),
        "its own folder: {here}"
    );
    let table = super::super::env::table().unwrap();
    let root = windows.to_string_lossy().to_lowercase();
    for (name, value) in lines.filter_map(|l| l.split_once('=')) {
        if name.eq_ignore_ascii_case(&table.path) {
            let system_only = value
                .split(';')
                .all(|f| f.to_lowercase().starts_with(&root));
            assert!(system_only, "a search path of the system folders alone");
            continue;
        }
        let kept = table
            .keep
            .iter()
            .chain(&SHELL_OWN.map(String::from))
            .any(|k| k.eq_ignore_ascii_case(name));
        assert!(kept, "{name} reached the worker");
    }
}

#[test]
fn the_worker_writes_renames_and_deletes_in_a_guarded_models_folder() {
    let dir = env::temp_dir().join(format!("kx-guarded-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&dir));
    crate::voice::make_guarded(&dir).unwrap();
    let (part, file) = (dir.join("a.part"), dir.join("a.gguf"));
    let steps = format!(
        "echo x>{}&&ren {} a.gguf&&del {}",
        part.display(),
        part.display(),
        file.display()
    );
    let shell = PathBuf::from(env::var_os("SystemRoot").unwrap()).join(r"System32\cmd.exe");
    let launch = Launch {
        args: vec!["/d".into(), "/c".into(), steps],
        memory_mb: CFG.memory_mb,
    };
    let (input, _output, mut child) = WinSpawner::at(shell).start(&launch).unwrap();
    drop(input);
    let code = child.code(WAIT);
    let left = (part.exists(), file.exists());
    drop(std::fs::remove_dir_all(&dir));
    assert_eq!(code, Some(0), "every step passed");
    assert_eq!(left, (false, false), "written, renamed and deleted");
}
