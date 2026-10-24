//! The fake word packs, held to the port: added bytes come back whole, a missing language is `None`, a failure on cue,
//! and opens counted per language.

use super::*;
use std::thread;

const EN: &[u8] = b"en pack";
const AR: &[u8] = "حزمة".as_bytes();

/// The bytes of language `iso`'s pack, copied out of the view.
fn read(packs: &dyn WordPacks, iso: &str) -> Result<Option<Vec<u8>>, PackError> {
    packs
        .open(iso)
        .map(|view| view.map(|pack| pack.bytes().to_vec()))
}

#[test]
fn added_bytes_come_back_whole_per_language() {
    let packs = FakeWordPacks::new();
    packs.add("en", EN.to_vec());
    packs.add("ar", AR.to_vec());
    assert_eq!(read(&packs, "en"), Ok(Some(EN.to_vec())));
    assert_eq!(read(&packs, "ar"), Ok(Some(AR.to_vec())));
    packs.add("en", AR.to_vec());
    assert_eq!(
        read(&packs, "en"),
        Ok(Some(AR.to_vec())),
        "a later add replaces"
    );
}

#[test]
fn a_language_without_a_pack_opens_none() {
    let packs = FakeWordPacks::new();
    packs.add("en", EN.to_vec());
    assert_eq!(read(&packs, "fr"), Ok(None));
}

#[test]
fn a_code_no_platform_could_name_has_no_pack() {
    let packs = FakeWordPacks::new();
    for iso in ["EN", "../en", "con"] {
        packs.add(iso, EN.to_vec());
        assert_eq!(read(&packs, iso), Ok(None), "{iso}");
        assert_eq!(packs.opens(iso), 1, "{iso}");
    }
}

#[test]
fn a_scripted_failure_refuses_the_next_open_only() {
    let packs = FakeWordPacks::new();
    packs.add("en", EN.to_vec());
    packs.fail_open();
    assert_eq!(read(&packs, "en"), Err(PackError::Unreadable));
    assert_eq!(read(&packs, "en"), Ok(Some(EN.to_vec())));
}

#[test]
fn opens_are_counted_per_language_failed_and_missing_ones_too() {
    let packs = FakeWordPacks::new();
    packs.add("en", EN.to_vec());
    packs.fail_open();
    for iso in ["en", "en", "fr"] {
        let _ = read(&packs, iso);
    }
    assert_eq!(
        [packs.opens("en"), packs.opens("fr"), packs.opens("ar")],
        [2, 1, 0]
    );
}

#[test]
fn clones_share_the_packs_and_serve_as_the_port_from_another_thread() {
    let packs = FakeWordPacks::new();
    let port: Arc<dyn WordPacks> = Arc::new(packs.clone());
    packs.add("en", EN.to_vec());
    let opened = thread::spawn(move || read(port.as_ref(), "en"));
    assert_eq!(opened.join().unwrap(), Ok(Some(EN.to_vec())));
    assert_eq!(packs.opens("en"), 1);
}
