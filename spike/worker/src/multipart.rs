//! A `multipart/form-data` body: text fields plus one file, as both engines expect.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::http::{CONTENT_TYPE, CRLF};

/// Start of every boundary we make.
const BOUNDARY_PREFIX: &str = "kx-spike-";
/// Room for the field headers when sizing the body.
const HEAD_ROOM: usize = 256;
/// Makes boundaries made in the same nanosecond differ.
static COUNT: AtomicU32 = AtomicU32::new(0);

/// One form field.
pub struct Part<'a> {
    name: &'a str,
    /// File name and content type, for a file.
    file: Option<(&'a str, &'a str)>,
    data: &'a [u8],
}

impl<'a> Part<'a> {
    /// A text field.
    pub fn text(name: &'a str, value: &'a str) -> Self {
        Part {
            name,
            file: None,
            data: value.as_bytes(),
        }
    }

    /// A file field.
    pub fn file(name: &'a str, filename: &'a str, content_type: &'a str, data: &'a [u8]) -> Self {
        Part {
            name,
            file: Some((filename, content_type)),
            data,
        }
    }
}

/// A boundary unlikely to appear in any clip.
pub fn boundary() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let n = COUNT.fetch_add(1, Ordering::Relaxed);
    format!("{BOUNDARY_PREFIX}{nanos:x}-{n:x}")
}

/// The `Content-Type` header value for `boundary`.
pub fn content_type(boundary: &str) -> String {
    format!("multipart/form-data; boundary={boundary}")
}

/// The whole form body.
pub fn body(boundary: &str, parts: &[Part]) -> Vec<u8> {
    let size = parts.iter().map(|p| p.data.len() + HEAD_ROOM).sum();
    let mut b = Vec::with_capacity(size);
    for p in parts {
        let mut head = format!(
            "--{boundary}{CRLF}Content-Disposition: form-data; name=\"{}\"",
            p.name
        );
        if let Some((file, kind)) = p.file {
            head += &format!("; filename=\"{file}\"{CRLF}{CONTENT_TYPE}{kind}");
        }
        head += &format!("{CRLF}{CRLF}");
        b.extend_from_slice(head.as_bytes());
        b.extend_from_slice(p.data);
        b.extend_from_slice(CRLF.as_bytes());
    }
    b.extend_from_slice(format!("--{boundary}--{CRLF}").as_bytes());
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_and_a_file_follow_the_form_layout() {
        let parts = [
            Part::text("language", "fr"),
            Part::file("file", "clip.wav", "audio/wav", b"RIFF"),
        ];
        let body = body("B", &parts);
        let want = "--B\r\nContent-Disposition: form-data; name=\"language\"\r\n\r\nfr\r\n\
--B\r\nContent-Disposition: form-data; name=\"file\"; filename=\"clip.wav\"\r\n\
Content-Type: audio/wav\r\n\r\nRIFF\r\n--B--\r\n";
        assert_eq!(String::from_utf8(body).expect("utf-8"), want);
        assert_eq!(content_type("B"), "multipart/form-data; boundary=B");
    }

    #[test]
    fn each_boundary_is_new() {
        assert_ne!(boundary(), boundary());
    }
}
