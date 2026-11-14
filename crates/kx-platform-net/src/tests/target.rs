//! Reading an address and a redirect's `Location`: HTTPS only, a plain host, and the request path.

use super::HOSTS;
use crate::{NetError, Target};
use kx_ipc::{HTTPS, MAX_HOST_BYTES, MAX_URL_BYTES};

/// The parts `url` reads as, or why not.
fn parts(url: &str) -> Result<(String, u16, String), NetError> {
    Target::parse(url).map(|t| (t.host().to_owned(), t.port(), t.path().to_owned()))
}

/// `base` joined with `to`, written back as an address.
fn joined(base: &str, to: &str) -> Result<String, NetError> {
    Target::parse(base)?.join(to).map(|t| t.to_string())
}

#[test]
fn an_https_address_reads_as_host_port_and_path() {
    let want = |p: &str| Ok(("models.example".to_owned(), 443, p.to_owned()));
    assert_eq!(parts("https://models.example/a/b.bin"), want("/a/b.bin"));
    assert_eq!(parts("https://models.example"), want("/"));
    assert_eq!(parts("https://models.example?x=1"), want("/?x=1"));
    assert_eq!(parts("https://models.example/a?x=1#top"), want("/a?x=1"));
}

#[test]
fn a_port_is_read_and_shown_only_when_not_https_own() {
    assert_eq!(
        parts("https://models.example:8443/a"),
        Ok(("models.example".to_owned(), 8443, "/a".to_owned()))
    );
    let shown = |u: &str| Target::parse(u).map(|t| t.to_string());
    assert_eq!(
        shown("https://models.example:443/a"),
        Ok("https://models.example/a".to_owned())
    );
    assert_eq!(
        shown("https://models.example:8443/a"),
        Ok("https://models.example:8443/a".to_owned())
    );
}

#[test]
fn the_scheme_and_host_are_read_in_any_case_and_kept_lower() {
    assert_eq!(
        parts("HTTPS://Models.Example/A"),
        Ok(("models.example".to_owned(), 443, "/A".to_owned()))
    );
}

#[test]
fn any_scheme_but_https_is_refused() {
    for url in [
        "http://models.example/a",
        "ftp://models.example/a",
        "file:///C:/a",
        "models.example/a",
        "//x/a",
    ] {
        assert_eq!(parts(url), Err(NetError::NotHttps), "{url}");
    }
}

#[test]
fn a_host_that_is_not_a_plain_name_is_refused() {
    for url in [
        "https:///a",
        "https://user@models.example/a",
        "https://models.example@evil.example/a",
        "https://[::1]/a",
        "https://models_example/a",
        "https://models.example\\@evil.example/a",
        "https://%6dodels.example/a",
    ] {
        assert_eq!(parts(url), Err(NetError::NotHttps), "{url}");
    }
    let long = format!("{HTTPS}{}/a", "a".repeat(MAX_HOST_BYTES + 1));
    assert_eq!(parts(&long), Err(NetError::NotHttps));
}

#[test]
fn a_bad_port_is_refused() {
    for port in ["", "0", "65536", "44a", "-1"] {
        let url = format!("https://models.example:{port}/a");
        assert_eq!(parts(&url), Err(NetError::NotHttps), "{url}");
    }
}

#[test]
fn an_address_with_a_space_a_control_or_past_the_cap_is_refused() {
    for url in [
        "https://models.example/a b",
        "https://models.example/a\n",
        "https://models.example/é",
    ] {
        assert_eq!(parts(url), Err(NetError::NotHttps), "{url:?}");
    }
    let root = "https://models.example/";
    let at_cap = format!("{root}{}", "a".repeat(MAX_URL_BYTES - root.len()));
    assert_eq!(at_cap.len(), MAX_URL_BYTES);
    assert!(parts(&at_cap).is_ok());
    assert_eq!(parts(&format!("{at_cap}a")), Err(NetError::NotHttps));
}

#[test]
fn a_full_or_scheme_relative_location_names_its_own_host() {
    let base = "https://models.example/a/b.bin";
    assert_eq!(
        joined(base, "https://cdn.example/x?s=1"),
        Ok("https://cdn.example/x?s=1".to_owned())
    );
    assert_eq!(
        joined(base, "//cdn.example/x"),
        Ok("https://cdn.example/x".to_owned())
    );
}

#[test]
fn a_relative_location_stays_on_the_same_host() {
    let base = "https://models.example:8443/a/b.bin?old=1";
    let on = |p: &str| Ok(format!("https://models.example:8443{p}"));
    assert_eq!(joined(base, "/x/y.bin"), on("/x/y.bin"));
    assert_eq!(joined(base, "?new=2"), on("/a/b.bin?new=2"));
    assert_eq!(joined(base, "c.bin#frag"), on("/a/c.bin"));
}

