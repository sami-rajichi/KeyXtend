//! A fetch ask: its ranges and the frames' caps, its part file, and links that name a plain listed host exactly.

use super::*;

/// The name a fetch ask is refused by after `change`, or `None` when it passes.
fn refused(change: impl FnOnce(&mut FetchAsk)) -> Option<&'static str> {
    let mut ask = fetch();
    change(&mut ask);
    ask.check().err().map(|e| match e {
        ModelError::Range(name) => name,
        other => panic!("{other}"),
    })
}

/// A link to `x.gguf` on `host`.
fn link_on(host: &str) -> Link {
    Link::new(format!(
        "https://{host}/handy-computer/m/resolve/0a1b/x.gguf"
    ))
    .unwrap()
}

#[test]
fn a_fetch_ask_passes_and_names_its_part_file_as_the_job_does() {
    assert_eq!(refused(|_| {}), None);
    let file = format!(r"D:\{SECRET}\x.gguf");
    let part = Some(PathBuf::from(format!("{file}{PART_EXT}")));
    assert_eq!(fetch().part(), part, "the fetch job writes this part");
}

#[cfg(windows)]
#[test]
fn a_file_that_is_no_text_has_no_part_and_is_refused() {
    use std::os::windows::ffi::OsStringExt;
    let lone = [u16::from(b'D'), u16::from(b':'), u16::from(b'\\'), 0xD800];
    let file = Redacted::new(PathBuf::from(std::ffi::OsString::from_wide(&lone)));
    let ask = FetchAsk { file, ..fetch() };
    assert_eq!(ask.part(), None);
    assert_eq!(ask.check(), Err(ModelError::Range("file")));
}

#[test]
fn a_fetch_ask_past_its_ranges_is_refused_by_its_name() {
    let free = (*FetchAsk::MIN_FREE_MB.start(), *FetchAsk::MIN_FREE_MB.end());
    let stall = (*FetchAsk::STALL_S.start(), *FetchAsk::STALL_S.end());
    assert_eq!(refused(|a| a.min_free_mb = free.0 - 1), Some("min_free_mb"));
    assert_eq!(refused(|a| a.min_free_mb = free.1 + 1), Some("min_free_mb"));
    assert_eq!(refused(|a| a.stall_s = stall.0 - 1), Some("stall_s"));
    assert_eq!(refused(|a| a.stall_s = stall.1 + 1), Some("stall_s"));
    let job = refused(|a| a.job.job_mb = 0);
    assert_eq!(job, Some("job_mb"), "the fetch job's own cap");
    assert_eq!(FetchAsk::MIN_FREE_MB, 256..=65_536, "the [voice] bound");
    assert_eq!(FetchAsk::STALL_S, 5..=300, "the plan's [voice] bound");
}

#[test]
fn a_fetch_ask_without_an_address_a_host_a_size_or_a_good_file_is_refused() {
    assert_eq!(refused(|a| a.links.clear()), Some("links"));
    assert_eq!(refused(|a| a.hosts.clear()), Some("hosts"));
    assert_eq!(refused(|a| a.hosts.push("bad host".into())), Some("hosts"));
    assert_eq!(refused(|a| a.hosts.push(String::new())), Some("hosts"));
    for wide in [".co", ".", ".hf..co"] {
        assert_eq!(
            refused(|a| a.hosts.push(wide.into())),
            Some("hosts"),
            "{wide}"
        );
    }
    assert_eq!(refused(|a| a.bytes = 0), Some("bytes"));
    let relative = Redacted::new(PathBuf::from("x.gguf"));
    assert_eq!(refused(|a| a.file = relative), Some("file"));
}

#[test]
fn a_link_under_a_dot_entry_alone_is_refused_and_a_redirect_there_is_reached() {
    for host in ["cdn.hf.co", "us.aws.cdn.hf.co"] {
        assert_eq!(refused(|a| a.links = vec![link_on(host)]), Some("links"));
        assert!(fetch().reaches(host), "a redirect may go there: {host}");
    }
    let named = |a: &mut FetchAsk| {
        a.hosts.push("cdn.hf.co".into());
        a.links.push(link_on("cdn.hf.co"));
    };
    assert_eq!(refused(named), None, "listed by name too");
}

#[test]
fn a_link_names_its_host_exactly_with_no_port_user_or_other_host() {
    let odd = [
        "https://huggingface.co:443/m/x.gguf",
        "https://user@huggingface.co/m/x.gguf",
        "https://huggingface.co.evil.example/m/x.gguf",
        "https://hf.co/m/x.gguf",
        "https://huggingface.co?x=/m/x.gguf",
        "https://huggingface.co",
    ];
    for text in odd {
        let link = Link::new(text.into()).unwrap();
        assert_eq!(refused(|a| a.links.push(link)), Some("links"), "{text}");
    }
}

#[test]
fn a_link_is_named_only_by_a_plain_entry_of_its_own_host() {
    let hosts = ["huggingface.co".to_owned(), ".hf.co".to_owned()];
    assert!(link_named(&hosts, "https://huggingface.co/m/x.gguf"));
    for link in [
        "https://cdn.hf.co/m/x.gguf",
        "https://hf.co/m/x.gguf",
        "http://huggingface.co/m/x.gguf",
        "huggingface.co/m/x.gguf",
        "",
    ] {
        assert!(!link_named(&hosts, link), "{link}");
    }
}

#[test]
fn a_fetch_ask_past_the_frames_caps_is_refused() {
    let big = kx_ipc::MAX_MODEL_BYTES;
    assert_eq!(refused(|a| a.bytes = big), None);
    assert_eq!(refused(|a| a.bytes = big + 1), Some("bytes"));
    let more = |a: &mut FetchAsk| a.links = vec![a.links[0].clone(); kx_ipc::MAX_URLS + 1];
    assert_eq!(refused(more), Some("links"));
    let tail = "a".repeat(kx_ipc::MAX_URL_BYTES);
    let long = Link::new(format!("https://huggingface.co/{tail}")).unwrap();
    assert_eq!(refused(|a| a.links.push(long)), Some("links"));
    let many = vec!["hf.co".to_owned(); kx_ipc::MAX_HOSTS + 1];
    assert_eq!(refused(|a| a.hosts = many), Some("hosts"));
    let name = "a".repeat(kx_ipc::MAX_HOST_BYTES + 1);
    assert_eq!(refused(|a| a.hosts.push(name)), Some("hosts"));
    assert_eq!(
        refused(|a| a.hosts.push("HF.co".into())),
        Some("hosts"),
        "lower case only"
    );
}

#[test]
fn a_fetch_file_whose_part_passes_the_frames_cap_is_refused() {
    let top = kx_ipc::MAX_PATH_BYTES - PART_EXT.len();
    let at = |len: usize| {
        let name = "a".repeat(len - DRIVE_ROOT.len());
        Redacted::new(PathBuf::from(format!("{DRIVE_ROOT}{name}")))
    };
    let (fits, over) = (at(top), at(top + 1));
    assert_eq!(refused(|a| a.file = fits), None);
    assert_eq!(refused(|a| a.file = over), Some("file"));
}
