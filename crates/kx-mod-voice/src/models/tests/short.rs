//! The bar's short names (owner, 2026-11-22): each row has its own, no two alike, and an other file shows its name
//! without the extension.

use super::*;
use kx_module_api::ports::MB;
use std::collections::BTreeSet;

/// The owner's short names, in the table's order.
const SHORT: [&str; 10] = [
    "Whisper Small",
    "Whisper Turbo",
    "Whisper Large v3",
    "Parakeet",
    "Canary",
    "Nemotron",
    "Qwen3 0.6B",
    "Qwen3 1.7B",
    "Cohere",
    "Cohere Arabic",
];

#[test]
fn every_row_has_a_short_name_and_no_two_are_equal() {
    let shorts: Vec<&str> = shipped().rows().iter().map(|r| r.short.as_str()).collect();
    assert_eq!(shorts, SHORT, "the owner's names");
    let unique: BTreeSet<&str> = shorts.iter().copied().collect();
    assert_eq!(unique.len(), shorts.len(), "no two are equal");
    assert!(shorts.iter().all(|s| !s.trim().is_empty()), "none is blank");
}

#[test]
fn an_other_file_shows_its_name_without_the_extension() {
    for (name, short) in [
        ("my-model.gguf", "my-model"),
        ("ggml.small.q5.BIN", "ggml.small.q5"),
    ] {
        let other = Other {
            name: name.into(),
            bytes: MB,
        };
        assert_eq!(other.short(), short, "{name}");
    }
}