#[test]
fn a_location_that_is_not_https_is_refused() {
    let base = "https://models.example/a";
    for to in [
        "http://cdn.example/x",
        "HTTP://cdn.example/x",
        "javascript:alert(1)",
        "data:text/plain,x",
        "",
    ] {
        assert_eq!(joined(base, to), Err(NetError::NotHttps), "{to}");
    }
    assert_eq!(
        joined(base, "https://user@cdn.example/x"),
        Err(NetError::NotHttps)
    );
}

#[test]
fn an_address_built_to_fool_a_reader_is_refused() {
    for url in [
        "https://models.example:443:80/a",
        "https://models.example:+443/a",
        "https://models.example:99999/a",
        "https://::1/a",
        "https://[::1]:443/a",
        "https://models.example%40evil.example/a",
        "https://models.example\\evil.example/a",
        "https://models.example;evil.example/a",
        "https:/models.example/a",
        "https:models.example/a",
        "https:\\\\models.example/a",
        "https//models.example/a",
        " https://models.example/a",
        "https://models.example/a\t",
        "https://models.example\0/a",
        "https://mo\u{AD}dels.example/a",
        "https://models\u{3002}example/a",
    ] {
        assert_eq!(parts(url), Err(NetError::NotHttps), "{url:?}");
    }
}

#[test]
fn a_name_after_a_query_or_fragment_is_only_path() {
    let at = |u: &str| Target::parse(u).map(|t| (t.host().to_owned(), t.path().to_owned()));
    let want = |p: &str| Ok(("models.example".to_owned(), p.to_owned()));
    assert_eq!(at("https://models.example#@evil.example/a"), want("/"));
    assert_eq!(
        at("https://models.example?@evil.example/a"),
        want("/?@evil.example/a")
    );
    assert_eq!(at("hTtPs://models.example/a@b"), want("/a@b"));
    assert_eq!(at("https://models.example/a/./b/../c"), want("/a/c"));
}

#[test]
fn dot_folders_and_a_lone_fragment_in_a_location_are_resolved() {
    let base = "https://models.example/a/b/m.bin?old=1";
    let on = |p: &str| Ok(format!("https://models.example{p}"));
    for (to, want) in [
        ("../c.bin", "/a/c.bin"),
        ("./c.bin", "/a/b/c.bin"),
        ("../../../../c.bin", "/c.bin"),
        ("/x/./y/../z", "/x/z"),
        (".", "/a/b/"),
        ("..", "/a/"),
        ("c/..", "/a/b/"),
        ("../c?x=../y", "/a/c?x=../y"),
        ("...", "/a/b/..."),
        ("#top", "/a/b/m.bin?old=1"),
    ] {
        assert_eq!(joined(base, to), on(want), "{to}");
    }
}

#[test]
fn a_location_that_joins_past_the_cap_is_refused() {
    let base = "https://models.example/d/m.bin";
    let room = MAX_URL_BYTES - "https://models.example/d/".len();
    let fits = "a".repeat(room);
    assert_eq!(joined(base, &fits).map(|u| u.len()), Ok(MAX_URL_BYTES));
    assert_eq!(joined(base, &format!("{fits}a")), Err(NetError::NotHttps));
    let half = MAX_URL_BYTES / 2;
    let long = format!("https://models.example/{}/m.bin", "d".repeat(half));
    assert_eq!(
        joined(&long, &"e".repeat(half)),
        Err(NetError::NotHttps),
        "two halves that each fit"
    );
}

#[test]
fn a_location_built_to_fool_a_reader_is_refused() {
    let base = "https://models.example/a";
    for to in [
        "//user@cdn.example/x",
        "//[::1]/x",
        "//cdn.example:0/x",
        "//",
        "///x",
        "https:cdn.example/x",
        "HtTp://cdn.example/x",
        "ftp:x",
        "mailto:a@cdn.example",
        "/x\r\nSet-Cookie: a=b",
        " /x",
        "/é",
    ] {
        assert_eq!(joined(base, to), Err(NetError::NotHttps), "{to:?}");
    }
}

#[test]
fn a_backslash_location_stays_on_the_same_host() {
    let base = Target::parse("https://models.example/a/b").unwrap();
    for to in ["\\\\evil.example/x", "/\\evil.example/x"] {
        let host = base.join(to).map(|t| t.host().to_owned());
        assert_eq!(host, Ok("models.example".to_owned()), "{to}");
    }
}

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
