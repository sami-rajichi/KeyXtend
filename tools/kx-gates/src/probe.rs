//! Probe documents for G17, G18 and G6: known text in Notepad, Chrome and Word, test folders in Explorer.
#![cfg(windows)]

use std::path::PathBuf;

use serde_json::{Value, json};
use spike_core::hold::Pt;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

use crate::apps::{self, AppKind, Ctx, Opened};
use crate::simuser::{self, guard, keys_ours};
use crate::win::{self, sleep_ms};
use crate::{clip, keys, launch, out, winclip};
use spike_core::uia::{self, Uia};

/// Word's probe app, which opens a prepared file.
const WORD_FILE: &str = "word_file";
/// Suffix of Word's prepared file.
const RTF_FILE: &str = ".rtf";
/// Suffix of the Explorer test folder.
const DIR_FILE: &str = "-dir";
/// Ends an RTF paragraph.
const RTF_PAR: &str = r"\par ";
/// End of the RTF file.
const RTF_TAIL: &str = "}";
/// Letters that fill the scroll lines.
const FILLER: &str = "abcdefghijklmnopqrstuvwxyz";

/// A probe document: the app showing it, and the file or folder to delete after.
pub struct Doc {
    /// The app and its window.
    pub app: Opened,
    /// The prepared file or folder.
    pub path: PathBuf,
}

/// The app called `name`, if the gate probes it.
pub fn kind_of(name: &str, apps: &[AppKind]) -> Result<AppKind, String> {
    let names: Vec<&str> = apps.iter().map(|a| a.name()).collect();
    apps.iter()
        .copied()
        .find(|a| a.name() == name)
        .ok_or_else(|| format!("this gate probes {}, not {name}", names.join(", ")))
}

/// `text` as RTF in `font` at `half_points`, one paragraph per line; specials escaped, other than ASCII as `\uN?`.
pub fn rtf(text: &str, font: &str, half_points: u32) -> String {
    let mut out = format!(r"{{\rtf1\ansi\deff0{{\fonttbl{{\f0 {font};}}}}\f0\fs{half_points} ");
    for line in text.lines() {
        for c in line.chars() {
            match c {
                '\\' | '{' | '}' => {
                    out.push('\\');
                    out.push(c);
                }
                c if c.is_ascii() => out.push(c),
                c => {
                    for unit in c.encode_utf16(&mut [0; 2]) {
                        out.push_str(&format!("\\u{}?", *unit as i16));
                    }
                }
            }
        }
        out.push_str(RTF_PAR);
    }
    out.push_str(RTF_TAIL);
    out
}

/// `rows` numbered lines of `cols` characters each, for the scroll documents.
pub fn scroll_text(rows: usize, cols: usize) -> String {
    let line = |i: usize| {
        let head = format!("{i:04} ");
        let fill = FILLER.chars().cycle().take(cols.saturating_sub(head.len()));
        head.chars().chain(fill).take(cols).collect::<String>()
    };
    (1..=rows).map(line).collect::<Vec<_>>().join("\n")
}

/// Opens `kind` on `text`: Notepad a text file, Chrome a page, Word an RTF file.
pub fn open_text(ctx: &Ctx, kind: AppKind, text: &str) -> Result<Doc, String> {
    let pr = &ctx.cfg.probes;
    let (doc, app) = match kind {
        AppKind::Notepad => {
            let app = launch::open_notepad(ctx, text)?;
            let path = app.file.clone().unwrap_or_default();
            (Doc { app, path }, kind.name())
        }
        AppKind::Chrome => {
            let css = &pr.page_css;
            return open_page(
                ctx,
                &format!("<pre style=\"{css}\">{}</pre>", launch::html_text(text)),
            );
        }
        AppKind::Word => (open_word(ctx, text)?, WORD_FILE),
        _ => return Err(format!("no text probe for {}", kind.name())),
    };
    sleep_ms(ctx.cfg.app(app)?.ready_ms);
    Ok(doc)
}

/// Chrome on a fresh probe page holding `body`, once it is ready.
pub fn open_page(ctx: &Ctx, body: &str) -> Result<Doc, String> {
    let title = launch::page_title(ctx, &ctx.cfg.probes.page_title);
    let (app, path) = launch::open_page(ctx, &title, body)?;
    sleep_ms(ctx.cfg.app(AppKind::Chrome.name())?.ready_ms);
    Ok(Doc { app, path })
}

