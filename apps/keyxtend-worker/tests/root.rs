//! The root `Cargo.toml`'s settings about this worker: the stage builds it with features it has, none of which turns its
//! gate feature on, so the staged worker never takes a sound file (ADR-0044), and tidy F5 keeps its `voice` job from the
//! network crate and its `fetch` and `speech-net` jobs from the audio crate and the engine, each named here unrenamed
//! (ADR-0042, ADR-0047).
#![allow(clippy::unwrap_used, reason = "a failed read should fail the test")]

use toml::{Table, Value};

/// The root manifest, from this crate.
const ROOT_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
/// This crate's manifest.
const OWN_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
/// Each job's module with the crates it must never name.
const KEPT: [(&str, &[&str]); 3] = [
    ("src/voice", &["kx-platform-net"]),
    ("src/fetch", &["cpal", "transcribe-cpp"]),
    ("src/net", &["cpal", "transcribe-cpp"]),
];
/// The key that renames a dependency, which would leave F5 reading the wrong name.
const RENAME: &str = "package";

/// The TOML file at `path`.
fn table(path: &str) -> Table {
    std::fs::read_to_string(path).unwrap().parse().unwrap()
}

/// The crates `[workspace.metadata.tidy] apart` bans in this crate's `module`.
fn banned(root: &Table, module: &str) -> Vec<String> {
    let entries = root["workspace"]["metadata"]["tidy"]["apart"]
        .as_array()
        .unwrap();
    let ours = |e: &&Value| e["package"].as_str() == Some(env!("CARGO_PKG_NAME"));
    let entry = entries
        .iter()
        .filter(ours)
        .find(|e| e["module"].as_str() == Some(module));
    let names = entry
        .and_then(|e| e["banned"].as_array())
        .into_iter()
        .flatten();
    names.filter_map(Value::as_str).map(str::to_owned).collect()
}

/// This crate's dependency `name`, from any of its dependency tables, the per-target ones too.
fn dependency(own: &Table, name: &str) -> Option<Value> {
    let targets = own.get("target").and_then(Value::as_table).into_iter();
    let tables = targets.flat_map(|t| t.values().filter_map(|v| v.get("dependencies")));
    std::iter::once(&own["dependencies"])
        .chain(tables)
        .find_map(|deps| deps.get(name).cloned())
}

#[test]
fn tidy_keeps_each_job_from_the_other_jobs_crates_which_this_crate_names_unrenamed() {
    let (root, own) = (table(ROOT_MANIFEST), table(OWN_MANIFEST));
    for (module, crates) in KEPT {
        let listed = banned(&root, module);
        for name in crates {
            assert!(
                listed.iter().any(|b| b == name),
                "{module} keeps from {name}"
            );
            let dep = dependency(&own, name).unwrap_or_else(|| panic!("{name} is a dependency"));
            assert!(dep.get(RENAME).is_none(), "{name} keeps its name");
        }
    }
}

/// True when turning `feature` on turns `gate` on, itself or through the features it lists.
fn turns_on(features: &Table, feature: &str, gate: &str) -> bool {
    let listed = features.get(feature).and_then(Value::as_array);
    feature == gate
        || listed
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .any(|f| turns_on(features, f, gate))
}

#[test]
fn the_stage_builds_this_worker_with_its_own_features_and_never_its_gate() {
    let (root, own) = (table(ROOT_MANIFEST), table(OWN_MANIFEST));
    let stage = &root["workspace"]["metadata"]["stage"];
    let features = own["features"].as_table().unwrap();
    let gate = stage["worker_gate"]["feature"].as_str().unwrap();
    assert!(features.contains_key(gate), "{gate} is a feature");
    let staged = stage["worker_features"].as_array().unwrap();
    for feature in staged.iter().map(|f| f.as_str().unwrap()) {
        assert!(features.contains_key(feature), "{feature} is a feature");
        assert!(
            !turns_on(features, feature, gate),
            "{feature} turns {gate} on"
        );
    }
    let mut default = features["default"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str);
    assert!(
        !default.any(|f| turns_on(features, f, gate)),
        "the default build has no gate"
    );
}
