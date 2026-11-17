//! A download's host list: a plain host takes only itself, and an `UNDER` entry takes the hosts under its domain,
//! never the domain itself nor a bare top-level one. A first address needs a plain entry by name.

use crate::IpcError::BadField;
use crate::rules::{UNDER, UNDER_NAMES, covers, listed, names};

#[test]
fn only_a_plain_entry_names_a_host_and_only_its_own() {
    assert!(names("huggingface.co", "huggingface.co"));
    let cases = [
        (".hf.co", "cdn.hf.co"),
        (".hf.co", "hf.co"),
        (".hf.co", ".hf.co"),
        ("huggingface.co", "cdn.huggingface.co"),
        ("huggingface.co", "huggingface.co.evil.example"),
        ("huggingface.co", "HUGGINGFACE.co"),
        ("HF.co", "HF.co"),
        ("", ""),
    ];
    for (entry, host) in cases {
        assert!(!names(entry, host), "{entry} {host}");
    }
}

#[test]
fn a_plain_entry_takes_only_its_own_host() {
    assert!(covers("huggingface.co", "huggingface.co"));
    for other in [
        "cdn.huggingface.co",
        "huggingface.com",
        "huggingface.co.evil.example",
    ] {
        assert!(!covers("huggingface.co", other), "{other}");
    }
}

#[test]
fn an_under_entry_takes_each_host_below_it_at_any_depth() {
    for host in [
        "us.aws.cdn.hf.co",
        "us.gcp.cdn.hf.co",
        "cdn-lfs-eu-1.hf.co",
        "cas-bridge.xethub.hf.co",
        "transfer.xethub-eu.hf.co",
    ] {
        assert!(covers(".hf.co", host), "{host}");
    }
}

#[test]
fn an_under_entry_never_takes_its_domain_a_look_alike_or_a_longer_name() {
    for host in [
        "hf.co",
        ".hf.co",
        "..hf.co",
        "a..hf.co",
        "evilhf.co",
        "hf.co.evil.example",
        "x.hf.com",
    ] {
        assert!(!covers(".hf.co", host), "{host}");
    }
}

#[test]
fn an_under_entry_of_a_bare_top_level_domain_takes_nothing() {
    for entry in [".co", ".", "..", ".hf.", "..co"] {
        assert!(!covers(entry, "us.aws.cdn.hf.co"), "{entry}");
        assert_eq!(listed(entry), Err(BadField), "{entry}");
    }
}

#[test]
fn a_two_name_public_suffix_passes_the_rule_so_the_table_test_pins_its_entries() {
    for suffix in [".co.uk", ".github.io"] {
        assert_eq!(listed(suffix), Ok(()), "{suffix}");
    }
}

#[test]
fn a_listed_entry_is_a_plain_host_or_under_a_domain_of_two_names() {
    assert_eq!(
        (UNDER, UNDER_NAMES),
        ('.', 2),
        "the rule as the notes state it"
    );
    for good in ["huggingface.co", "a1", ".hf.co", ".cdn.hf.co"] {
        assert_eq!(listed(good), Ok(()), "{good}");
    }
    for bad in ["HF.co", ".HF.co", "a b", ".hf co", ".hf..co", ""] {
        assert_eq!(listed(bad), Err(BadField), "{bad}");
    }
}
