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
