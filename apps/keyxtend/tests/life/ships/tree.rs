//! The uiAccess app decodes no media and opens no box (ADR-0004, ADR-0034, ADR-0038): tidy bans `png`, `fdeflate`,
//! `age` and `keepass` from its tree, and no crate it builds with, its tests' included, reaches them or the worker,
//! which alone decodes images, opens boxes and reads `KeePass` files.

use std::collections::{BTreeMap, BTreeSet};

/// The lock file, which lists every package with its dependencies of every kind.
const LOCK: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"));
/// The root manifest, with tidy's lists and the stage's worker.
const ROOT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml"));
/// The app's package, where the walk starts.
const APP: &str = env!("CARGO_PKG_NAME");
/// The image decoders only the worker may use.
const DECODERS: [&str; 2] = ["png", "fdeflate"];
/// The formats of the backup box and of `KeePass`, which only the worker's vault job may use.
const BOX_FORMATS: [&str; 2] = ["age", "keepass"];

/// Each package's dependency names, its versions merged: the lock names one as `name`, `name version` or more.
fn graph() -> BTreeMap<String, BTreeSet<String>> {
    let lock: toml::Table = LOCK.parse().unwrap();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in lock["package"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap().to_owned();
        let deps = package.get("dependencies").and_then(toml::Value::as_array);
        let names = deps.into_iter().flatten().filter_map(toml::Value::as_str);
        let firsts = names.filter_map(|d| d.split_whitespace().next());
        graph
            .entry(name)
            .or_default()
            .extend(firsts.map(str::to_owned));
    }
    graph
}

/// Every package the app reaches in `graph`, itself included.
fn reached(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
    let mut seen = BTreeSet::from([APP.to_owned()]);
    let mut queue = vec![APP.to_owned()];
    while let Some(name) = queue.pop() {
        for dep in graph.get(&name).into_iter().flatten() {
            if seen.insert(dep.clone()) {
                queue.push(dep.clone());
            }
        }
    }
    seen
}

/// The value at `path` in the root manifest's `[workspace.metadata]`.
fn metadata(path: &[&str]) -> toml::Value {
    let root: toml::Table = ROOT.parse().unwrap();
    let start = root["workspace"]["metadata"].clone();
    path.iter().fold(start, |v, key| v[*key].clone())
}

#[test]
fn tidy_bans_the_image_decoders_and_the_box_formats_from_the_app() {
    let banned = metadata(&["tidy", "banned_media"]);
    let banned: Vec<&str> = banned
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_str)
        .collect();
    let formats = DECODERS.iter().chain(&BOX_FORMATS);
    assert!(formats.copied().all(|d| banned.contains(&d)), "{banned:?}");
    assert_eq!(metadata(&["tidy", "app"]).as_str(), Some(APP));
}

#[test]
fn nothing_the_app_builds_with_reaches_a_decoder_a_box_format_or_the_worker() {
    let graph = graph();
    let reached = reached(&graph);
    let worker = metadata(&["stage", "worker"]);
    let worker = worker.as_str().unwrap();
    assert!(graph.contains_key(worker), "the lock lists {worker}");
    assert!(!reached.contains(worker), "{APP} depends on {worker}");
    for banned in DECODERS.iter().chain(&BOX_FORMATS) {
        assert!(graph.contains_key(*banned), "the lock lists {banned}");
        assert!(!reached.contains(*banned), "{APP} reaches {banned}");
    }
}
