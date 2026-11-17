//! The model files port: its gate, fingerprints, errors and `Debug` that hides every path and name; the path rule and
//! a fetch ask's checks have files of their own.

mod fetch;
mod paths;

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
