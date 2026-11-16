//! A download's host list: a plain host takes only itself, and an `UNDER` entry takes the hosts under one site's
//! domain, never the domain itself nor a whole top-level one.

use crate::IpcError::BadField;
use crate::rules::{UNDER, UNDER_NAMES, covers, listed};

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
fn an_under_entry_of_a_whole_top_level_domain_takes_nothing() {
    for entry in [".co", ".", "..", ".hf.", "..co"] {
        assert!(!covers(entry, "us.aws.cdn.hf.co"), "{entry}");
        assert_eq!(listed(entry), Err(BadField), "{entry}");
    }
}

#[test]
fn a_listed_entry_is_a_plain_host_or_under_a_site_of_two_names() {
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