/// Word on a fresh RTF file holding `text`, spotted by the file name in its title.
fn open_word(ctx: &Ctx, text: &str) -> Result<Doc, String> {
    let (pr, path) = (&ctx.cfg.probes, ctx.file(RTF_FILE));
    out::write(&path, &rtf(text, &pr.rtf_font, pr.rtf_half_points))?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
    let started = launch::launch(ctx, WORD_FILE, &path.to_string_lossy(), stem.clone())
        .inspect_err(|_| drop(std::fs::remove_file(&path)))?;
    let app = launch::opened(AppKind::Word, started, Some(path.clone()), stem);
    Ok(Doc { app, path })
}

/// Explorer on a fresh folder holding the empty files `files` and the folders `dirs`.
pub fn open_folder(ctx: &Ctx, files: &[String], dirs: &[String]) -> Result<Doc, String> {
    let path = ctx.file(DIR_FILE);
    let fail = |e: std::io::Error| format!("{}: {e}", path.display());
    std::fs::create_dir(&path).map_err(fail)?;
    let filled = files
        .iter()
        .try_for_each(|f| std::fs::File::create(path.join(f)).map(drop))
        .and_then(|()| {
            dirs.iter()
                .try_for_each(|d| std::fs::create_dir(path.join(d)))
        });
    let kind = AppKind::Explorer;
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
    let started = filled
        .map_err(fail)
        .and_then(|()| launch::launch(ctx, kind.name(), &path.to_string_lossy(), name.clone()))
        .inspect_err(|_| drop(std::fs::remove_dir_all(&path)))?;
    sleep_ms(ctx.cfg.app(kind.name())?.ready_ms);
    let app = launch::opened(kind, started, Some(path.clone()), name);
    Ok(Doc { app, path })
}

/// Closes the doc's window and, once nothing shows it, deletes its file or folder; returns what is left.
pub fn close(ctx: &Ctx, doc: Doc) -> Vec<String> {
    let mut left = apps::clean_up(ctx, doc.app);
    if left.is_empty() {
        let gone = if doc.path.is_dir() {
            std::fs::remove_dir_all(&doc.path)
        } else {
            std::fs::remove_file(&doc.path)
        };
        if let Err(e) = gone {
            left.push(format!("{}: {e}", doc.path.display()));
        }
    }
    left
}

/// Brings the doc's window to the front, or fails.
pub fn front(ctx: &Ctx, app: &Opened) -> Result<(), String> {
    if win::front(app.hwnd, &ctx.cfg.timing, &ctx.cfg.keys) {
        guard(app, &[])
    } else {
        Err(format!(
            "could not bring {} to the front",
            win::describe(app.hwnd)
        ))
    }
}

/// Lets go of any button or key a run left down; notes what it did.
pub fn released() -> Vec<String> {
    match simuser::release_all() {
        Ok(true) => vec!["released a button or key left down".to_string()],
        Ok(false) => Vec::new(),
        Err(e) => vec![format!("could not release: {e}")],
    }
}

/// Runs `drive` on `doc`, lets go of anything still held, then closes it; notes what was left.
pub fn run_on<T>(
    ctx: &Ctx,
    doc: Doc,
    drive: impl FnOnce(&Doc) -> Result<T, String>,
) -> Result<(T, Vec<String>), String> {
    let driven = drive(&doc);
    let mut notes = released();
    notes.extend(close(ctx, doc));
    driven
        .map(|got| (got, notes.clone()))
        .map_err(|e| format!("{e}; clean-up: {notes:?}"))
}

/// Copies the selection with Ctrl+C while our window is in front; the clipboard is emptied first.
fn copy(ctx: &Ctx, app: &Opened) -> Result<String, String> {
    let t = &ctx.cfg.timing;
    keys_ours(app)?;
    clip::write_text(None, t.read_wait_ms, t.poll_ms)?;
    let before = clip::sequence();
    keys::combo(&ctx.cfg.keys.copy)?;
    if !clip::wait_change(before, t.read_wait_ms, t.poll_ms) {
        return Err("nothing was copied".to_string());
    }
    Ok(clip::read_text(t.read_wait_ms, t.poll_ms, ctx.cfg.g1.max_read_chars)?.unwrap_or_default())
}

