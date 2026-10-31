//! The content hash: a stable 64-bit FNV-1a over a copy's parts alone, so a copy of the same content finds its entry,
//! whatever the kind rules or the media table say. Lists keep it, so it never changes; equal hashes are confirmed by
//! comparing the content.

use kx_module_api::ports::{ClipPart, ImageCodec};

/// FNV-1a's 64-bit offset basis (Fowler, Noll and Vo; IETF draft-eastlake-fnv).
const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a's 64-bit prime.
const PRIME: u64 = 0x0000_0100_0000_01b3;

/// The hash of a copy of `parts`: each part's tag and its length-prefixed bytes.
#[must_use]
pub fn of(parts: &[ClipPart]) -> u64 {
    let mut h = Fnv(OFFSET);
    for part in parts {
        h.part(part);
    }
    h.0
}

/// FNV-1a over `bytes`, which the store also uses to tell when a list changed.
pub(crate) fn fnv(bytes: &[u8]) -> u64 {
    let mut h = Fnv(OFFSET);
    h.eat(bytes);
    h.0
}

/// A running FNV-1a hash.
struct Fnv(u64);

impl Fnv {
    /// Hashes in `bytes`.
    fn eat(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(PRIME);
        }
    }

    /// Hashes in `n` as 8 little-endian bytes.
    fn count(&mut self, n: usize) {
        self.eat(&u64::try_from(n).unwrap_or(u64::MAX).to_le_bytes());
    }

    /// Hashes in `bytes` after their length, so where one part ends never shifts.
    fn run(&mut self, bytes: &[u8]) {
        self.count(bytes.len());
        self.eat(bytes);
    }

    /// Hashes in one part: its tag, then its bytes; an image, its codec first; a file list, its count and each path.
    fn part(&mut self, part: &ClipPart) {
        self.eat(&[part_tag(part)]);
        match part {
            ClipPart::Text(text) => self.run(text.expose().as_bytes()),
            ClipPart::Html(bytes) | ClipPart::Rtf(bytes) => self.run(bytes.expose()),
            ClipPart::Image { codec, bytes } => {
                self.eat(&[codec_tag(*codec)]);
                self.run(bytes.expose());
            }
            ClipPart::Files(paths) => {
                self.count(paths.expose().len());
                for path in paths.expose() {
                    self.run(path.as_os_str().as_encoded_bytes());
                }
            }
        }
    }
}

/// The part's tag; saved hashes hold it, so it never changes.
fn part_tag(part: &ClipPart) -> u8 {
    match part {
        ClipPart::Text(_) => 1,
        ClipPart::Html(_) => 2,
        ClipPart::Rtf(_) => 3,
        ClipPart::Image { .. } => 4,
        ClipPart::Files(_) => 5,
    }
}

/// The codec's tag; saved hashes hold it, so it never changes.
fn codec_tag(codec: ImageCodec) -> u8 {
    match codec {
        ImageCodec::Png => 1,
        ImageCodec::Dib => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kx_module_api::Redacted;
    use kx_module_api::ports::ImageCodec;

    use super::*;
    use crate::fixture::{files, text};

    /// An HTML part.
    fn html(s: &str) -> ClipPart {
        ClipPart::Html(Redacted::new(s.as_bytes().to_vec()))
    }

    /// An image part of `codec`.
    fn image(codec: ImageCodec, bytes: &[u8]) -> ClipPart {
        let bytes = Redacted::new(bytes.to_vec());
        ClipPart::Image { codec, bytes }
    }

    #[test]
    fn fnv_matches_the_reference_vectors() {
        assert_eq!(fnv(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv(b"abc"), 0xe71f_a219_0541_574b);
    }

    #[test]
    fn the_same_parts_give_the_same_pinned_hash() {
        let parts = [text("École"), html("<b>É</b>")];
        assert_eq!(of(&parts), of(&parts.clone()));
        assert_eq!(of(&parts), PINNED, "saved hashes must keep matching");
    }

    #[test]
    fn part_bounds_and_part_kinds_count() {
        let split = |a: ClipPart, b: ClipPart| of(&[a, b]);
        assert_ne!(split(text("ab"), html("c")), split(text("a"), html("bc")));
        let rtf = ClipPart::Rtf(Redacted::new(b"x".to_vec()));
        assert_ne!(of(&[html("x")]), of(&[rtf]));
        assert_ne!(of(&[text("x")]), of(&[html("x")]));
    }

    #[test]
    fn the_image_codec_and_the_path_bounds_count() {
        let png = of(&[image(ImageCodec::Png, b"px")]);
        assert_ne!(png, of(&[image(ImageCodec::Dib, b"px")]));
        assert_ne!(of(&[files(&["ab", "c"])]), of(&[files(&["a", "bc"])]));
        let none = ClipPart::Files(Redacted::new(Vec::<PathBuf>::new()));
        assert_ne!(of(&[none]), of(&[]));
    }

    /// The hash of `the_same_parts_give_the_same_pinned_hash`'s parts, as the build gives it and a separate FNV-1a
    /// script gave it.
    const PINNED: u64 = 2_096_625_033_897_716_263;
}
