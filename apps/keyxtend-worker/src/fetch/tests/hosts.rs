//! Addresses (edge 22): a redirect to an unlisted host or to plain HTTP is refused before it is asked, and so is a
//! table address that names no listed host or is not plain HTTPS. An address that fails hands over to the next one.

use super::*;

/// Runs a download of `FILE` from `urls` over `acts`; its notes, what the server was asked, and the folder.
fn fetch(urls: &[&str], acts: Vec<Act>) -> (Vec<FetchNote>, Vec<Seen>, Dir) {
    let (dir, server) = (Dir::new(), Server::new(acts));
    let (mut job, mut out) = (job(&server), Vec::new());
    take(&mut job, get_of(1, &dir, urls, FILE), &mut out);
    drive(&mut job, &mut out);
    (out, server.seen(), dir)
}

#[test]
fn a_redirect_to_an_unlisted_host_is_refused_before_it_is_asked() {
    let moved = Act::moved("https://elsewhere.example/m.gguf");
    let (out, asked, dir) = fetch(&[URL], vec![moved, whole(FILE)]);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Redirect)]);
    assert_eq!(asked, [seen(URL, 0)]);
    assert_eq!(dir.files(), (None, None));
}

#[test]
fn a_redirect_to_plain_http_is_refused_before_it_is_asked() {
    for to in ["http://cdn.example/m.gguf", "ftp://cdn.example/m.gguf"] {
        let (out, asked, dir) = fetch(&[URL], vec![Act::moved(to), whole(FILE)]);
        assert_eq!(
            out,
            [progress(1, 0), fault(1, FetchFault::NotHttps)],
            "{to}"
        );
        assert_eq!(asked, [seen(URL, 0)]);
        assert_eq!(dir.files(), (None, None));
    }
}

#[test]
fn a_table_address_outside_the_rules_is_never_asked() {
    let cases = [
        ("https://elsewhere.example/m.gguf", FetchFault::Redirect),
        ("https://user@models.example/m.gguf", FetchFault::NotHttps),
        ("https://[::1]/m.gguf", FetchFault::NotHttps),
    ];
    for (url, want) in cases {
        let (out, asked, dir) = fetch(&[url], vec![whole(FILE)]);
        assert_eq!(out, [progress(1, 0), fault(1, want)], "{url}");
        assert!(asked.is_empty(), "{url}");
        assert_eq!(dir.files(), (None, None));
    }
}

#[test]
fn a_redirect_to_a_listed_host_is_followed() {
    let (out, asked, dir) = fetch(&[URL], vec![Act::moved(MIRROR), whole(FILE)]);
    assert_eq!(out.last(), Some(&done(1)));
    assert_eq!(asked, [seen(URL, 0), seen(MIRROR, 0)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn the_next_address_serves_when_one_fails_or_is_refused() {
    let firsts = [
        Act::Fail(NetError::Net),
        Act::status(404),
        Act::moved("https://elsewhere.example/m.gguf"),
        whole(&[FILE, b"x"].concat()),
    ];
    for first in firsts {
        let (out, asked, dir) = fetch(&[URL, MIRROR], vec![first, whole(FILE)]);
        assert_eq!(out.last(), Some(&done(1)));
        assert_eq!(asked, [seen(URL, 0), seen(MIRROR, 0)]);
        assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
    }
}

#[test]
fn a_body_cut_partway_goes_on_from_the_next_address_at_the_parts_length() {
    let mut cut = reads(&FILE[..20]);
    cut.push(Step::Fail(NetError::Stall));
    let acts = vec![
        Act::Answer(Head::whole(FILE.len()), cut),
        Act::part(&FILE[20..], 20, 40),
    ];
    let (out, asked, dir) = fetch(&[URL, MIRROR], acts);
    let want = [progress(1, 0), progress(1, 20), progress(1, 40), done(1)];
    assert_eq!(out, want);
    assert_eq!(asked, [seen(URL, 0), seen(MIRROR, 20)]);
    assert_eq!(dir.files(), (Some(FILE.to_vec()), None));
}

#[test]
fn a_system_without_an_adapter_fails_every_download_as_net() {
    let dir = Dir::new();
    let (mut job, mut out) = (Job::new(Box::new(super::super::Offline)), Vec::new());
    take(&mut job, get_of(1, &dir, &[URL, MIRROR], FILE), &mut out);
    drive(&mut job, &mut out);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Net)]);
    assert_eq!(dir.files(), (None, None));
}

#[test]
fn when_every_address_fails_the_first_fault_since_the_last_answer_is_told() {
    let refused = Act::moved("https://elsewhere.example/m.gguf");
    let (out, asked, dir) = fetch(&[URL, MIRROR], vec![refused, Act::Fail(NetError::Net)]);
    assert_eq!(out, [progress(1, 0), fault(1, FetchFault::Redirect)]);
    assert_eq!(asked.len(), 2);
    assert_eq!(dir.files(), (None, None));
    let mut stalled = reads(&FILE[..10]);
    stalled.push(Step::Fail(NetError::Stall));
    let acts = vec![Act::status(500), Act::Answer(Head::whole(40), stalled)];
    let (out, _, dir) = fetch(&[URL, MIRROR], acts);
    assert_eq!(out.last(), Some(&fault(1, FetchFault::Stall)), "after it");
    assert_eq!(dir.files(), (None, Some(FILE[..10].to_vec())));
}
