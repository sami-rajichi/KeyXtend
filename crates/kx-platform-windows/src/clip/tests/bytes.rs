//! The bytes a write sets for each part, and the `CF_HDROP` file list both ways.

use proptest::prelude::*;

use super::*;
use crate::clip::hdrop;

fn units(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_ne_bytes(*b))
        .collect()
}

#[test]
fn text_goes_as_utf16_with_cr_lf_and_a_closing_nul() {
    let part = ClipPart::Text(Redacted::new("a\nلا".to_owned()));
    let w = F.encode(&part);
    let want: Vec<u16> = "a\r\nلا\0".encode_utf16().collect();
    assert_eq!(units(&w[0].1), want);
}

#[test]
fn html_and_rtf_keep_their_bytes_and_close_with_a_nul() {
    let html = ClipPart::Html(Redacted::new(b"<b>x</b>".to_vec()));
    let rtf = ClipPart::Rtf(Redacted::new(b"{\\rtf1 x}".to_vec()));
    let (html, rtf) = (F.encode(&html), F.encode(&rtf));
    assert_eq!((html[0].0, &*html[0].1), (F.html, &b"<b>x</b>\0"[..]));
    assert_eq!((rtf[0].0, &*rtf[0].1), (F.rtf, &b"{\\rtf1 x}\0"[..]));
}

#[test]
fn images_keep_their_bytes_in_their_own_format() {
    let image = |codec| ClipPart::Image {
        codec,
        bytes: Redacted::new(vec![1, 2, 3]),
    };
    let (png, dib) = (image(ImageCodec::Png), image(ImageCodec::Dib));
    let (png, dib) = (F.encode(&png), F.encode(&dib));
    assert_eq!((png[0].0, &*png[0].1), (F.png, &[1u8, 2, 3][..]));
    assert_eq!((dib[0].0, &*dib[0].1), (F.dib, &[1u8, 2, 3][..]));
}

#[test]
fn a_dib_with_a_v4_or_v5_header_goes_as_dibv5_and_a_smaller_one_as_dib() {
    let dib = |header: u32| ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(header.to_le_bytes().into_iter().chain([0; 8]).collect()),
    };
    for (header, format) in [(40, F.dib), (108, F.dib_v5), (124, F.dib_v5)] {
        assert_eq!(
            F.encode(&dib(header))[0].0,
            format,
            "a {header}-byte header"
        );
    }
}

/// Windows makes `CF_DIB` and `CF_BITMAP` from a set DIB, never from PNG, so classic apps need the DIB set too.
#[test]
fn an_insert_of_png_and_dib_sets_both_formats() {
    let png = ClipPart::Image {
        codec: ImageCodec::Png,
        bytes: Redacted::new(vec![1, 2, 3]),
    };
    let v5: Vec<u8> = 124u32.to_le_bytes().into_iter().chain([0; 8]).collect();
    let dib = ClipPart::Image {
        codec: ImageCodec::Dib,
        bytes: Redacted::new(v5.clone()),
    };
    let parts = [png, dib];
    let set = F.encode_all(&parts);
    let set: Vec<(u32, &[u8])> = set.iter().map(|(f, b)| (*f, &**b)).collect();
    assert_eq!(set, [(F.png, &[1u8, 2, 3][..]), (F.dib_v5, &v5[..])]);
}

#[test]
fn files_go_as_a_list_that_says_copy() {
    let paths = vec![PathBuf::from(r"C:\a.txt"), PathBuf::from(r"D:\b c\é.png")];
    let part = ClipPart::Files(Redacted::new(paths.clone()));
    let w = F.encode(&part);
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].0, F.files);
    assert_eq!(hdrop::decode(&w[0].1), Some(paths));
    assert_eq!((w[1].0, &*w[1].1), (F.effect, &F.copy.to_ne_bytes()[..]));
}

#[test]
fn a_file_list_round_trips_with_utf16_paths_after_the_header() {
    let paths = vec![PathBuf::from(r"C:\x"), PathBuf::from(r"C:\مجلد\y")];
    let bytes = hdrop::encode(&paths);
    let at = u32::from_ne_bytes(bytes[..4].try_into().unwrap());
    assert_eq!(usize::try_from(at).unwrap(), hdrop::HEADER);
    let want: Vec<u16> = "C:\\x\0C:\\مجلد\\y\0\0".encode_utf16().collect();
    assert_eq!(units(&bytes[hdrop::HEADER..]), want);
    assert_eq!(hdrop::decode(&bytes), Some(paths));
}

#[test]
fn an_empty_list_has_no_paths() {
    assert_eq!(hdrop::decode(&hdrop::encode(&[])), Some(Vec::new()));
}

#[test]
fn a_list_that_is_not_utf16_or_has_a_bad_header_is_refused() {
    let mut narrow = hdrop::encode(&[PathBuf::from("a")]);
    narrow[16..20].copy_from_slice(&0i32.to_ne_bytes());
    assert_eq!(hdrop::decode(&narrow), None, "a 16-bit era list");
    let mut far = hdrop::encode(&[PathBuf::from("a")]);
    far[..4].copy_from_slice(&u32::MAX.to_ne_bytes());
    assert_eq!(hdrop::decode(&far), None, "paths past the end");
    let mut inside = hdrop::encode(&[PathBuf::from("a")]);
    inside[..4].copy_from_slice(&4u32.to_ne_bytes());
    assert_eq!(hdrop::decode(&inside), None, "paths inside the header");
    assert_eq!(hdrop::decode(&[0; 7]), None, "a cut header");
}

#[test]
fn a_list_without_its_last_nul_keeps_only_the_paths_that_ended() {
    let mut bytes = hdrop::encode(&[PathBuf::from("a"), PathBuf::from("bc")]);
    bytes.truncate(bytes.len() - 4);
    assert_eq!(hdrop::decode(&bytes), Some(vec![PathBuf::from("a")]));
}

#[cfg(windows)]
#[test]
fn a_dibv5_header_starts_at_the_size_of_windows_v4_header() {
    use windows::Win32::Graphics::Gdi::BITMAPV4HEADER;
    let v4 = usize::try_from(crate::clip::parts::V4_HEADER).unwrap();
    assert_eq!(v4, size_of::<BITMAPV4HEADER>());
}

#[cfg(windows)]
#[test]
fn the_header_is_as_long_as_windows_dropfiles() {
    use windows::Win32::UI::Shell::DROPFILES;
    assert_eq!(hdrop::HEADER, size_of::<DROPFILES>());
}

/// The longest random file list tried.
const MAX_LIST: usize = 256;

proptest! {
    #[test]
    fn random_bytes_never_panic_the_file_list_reader(
        bytes in prop::collection::vec(any::<u8>(), 0..MAX_LIST),
    ) {
        let _ = hdrop::decode(&bytes);
    }

    #[test]
    fn any_paths_round_trip(names in prop::collection::vec("[^\u{0}]{1,12}", 0..5)) {
        let paths: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
        prop_assert_eq!(hdrop::decode(&hdrop::encode(&paths)), Some(paths));
    }
}
