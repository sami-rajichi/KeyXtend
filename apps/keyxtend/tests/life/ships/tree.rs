//! The uiAccess app decodes no media, opens no box, hears no microphone, downloads nothing and uploads nothing
//! (ADR-0004, ADR-0034, ADR-0038, ADR-0042, ADR-0047): tidy bans `png`, `fdeflate`, `age`, `keepass`, the speech
//! engine, `cpal` and the HTTPS crate from its tree, and no crate it builds with, its tests' and the cloud relay's
//! included, reaches them, any of tidy's network crates or the worker, which alone does each.

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
/// The speech engine and the microphone's audio crate, which only the worker's `voice` job may use (ADR-0042).
const VOICE: [&str; 3] = ["transcribe-cpp", "transcribe-cpp-sys", "cpal"];
/// The HTTPS crate only the worker's `fetch` and `speech-net` jobs may use (ADR-0042, ADR-0047).
const NETWORK: [&str; 1] = ["kx-platform-net"];
/// The app's crate that relays a cloud dictation's sound to `speech-net` unread (ADR-0047 decision 3).
const RELAY: &str = "kx-platform";

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

/// Every package `from` reaches in `graph`, itself included.
fn reached(graph: &BTreeMap<String, BTreeSet<String>>, from: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::from([from.to_owned()]);
    let mut queue = vec![from.to_owned()];
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

/// The names in the root manifest's tidy list `list`.
fn tidy_list(list: &str) -> Vec<String> {
    let value = metadata(&["tidy", list]);
    let names = value
        .as_array()
        .unwrap()
        .iter()
        .filter_map(toml::Value::as_str);
    names.map(str::to_owned).collect()
}

#[test]
fn tidy_bans_the_image_decoders_and_the_box_formats_from_the_app() {
    let banned = tidy_list("banned_media");
    let mut formats = DECODERS.iter().chain(&BOX_FORMATS);
    assert!(formats.all(|d| banned.iter().any(|b| b == d)), "{banned:?}");
    assert_eq!(metadata(&["tidy", "app"]).as_str(), Some(APP));
}

#[test]
fn tidy_bans_the_engine_the_audio_and_the_network_crates_from_the_app() {
    let (media, network) = (tidy_list("banned_media"), tidy_list("banned_network"));
    assert!(
        VOICE.iter().all(|c| media.iter().any(|b| b == c)),
        "{media:?}"
    );
    assert!(
        NETWORK.iter().all(|c| network.iter().any(|b| b == c)),
        "{network:?}"
    );
}

#[test]
fn nothing_the_app_builds_with_reaches_a_decoder_a_box_format_a_voice_crate_or_the_worker() {
    let graph = graph();
    let reached = reached(&graph, APP);
    let worker = metadata(&["stage", "worker"]);
    let worker = worker.as_str().unwrap();
    assert!(graph.contains_key(worker), "the lock lists {worker}");
    assert!(!reached.contains(worker), "{APP} depends on {worker}");
    let all = DECODERS
        .iter()
        .chain(&BOX_FORMATS)
        .chain(&VOICE)
        .chain(&NETWORK);
    for banned in all {
        assert!(graph.contains_key(*banned), "the lock lists {banned}");
        assert!(!reached.contains(*banned), "{APP} reaches {banned}");
    }
}

#[test]
fn the_cloud_relay_builds_into_the_app_which_reaches_no_network_crate_and_the_worker_alone_does() {
    let graph = graph();
    let app = reached(&graph, APP);
    assert!(app.contains(RELAY), "{APP} builds the relay");
    let banned = tidy_list("banned_network");
    assert!(NETWORK.iter().all(|n| banned.iter().any(|b| b == n)));
    for network in &banned {
        assert!(!app.contains(network), "{APP} reaches {network}");
    }
    let worker = metadata(&["stage", "worker"]);
    let worker = reached(&graph, worker.as_str().unwrap());
    assert!(
        NETWORK.iter().all(|n| worker.contains(*n)),
        "the worker uploads, so the walk sees the network"
    );
}
