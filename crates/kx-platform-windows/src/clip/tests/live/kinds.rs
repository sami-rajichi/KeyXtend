//! Each kind of part written as the product writes it, then heard by a watch that reads the tests' own writes.

use windows::Win32::System::Ole::DROPEFFECT_COPY;

use super::*;

/// The sizes a DIB header may have: `BITMAPINFOHEADER`, V4 and V5.
const DIB_HEADERS: [u32; 3] = [40, 108, 124];

/// A 1×1 32-bit DIB: a `BITMAPINFOHEADER` (size, width, height, planes, bits, no compression, image size, then four
/// zero fields), then one blue-green-red-alpha pixel.
fn dib() -> Vec<u8> {
    let mut b = Vec::new();
    for v in [40u32, 1, 1] {
        b.extend(v.to_le_bytes());
    }
    b.extend(1u16.to_le_bytes());
    b.extend(32u16.to_le_bytes());
    for v in [0u32, 4, 0, 0, 0, 0] {
        b.extend(v.to_le_bytes());
    }
    b.extend([1, 2, 3, 255]);
    b
}

fn image(codec: ImageCodec, bytes: Vec<u8>) -> ClipPart {
    ClipPart::Image {
        codec,
        bytes: Redacted::new(bytes),
    }
}

#[test]
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
fn a_dib_comes_back_as_one_dib_image() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    write(&[image(ImageCodec::Dib, dib())]);
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
fn png_is_the_one_image_kept_beside_a_dib() {
    let _real = hold();
    let watch = Watch::own(ROOMY);
    let png = image(ImageCodec::Png, b"\x89PNG kx-test".to_vec());
    write(&[png.clone(), image(ImageCodec::Dib, dib())]);
    assert!(watch.hears(&[png]));
}

#[test]
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
