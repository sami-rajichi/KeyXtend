//! The model files port: its gate, the path rule, fingerprints, a fetch ask's ranges and part file, and `Debug` that
//! hides every path and name.

use super::*;
use crate::REDACTED_MARKER;
use crate::ports::{JobLimits, Link};
use std::collections::HashSet;

/// A folder name no output may show.
const SECRET: &str = "Taxes-4417";
/// A drive's root, which every model path starts with.
const DRIVE_ROOT: &str = r"D:\";
/// A file's SHA-256 in lower-case hex.
const HEX: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

fn fetch() -> FetchAsk {
    let link =
        Link::new("https://huggingface.co/handy-computer/m/resolve/0a1b/x.gguf".into()).unwrap();
    FetchAsk {
        job: job(),
        links: vec![link],
        hosts: vec!["huggingface.co".into(), ".hf.co".into()],
        file: Redacted::new(PathBuf::from(format!(r"D:\{SECRET}\x.gguf"))),
        bytes: 776_000_000,
        print: Fingerprint::from_hex(HEX).unwrap(),
        min_free_mb: 2_048,
        stall_s: 30,
    }
}

/// The name a fetch ask is refused by after `change`, or `None` when it passes.
fn refused(change: impl FnOnce(&mut FetchAsk)) -> Option<&'static str> {
    let mut ask = fetch();
    change(&mut ask);
    ask.check().err().map(|e| match e {
        ModelError::Range(name) => name,
        other => panic!("{other}"),
    })
}

#[test]
fn the_service_is_named_and_gated_by_the_network() {
    assert_eq!(ModelFilesKey::ID.as_str(), "model-files");
    assert_eq!(
        ModelFilesKey::CAPABILITY,
        Some(Capability::Network),
        "ADR-0042 decision 8"
    );
}

#[test]
fn only_an_absolute_drive_path_is_ok() {
    for good in [
        r"D:\KeyXtend\models",
        "C:/models/x.gguf",
        r"E:\a b\c..d.gguf",
        r"D:\",
    ] {
        assert!(path_ok(Path::new(good)), "{good}");
    }
    for bad in [
        "",
        "models",
        r"\models",
        r"\\server\share\m",
        r"\\?\D:\m",
        r"\\.\pipe\x",
        "D:models",
    ] {
        assert!(!path_ok(Path::new(bad)), "{bad}");
    }
}

#[test]
fn a_path_that_climbs_or_names_a_stream_is_refused() {
    let bad = [
        r"D:\a\..\b",
        "D:/a/../b",
        r"D:\a\.\b",
        r"D:\a\.. \b",
        r"D:\a\...",
        r"D:\m\x.gguf:zone",
    ];
    for path in bad {
        assert!(!path_ok(Path::new(path)), "edge 38: {path}");
    }
}

#[test]
fn a_path_with_a_control_character_or_a_device_name_is_refused() {
    let bad = [
        "D:\\m\\x\0.gguf",
        "D:\\m\\x\n.gguf",
        "D:\\m\u{7f}\\x.gguf",
        r"D:\m\NUL.gguf",
        r"D:\com1.gguf",
        r"D:\Lpt9\x.gguf",
        r"D:\m\aux",
        r"D:\m\con.tar.gz",
        r"D:\m\prn .gguf",
        r"D:\m\COM¹.gguf",
        r"D:\Lpt³\x.gguf",
        r"D:\m\CONIN$",
        r"D:\m\conout$.gguf",
    ];
    for path in bad {
        assert!(!path_ok(Path::new(path)), "{path:?}");
    }
    let near = [
        r"D:\m\com10.gguf",
        r"D:\m\nullable.gguf",
        r"D:\console\x.gguf",
    ];
    for path in near {
        assert!(path_ok(Path::new(path)), "{path}");
    }
}

#[test]
fn a_fingerprint_reads_and_writes_lower_case_hex() {
    let print = Fingerprint::from_hex(HEX).unwrap();
    assert_eq!(print.hex(), HEX);
    let upper = Fingerprint::from_hex(&HEX.to_uppercase());
    assert_eq!(upper, Some(print), "either case reads");
    assert_eq!(format!("{print:?}"), format!("Fingerprint({HEX})"));
    let short = &HEX[1..];
    let long = format!("{HEX}0");
    let (odd, signed) = (format!("g{short}"), format!("+{short}"));
    for bad in [short, &long, &odd, &signed, "", &"é".repeat(32)] {
        assert_eq!(Fingerprint::from_hex(bad), None, "{bad}");
    }
    let ab = Fingerprint::new([0xab; Fingerprint::BYTES]);
    assert_eq!(ab.hex(), "ab".repeat(32));
    assert_eq!(ab.bytes(), [0xab; Fingerprint::BYTES], "the bytes it wraps");
}

#[test]
fn a_path_past_the_frames_cap_is_refused() {
    let at = |len: usize| format!("{DRIVE_ROOT}{}", "a".repeat(len - DRIVE_ROOT.len()));
    assert!(path_ok(Path::new(&at(kx_ipc::MAX_PATH_BYTES))));
    assert!(!path_ok(Path::new(&at(kx_ipc::MAX_PATH_BYTES + 1))));
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

#[test]
fn each_frame_fault_maps_to_its_error() {
    use kx_ipc::FetchFault as F;
    let map = [
        (F::BadSize, ModelError::Size),
        (F::BadHash, ModelError::Print),
        (F::Redirect, ModelError::Host),
        (F::NotHttps, ModelError::Host),
        (F::Stall, ModelError::Stalled),
        (F::DiskFull, ModelError::Full),
        (F::Disk, ModelError::Io),
        (F::Net, ModelError::Net),
        (F::Server, ModelError::Server),
        (F::Cancelled, ModelError::Cancelled),
        (F::Internal, ModelError::Internal),
    ];
    for (fault, error) in map {
        assert_eq!(ModelError::from(fault), error, "{fault:?}");
    }
}

#[test]
fn no_drive_scan_or_ask_prints_a_path_or_a_name() {
    let name = || Redacted::new(SECRET.to_owned());
    let drive = Drive {
        root: Redacted::new(PathBuf::from(format!(r"D:\{SECRET}"))),
        name: name(),
        free: 7,
    };
    let file = ModelFile {
        name: name(),
        bytes: 9,
    };
    let scan = FolderScan {
        folders: vec![name()],
        files: vec![file],
    };
    let shown = format!("{drive:?}{scan:?}{:?}", fetch());
    assert!(!shown.contains(SECRET), "{shown}");
    assert!(
        shown.contains(REDACTED_MARKER) && shown.contains("free: 7"),
        "{shown}"
    );
}

#[test]
fn each_error_says_something_else() {
    let errors = [
        ModelError::Range("stall_s"),
        ModelError::BadPath,
        ModelError::Missing,
        ModelError::ReadOnly,
        ModelError::Full,
        ModelError::Io,
        ModelError::Net,
        ModelError::Stalled,
        ModelError::Host,
        ModelError::Size,
        ModelError::Print,
        ModelError::Server,
        ModelError::Cancelled,
        ModelError::Internal,
        ModelError::Gone,
    ];
    let texts: HashSet<_> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), errors.len());
}
