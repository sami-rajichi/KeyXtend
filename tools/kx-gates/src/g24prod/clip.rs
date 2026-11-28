//! What the copy put on the clipboard after a Copy: its picture's DIB, or the video file its file drop names. Only
//! sizes and lengths come out; the gate's own clipboard keeping puts the owner's back at the end.
#![cfg(windows)]

use std::path::{Path, PathBuf};

use kx_module_api::ports::FileKind;
use kx_platform_windows::clip::open;
use windows::Win32::System::Ole::{CF_DIB, CF_HDROP};

use super::hdrop;
use super::pic::Pic;
use crate::apps::Ctx;
use crate::clipkeep;
use crate::win;

/// The clipboard's change count now, taken before a Copy.
pub fn mark() -> u32 {
    open::sequence()
}

/// What `read` makes of `format` once the clipboard changed from `since` and holds it, within `wait_ms`.
fn after<T>(
    ctx: &Ctx,
    (since, wait_ms): (u32, u64),
    format: u16,
    read: impl Fn(&[u8]) -> Result<T, String>,
) -> Result<T, String> {
    let wait = clipkeep::wait(&ctx.cfg.timing);
    let got = win::poll_until(wait_ms, ctx.cfg.timing.poll_ms, || {
        if open::sequence() == since {
            return None;
        }
        let o = open::open(wait).ok()?;
        open::with_data(&o, u32::from(format), &read)
    });
    got.unwrap_or_else(|| Err(format!("nothing came to the clipboard within {wait_ms} ms")))
}

/// The picture a Copy put on the clipboard since change count `since`, and its DIB's bytes.
pub fn picture(ctx: &Ctx, since: u32) -> Result<(Pic, usize), String> {
    let read = |b: &[u8]| Pic::from_dib(b).map(|p| (p, b.len()));
    after(ctx, (since, ctx.cfg.g24.wait_ms), CF_DIB.0, read)
}

/// The one video file a Copy put on the clipboard since change count `since`, within `wait_ms`; refused unless it lies
/// in the copy's data folder `data` with the video extension.
pub fn video(ctx: &Ctx, (since, wait_ms): (u32, u64), data: &Path) -> Result<PathBuf, String> {
    let files = after(ctx, (since, wait_ms), CF_HDROP.0, hdrop::dropped)?;
    let [file] = files.as_slice() else {
        return Err(format!(
            "the Copy named {} files, not one video",
            files.len()
        ));
    };
    let ext = FileKind::Mp4.ext();
    let real = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    if !hdrop::inside(&real(file), &real(data)) || file.extension().is_none_or(|e| e != ext) {
        return Err("the copied file is no video in the copy's data folder".into());
    }
    Ok(file.clone())
}
