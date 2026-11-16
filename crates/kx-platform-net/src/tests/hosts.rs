//! A download's host list: a redirect's host is listed by name or under a `.` entry, and a first address by name only.

use super::HOSTS;
use crate::{NetError, Target};
use std::borrow::Cow;

#[test]
fn only_a_listed_host_passes() {
    let listed = |u: &str| Target::parse(u).and_then(|t| t.listed(&HOSTS));
    assert_eq!(listed("https://cdn.example/x"), Ok(()));
    assert_eq!(listed("https://CDN.example/x"), Ok(()));
    assert_eq!(listed("https://evil.example/x"), Err(NetError::Unlisted));
    assert_eq!(listed("https://cdn.example./x"), Err(NetError::Unlisted));
    assert_eq!(
        listed("https://cdn.example.evil.example/x"),
        Err(NetError::Unlisted)
    );
}

#[test]
fn an_under_entry_lets_each_host_below_its_site_pass_and_nothing_else() {
    let hosts = [Cow::Borrowed("models.example"), Cow::Borrowed(".hf.co")];
    let listed = |u: &str| Target::parse(u).and_then(|t| t.listed(&hosts));
    for ok in [
        "us.aws.cdn.hf.co",
        "CDN-lfs-eu-1.hf.co",
        "cas-bridge.xethub.hf.co",
    ] {
        assert_eq!(listed(&format!("https://{ok}/x")), Ok(()), "{ok}");
    }
    for bad in [
        "hf.co",
        "evilhf.co",
        "hf.co.evil.example",
        "x.models.example",
    ] {
        let got = listed(&format!("https://{bad}/x"));
        assert_eq!(got, Err(NetError::Unlisted), "{bad}");
    }
    let wide = [Cow::Borrowed(".co")];
    let host = Target::parse("https://us.aws.cdn.hf.co/x").unwrap();
    assert_eq!(
        host.listed(&wide),
        Err(NetError::Unlisted),
        "a bare top-level domain"
    );
}

#[test]
fn only_a_plain_entry_names_a_first_address() {
    let hosts = [Cow::Borrowed("models.example"), Cow::Borrowed(".hf.co")];
    let named = |u: &str| Target::parse(u).and_then(|t| t.named(&hosts));
    assert_eq!(named("https://models.example/x"), Ok(()));
    assert_eq!(named("https://MODELS.example:8443/x"), Ok(()));
    for bad in ["cdn.hf.co", "hf.co", "x.models.example", "other.example"] {
        let got = named(&format!("https://{bad}/x"));
        assert_eq!(got, Err(NetError::Unlisted), "{bad}");
    }
}
