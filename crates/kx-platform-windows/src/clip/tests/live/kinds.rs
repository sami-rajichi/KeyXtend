//! Each kind of part written as the product writes it, then heard by a watch that reads the tests' own writes; opt-in
//! like every live test.

use windows::Win32::System::Ole::DROPEFFECT_COPY;

use super::*;

/// Bytes in a `BITMAPINFOHEADER`.
const INFO: u32 = 40;
/// Bytes in a `BITMAPV5HEADER`.
const V5: u32 = 124;
/// The sizes a DIB header may have: `BITMAPINFOHEADER`, V4 and V5.
const DIB_HEADERS: [u32; 3] = [INFO, 108, V5];
/// Bytes in one 32-bit pixel.
const PIXEL: u32 = 4;
/// The side of a picture far bigger than `PICTURE_CAP`.
const BIG_SIDE: u32 = 64;
/// A cap above the test text and far below a `BIG_SIDE` picture.
const PICTURE_CAP: u64 = 1024;

/// A square 32-bit DIB of `side` pixels with a `header`-byte header: size, width, height, planes, bits, no
/// compression and image size, then zeros up to `header`, then blue-green-red-alpha pixels.
fn dib(header: u32, side: u32) -> Vec<u8> {
    let mut b = Vec::new();
    for v in [header, side, side] {
        b.extend(v.to_le_bytes());
    }
    b.extend(1u16.to_le_bytes());
    b.extend(32u16.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend((side * side * PIXEL).to_le_bytes());
    b.resize(usize::try_from(header).unwrap(), 0);
    for _ in 0..side * side {
        b.extend([1, 2, 3, 255]);
    }
    b
}

fn image(codec: ImageCodec, bytes: Vec<u8>) -> ClipPart {
    ClipPart::Image {
        codec,
        bytes: Redacted::new(bytes),
    }
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn html_rtf_and_text_come_back_in_the_order_written() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let parts = [
        ClipPart::Html(Redacted::new(b"<b>kx-test</b>".to_vec())),
        ClipPart::Rtf(Redacted::new(b"{\\rtf1 kx-test}".to_vec())),
        text(TEXT),
    ];
    write(&parts);
    assert!(watch.hears(&parts));
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn files_come_back_as_paths_and_say_copy() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let dir = std::env::temp_dir();
    let paths = vec![
        dir.join("kx-test-clip-a.txt"),
        dir.join("kx-test-clip-é.txt"),
    ];
    let files = ClipPart::Files(Redacted::new(paths));
    write(std::slice::from_ref(&files));
    assert!(watch.hears(&[files]));
    let open = open::open(WAIT).unwrap();
    let effect = open::dword(&open, &table().unwrap().drop_effect);
    assert_eq!(effect, Some(DROPEFFECT_COPY.0));
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn a_dib_comes_back_as_one_dib_image() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    write(&[image(ImageCodec::Dib, dib(INFO, 1))]);
    let note = watch.hear(|n| matches!(n, ClipNote::Copied { .. }));
    let Some(ClipNote::Copied { parts, .. }) = note else {
        panic!("no copy heard");
    };
    let [ClipPart::Image { codec, bytes }] = parts.as_slice() else {
        panic!("not one image part");
    };
    let header = u32::from_le_bytes(bytes.expose()[..4].try_into().unwrap());
    assert_eq!(*codec, ImageCodec::Dib);
    assert!(DIB_HEADERS.contains(&header), "a DIB header");
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn png_is_the_one_image_kept_beside_a_dib() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let png = image(ImageCodec::Png, b"\x89PNG kx-test".to_vec());
    write(&[png.clone(), image(ImageCodec::Dib, dib(INFO, 1))]);
    assert!(watch.hears(&[png]));
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn a_big_picture_beside_short_text_is_never_fetched() {
    let _real = hold();
    let watch = Watch::own(PICTURE_CAP);
    write(&[text(TEXT), image(ImageCodec::Dib, dib(INFO, BIG_SIDE))]);
    assert!(watch.hears(&[text(TEXT)]), "the text alone, not too big");
}

#[test]
#[cfg_attr(not(kx_live_clipboard), ignore = "writes the real clipboard; opt-in")]
fn a_copy_of_formats_nobody_keeps_has_no_parts() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let zero = 0u32.to_ne_bytes();
    let put = |open: &open::Open| {
        open::empty(open)?;
        open::put(open, open::format_id("KeyXtend.Test.Private")?, b"x")?;
        let t = table()?;
        for name in [HISTORY, t.cloud_off.as_str()] {
            open::put(open, open::format_id(name)?, &zero)?;
        }
        Ok::<_, ClipError>(())
    };
    open::open(WAIT).and_then(|o| put(&o)).unwrap();
    assert!(watch.hears(&[]));
}
