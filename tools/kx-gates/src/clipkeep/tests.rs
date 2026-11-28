//! Tests of `clipkeep`.

use super::*;
use windows::Win32::System::Ole::{
    CF_BITMAP, CF_ENHMETAFILE, CF_METAFILEPICT, CF_OWNERDISPLAY, CF_PALETTE,
};

/// A registered format's id, as Windows gives one to `CanIncludeInClipboardHistory`.
const NUMBER_MARKER: u32 = FIRST_NAMED + 7;

/// A clipboard that holds no marker format by name.
fn no_format(_: &str) -> bool {
    false
}

/// A clipboard that holds every marker format by name.
fn every_format(_: &str) -> bool {
    true
}

/// A clipboard whose number markers hold no number.
fn no_number(_: &str) -> Option<u32> {
    None
}

/// A clipboard holding `held`, read as `save` reads it: each format's bytes while they fit in what is left.
fn reader(held: &[(u32, Vec<u8>)]) -> impl Fn(u32, usize) -> Option<Option<Vec<u8>>> + '_ {
    move |f, left| {
        let (_, b) = held.iter().find(|h| h.0 == f)?;
        Some((b.len() <= left).then(|| b.clone()))
    }
}

#[test]
fn memory_formats_can_be_saved_raw() {
    for f in [CF_UNICODETEXT, CF_TEXT, CF_LOCALE, CF_HDROP] {
        assert!(raw_ok(u32::from(f.0)), "{}", f.0);
    }
    assert!(raw_ok(FIRST_NAMED), "a registered format such as HTML");
}

#[test]
fn picture_and_handle_formats_cannot() {
    let handles = [
        CF_BITMAP,
        CF_METAFILEPICT,
        CF_PALETTE,
        CF_ENHMETAFILE,
        CF_OWNERDISPLAY,
    ];
    for f in handles {
        assert!(!raw_ok(u32::from(f.0)), "{}", f.0);
    }
    assert!(
        !raw_ok(0x0200) && !raw_ok(0x0300),
        "private and GDI object ranges"
    );
}

#[test]
fn a_number_marker_such_as_our_put_backs_is_kept_and_put_back_byte_for_byte() {
    let text = vec![b'k', 0, b'x', 0, 0, 0];
    let held = vec![
        (u32::from(CF_UNICODETEXT.0), text),
        (NUMBER_MARKER, 0u32.to_ne_bytes().to_vec()),
    ];
    let formats = held.iter().map(|h| h.0).collect();
    // Every number marker holds 0, as our own put-back's and a password manager's may.
    let zero = |_: &str| Some(0);
    let saved = keep_from(no_format, zero, formats, 64, reader(&held))
        .expect("every number marker at 0 is kept");
    assert_eq!(saved.0, held, "the marker's 0 too");
    let mut put = Vec::new();
    let back = put_all(&saved, |f, b| {
        put.push((f, b.to_vec()));
        Ok::<(), ()>(())
    });
    assert_eq!(back, Ok(()));
    assert_eq!(put, held, "every format in order, byte for byte");
}

#[test]
fn a_private_copy_a_handle_or_too_many_bytes_is_refused() {
    let held = vec![(u32::from(CF_UNICODETEXT.0), vec![1, 0])];
    let formats = || held.iter().map(|h| h.0).collect();
    let e = keep_from(every_format, no_number, formats(), 64, reader(&held)).err();
    assert!(
        e.is_some_and(|e| e.contains("private")),
        "a presence marker"
    );
    let bitmap = vec![(u32::from(CF_BITMAP.0), vec![1])];
    let e = keep_from(no_format, no_number, vec![bitmap[0].0], 64, reader(&bitmap)).err();
    assert!(e.is_some_and(|e| e.contains("put back")), "a handle");
    let e = keep_from(no_format, no_number, formats(), 1, reader(&held)).err();
    assert!(e.is_some_and(|e| e.contains("more than 1")), "too big");
}

#[test]
fn a_put_back_stops_at_the_first_format_refused() {
    let saved = Saved(vec![(1, vec![1]), (2, vec![2])]);
    let mut tried = 0;
    let back = put_all(&saved, |_, _| {
        tried += 1;
        Err("busy")
    });
    assert_eq!((back, tried), (Err("busy"), 1));
}

#[test]
fn an_untouched_clipboard_is_never_written_back() {
    use std::cell::Cell;
    let puts = Cell::new(0);
    let put = || {
        puts.set(puts.get() + 1);
        Ok(())
    };
    assert_eq!(put_if(7, 7, put), Ok(()));
    assert_eq!(puts.get(), 0, "the same sequence number");
    assert_eq!(put_if(7, 8, put), Ok(()));
    assert_eq!(puts.get(), 1, "something wrote it");
}

#[test]
fn the_clipboard_goes_back_once_after_the_test_even_when_it_panics() {
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let puts = Cell::new(0);
    let put = |(): &()| {
        puts.set(puts.get() + 1);
        Ok(())
    };
    assert_eq!(kept((), put, || Ok(7)), Ok(7));
    assert_eq!(puts.get(), 1, "once after a run");
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        kept((), put, || -> Result<(), String> { panic!("a test broke") })
    }));
    assert!(panicked.is_err());
    assert_eq!(puts.get(), 2, "once after a panic");
    let failed = kept((), |(): &()| Err("busy".into()), || Ok(1));
    assert!(failed.expect_err("not put back").contains("busy"));
    let both = kept(
        (),
        |(): &()| Err("busy".into()),
        || -> Result<(), String> { Err("no strip".into()) },
    );
    assert!(both.expect_err("both").starts_with("no strip"));
}

#[test]
fn a_wait_past_what_the_product_holds_is_capped() {
    assert_eq!(ms(250), 250);
    assert_eq!(ms(u64::MAX), u32::MAX);
}

#[test]
fn a_test_that_reaches_a_clipboard_writer_panics_before_anything_is_written() {
    assert!(refused(live), "the guard");
    assert!(
        refused(|| drop(restore(&Saved(Vec::new()), NOW))),
        "restore"
    );
    assert!(
        refused(|| _ = write_text(Some("x"), &[], NOW)),
        "write_text"
    );
    assert!(refused(|| _ = write_text(None, &[], NOW)), "an empty write");
    assert!(!refused(|| ()), "only the guard's refusal counts");
}