/// Copies the selection; passes when it is a part of the known text, which is then the only text logged.
pub fn copied(ctx: &Ctx, app: &Opened) -> (bool, Value) {
    match copy(ctx, app) {
        Ok(c) if part_of(&c, &ctx.cfg.probes.text) => (true, json!({ "copied": c })),
        Ok(c) => (false, json!({ "copied_chars": c.chars().count() })),
        Err(e) => (false, json!({ "error": e })),
    }
}

/// Removes from Windows clipboard history the entries made since `since` whose text `is_test`; reports how many.
pub fn forget_copies(ctx: &Ctx, since: i64, is_test: &(dyn Fn(&str) -> bool + Sync)) -> Value {
    let pr = &ctx.cfg.probes;
    sleep_ms(pr.history_settle_ms);
    let scope = winclip::Scope {
        since,
        max_chars: pr.forget_max_chars,
        is_ours: is_test,
    };
    match winclip::forget(&scope) {
        Ok(n) => json!(n),
        Err(e) => json!({ "error": e }),
    }
}

/// True for a copy G17 or G18 made: a start of the probe text, at least `forget_min_chars` long.
pub fn probe_copy(ctx: &Ctx, text: &str) -> bool {
    let pr = &ctx.cfg.probes;
    winclip::leads(text, &pr.text, pr.forget_min_chars)
}

/// True when `copied` is a non-empty part of `known`, ignoring spaces and line ends at its edges.
pub fn part_of(copied: &str, known: &str) -> bool {
    let c = copied.trim();
    !c.is_empty() && known.contains(c)
}

/// The start and middle points of the first text line in our window, once UI Automation shows one.
pub fn text_points(ctx: &Ctx, uia: &Uia, app: &Opened) -> Result<(Pt, Pt), String> {
    let t = &ctx.cfg.timing;
    let line = win::poll_until(t.read_wait_ms, t.poll_ms, || {
        uia::lines(&uia.text(app.hwnd)?).ok()?.into_iter().next()
    })
    .ok_or("no text line through UI Automation")?;
    let (start, mid) = uia::line_points(&line, ctx.cfg.probes.inset_px);
    guard(app, &[start, mid])?;
    Ok((start, mid))
}

/// Explorer item `name` in our window and where to press it, once UI Automation shows it.
pub fn item(
    ctx: &Ctx,
    uia: &Uia,
    app: &Opened,
    name: &str,
) -> Result<(IUIAutomationElement, Pt), String> {
    let t = &ctx.cfg.timing;
    let found = win::poll_until(t.read_wait_ms, t.poll_ms, || {
        let it = uia.item(app.hwnd, name)?;
        let p = uia.grip(&it, name).ok()?;
        Some((it, p))
    })
    .ok_or_else(|| format!("no item {name} through UI Automation"))?;
    guard(app, &[found.1])?;
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtf_escapes_specials_and_non_ascii_and_ends_each_line() {
        let got = rtf("a{b}\\c\nلا", "Calibri", 28);
        assert!(got.starts_with(r"{\rtf1\ansi\deff0{\fonttbl{\f0 Calibri;}}\f0\fs28 "));
        assert!(got.ends_with(RTF_TAIL));
        assert!(got.contains(r"a\{b\}\\c\par "));
        let lam_alef = ["\\", "u1604?", "\\", "u1575?", "\\par "].concat();
        assert!(got.contains(&lam_alef));
    }

    #[test]
    fn scroll_text_has_numbered_lines_of_the_asked_width() {
        let text = scroll_text(3, 12);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines, ["0001 abcdefg", "0002 abcdefg", "0003 abcdefg"]);
        assert_eq!(scroll_text(1, 2), "00");
    }

    #[test]
    fn only_a_non_empty_part_of_the_known_text_passes() {
        let known = "Hold still to grab";
        assert!(part_of("still to", known));
        assert!(part_of(" Hold\r\n", known));
        assert!(!part_of("  ", known));
        assert!(!part_of("other", known));
    }

    #[test]
    fn kind_of_accepts_only_the_gate_apps() {
        let apps = [AppKind::Notepad, AppKind::Explorer];
        assert_eq!(kind_of("explorer", &apps), Ok(AppKind::Explorer));
        assert!(kind_of("word", &apps).is_err());
    }
}
