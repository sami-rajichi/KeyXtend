//! The environment table parses, keeps no engine, loader or search-path variable, and builds a sorted block with the
//! system folders as the only search path.

use super::*;

/// What a test system has: a kept name in another case, the engine's and loader's variables, our own and a path.
fn ours() -> Vec<(OsString, OsString)> {
    [
        ("systemroot", r"C:\Windows"),
        ("GGML_VK_VISIBLE_DEVICES", "0"),
        ("TRANSCRIBE_MODEL", r"C:\x.bin"),
        ("VK_ICD_FILENAMES", r"C:\evil.json"),
        ("CARGO_HOME", r"D:\dev\cargo"),
        ("Path", r"C:\evil;C:\Windows"),
        ("TEMP", r"C:\Temp"),
    ]
    .map(|(n, v)| (OsString::from(n), OsString::from(v)))
    .to_vec()
}

#[test]
fn the_shipped_table_parses_and_keeps_no_banned_or_search_path_name() {
    let table = table().expect("the shipped table parses");
    let starts = ["GGML_", "TRANSCRIBE_", "VK_"];
    for name in &table.keep {
        let upper = name.to_ascii_uppercase();
        assert!(!starts.iter().any(|s| upper.starts_with(s)), "{name}");
        assert!(!name.eq_ignore_ascii_case(&table.path), "{name}");
    }
}

#[test]
fn a_table_that_keeps_a_banned_name_or_the_search_path_is_refused() {
    let base = "path = \"PATH\"\npath_gap = \";\"\nbanned = [\"VK_\"]\n";
    assert!(Table::parse(&format!("keep = [\"vk_layer_path\"]\n{base}")).is_none());
    assert!(Table::parse(&format!("keep = [\"Path\"]\n{base}")).is_none());
    assert!(Table::parse(&format!("keep = [\"TEMP\"]\n{base}")).is_some());
}

#[test]
fn the_worker_keeps_only_kept_names_and_the_system_folders_as_its_path() {
    let folders = [
        PathBuf::from(r"C:\Windows\system32"),
        PathBuf::from(r"C:\Windows"),
    ];
    let vars = table().unwrap().vars(ours(), &folders);
    let names: Vec<&OsStr> = vars.iter().map(|(n, _)| n.as_os_str()).collect();
    assert_eq!(names, ["PATH", "systemroot", "TEMP"], "sorted in any case");
    assert_eq!(vars[0].1, r"C:\Windows\system32;C:\Windows");
}

#[test]
fn a_block_ends_each_variable_and_itself_with_a_nul() {
    let vars = [("A", "1"), ("B", "")].map(|(n, v)| (OsString::from(n), OsString::from(v)));
    let wide: Vec<u16> = "A=1\0B=\0\0".encode_utf16().collect();
    assert_eq!(block(&vars), wide);
}
