//! A new video's name: a count, from a start the wall clock gives, past every video held, so it never replaces one.

use kx_module_api::ports::{FileKind, FolderPath, SnipFile};

/// The first video `<n>.mp4` in `unsaved` from `start` up whose name none of `held` has in any case, as Windows reads
/// names, with its count; `None` past the last count.
pub(crate) fn fresh(
    unsaved: &FolderPath,
    start: u64,
    held: &[SnipFile],
) -> Option<(u64, SnipFile)> {
    let ext = FileKind::Mp4.ext();
    let tries = u64::try_from(held.len()).ok()?.checked_add(1)?;
    (0..tries).find_map(|step| {
        let count = start.checked_add(step)?;
        let name = format!("{count}.{ext}");
        let taken = held.iter().any(|h| {
            let theirs = h.expose().file_name().and_then(|n| n.to_str());
            theirs.is_some_and(|n| n.eq_ignore_ascii_case(&name))
        });
        let path = unsaved.expose().join(&name);
        let file = (!taken)
            .then(|| SnipFile::new(path, FileKind::Mp4))
            .flatten();
        file.map(|f| (count, f))
    })
}
