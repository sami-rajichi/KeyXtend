//! Property tests of the backup blob's format: random bytes and a good blob cut or changed anywhere never panic, and
//! any blob comes back as it was saved.

use std::path::PathBuf;

use super::codec::{MARK, decode, encode};
use super::{Checks, Kept, Site};
use crate::entry::Stamp;
use kx_module_api::Redacted;
use kx_module_api::ports::{Padlock, Place, PlaceKind};
use proptest::prelude::*;

/// Short texts of letters, Arabic, spaces and path marks.
fn any_text() -> impl Strategy<Value = String> {
    let chars = vec!['a', 'Z', 'é', 'ع', ' ', ':', '\\', '/', '7'];
    prop::collection::vec(prop::sample::select(chars), 0..8).prop_map(|v| v.into_iter().collect())
}

/// A place the box may live in.
fn any_site() -> impl Strategy<Value = Site> {
    let kind = prop::sample::select(PlaceKind::ALL.to_vec());
    let times = (
        any::<Option<i64>>(),
        any::<Option<u64>>(),
        any::<u64>(),
        any::<bool>(),
    );
    (kind, any_text(), any_text(), times).prop_map(
        |(kind, name, path, (at, mark, len, waiting))| Site {
            place: Place {
                kind,
                name: Redacted::new(name),
                path: Redacted::new(PathBuf::from(path)),
            },
            written: at.map(Stamp::from_unix_ms),
            mark,
            len,
            waiting,
        },
    )
}

/// A padlock, or none.
fn any_padlock() -> impl Strategy<Value = Option<Padlock>> {
    let bytes = prop::collection::vec(any::<u8>(), 0..16);
    prop::option::of((any_text(), bytes)).prop_map(|p| {
        p.map(|(recipient, locked)| Padlock {
            recipient: Redacted::new(recipient),
            locked: Redacted::new(locked),
        })
    })
}

/// Any blob.
fn any_kept() -> impl Strategy<Value = Kept> {
    let sites = prop::collection::vec(any_site(), 0..4);
    let left = prop::collection::vec(any::<u64>(), 0..4);
    let flags = (any::<bool>(), any::<bool>(), any::<bool>());
    let checks = (any::<Option<i64>>(), any::<u8>(), any::<u8>());
    (sites, any_padlock(), left, flags, checks).prop_map(
        |(sites, padlock, left, (offered, behind, held), (due, step, missed))| Kept {
            sites,
            padlock,
            left,
            offered,
            behind,
            held,
            checks: Checks {
                due: due.map(Stamp::from_unix_ms),
                step,
                missed,
            },
        },
    )
}

proptest! {
    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..256)) {
        let _ = decode(&bytes);
        let headed = [MARK.as_slice(), &[1], &bytes].concat();
        let _ = decode(&headed);
    }

    #[test]
    fn any_blob_comes_back_as_it_was_saved(kept in any_kept()) {
        prop_assert_eq!(decode(&encode(&kept)), Ok(kept));
    }

    #[test]
    fn a_cut_or_changed_blob_never_panics(kept in any_kept(), at: prop::sample::Index, byte: u8) {
        let bytes = encode(&kept).to_vec();
        let at = at.index(bytes.len());
        prop_assert!(decode(&bytes[..at]).is_err());
        let mut changed = bytes.clone();
        changed[at] = byte;
        let _ = decode(&changed);
    }
}
