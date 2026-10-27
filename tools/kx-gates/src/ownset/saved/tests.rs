//! Tests of the sidecar's bound.

use kx_test_support::tempdir::TempDir;

use super::*;

/// The store's cap, as a length.
fn cap() -> usize {
    usize::try_from(MAX_FILE_BYTES).expect("fits")
}

/// A kept file at `path` holding the store's cap of the widest byte.
fn widest(path: PathBuf) -> Saved {
    let bytes = Some(vec![u8::MAX; cap()]);
    Saved { path, bytes }
}

#[test]
fn the_largest_pair_the_store_allows_fits_the_sidecars_bound() {
    // Longer than any Windows path in UTF-16 units, and JSON doubles each backslash.
    let long = PathBuf::from("\\".repeat(usize::from(u16::MAX)));
    let pair = Pair {
        settings: widest(long.clone()),
        previous: widest(long),
    };
    let json = serde_json::to_vec(&pair).expect("JSON");
    let len = u64::try_from(json.len()).expect("fits");
    assert!(len <= side_cap(), "{len} bytes");
}

#[test]
fn a_sidecar_past_its_bound_is_refused_before_it_is_parsed() {
    let dir = TempDir::new("gates-saved").expect("a temp folder");
    let side = dir.path().join("side.json");
    let none = |name: &str| Saved {
        path: dir.path().join(name),
        bytes: None,
    };
    let pair = Pair {
        settings: none("a"),
        previous: none("b"),
    };
    let bound = usize::try_from(side_cap()).expect("fits");
    let mut json = serde_json::to_vec(&pair).expect("JSON");
    json.resize(bound + 1, b' ');
    fs::write(&side, &json).expect("written");
    let e = load(&side).expect_err("past the bound");
    assert!(e.contains(&side_cap().to_string()), "{e}");
    json.truncate(bound);
    fs::write(&side, &json).expect("written");
    assert_eq!(load(&side), Ok(Some(pair)), "at the bound it loads");
}
