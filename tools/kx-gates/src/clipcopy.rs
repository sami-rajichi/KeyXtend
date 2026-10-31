//! The copy G20 and `g8 clip` start: the plain kx-gates starts the test build without its gate flag, on the data folder
//! beside it, never the owner's. Its settings bytes are kept and come back, and its history is emptied before and after.
#![cfg(windows)]

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::Instant;

use kx_mod_clipboard::{ClipSettings, ID, SPEC};
use kx_platform::PORTABLE_DIR;
use kx_platform_windows::uiaccess;
use kx_platform_windows::worker::WORKER_EXE;
use kx_settings::VERSION_KEY;
use toml::{Table, Value};

use crate::apps::Ctx;
use crate::cfgfile::P4_FILE;
use crate::copyend::{self, Ended};
use crate::g8start::{self, Started};
use crate::owncopy;
use crate::ownset::Kept;
use crate::product::{self, NAME};

/// The copy's session: the test build, its history folder, and its settings bytes kept until `close`.
pub struct ClipSession<'a> {
    ctx: &'a Ctx<'a>,
    /// The test build's program.
    pub program: PathBuf,
    data: PathBuf,
    /// The session made the data folder, so it goes again at the end and later runs use the owner's.
    made: bool,
    kept: Kept,
    /// Set once a copy could not be ended, so the backup stays.
    stray: Cell<bool>,
}

/// Refuses a run under uiAccess, which the test build would inherit, or while a product copy runs, whose watch would
/// keep the gate's copies.
fn refuse(ui_access: Result<bool, String>, running: bool) -> Result<(), String> {
    match ui_access {
        Err(e) => Err(format!("kx-gates cannot tell whether it has uiAccess: {e}")),
        Ok(true) => Err(
            "run the plain kx-gates (cargo run -p kx-gates): the test build would inherit uiAccess"
                .into(),
        ),
        Ok(false) if running => Err(format!(
            "close {NAME} first: its watch would keep the gate's copies"
        )),
        Ok(false) => Ok(()),
    }
}

/// The settings for a run: `file` with its `[clipboard]` section the defaults, `history_key` and `skip_app` alone
/// skipped; every other section stays.
pub(crate) fn clip_table(file: &Table, history_key: bool, skip_app: &str) -> Result<Table, String> {
    let mut s =
        ClipSettings::from_defaults().map_err(|e| format!("the clipboard defaults: {e}"))?;
    s.history_key = history_key;
    s.skip_apps = vec![skip_app.to_string()];
    let mut section =
        Table::try_from(&s).map_err(|e| format!("the gate's clipboard settings: {e}"))?;
    section.insert(
        VERSION_KEY.to_string(),
        Value::Integer(i64::from(SPEC.version)),
    );
    let mut table = file.clone();
    table.insert(ID.as_str().to_string(), Value::Table(section));
    Ok(table)
}

/// The test build's program, as `[gate_input] program` names it; an error says how to build it.
pub(crate) fn program(ctx: &Ctx) -> Result<PathBuf, String> {
    let path = ctx.cfg.program(&ctx.cfg.gate_input.program);
    if path.is_file() {
        return Ok(path);
    }
    Err(format!(
        "{} is missing: build the test build, see [gate_input] in {P4_FILE}",
        path.display()
    ))
}

/// The worker program beside `program`, which draws the history's pictures; an error says how to build it.
pub(crate) fn worker(program: &Path) -> Result<(), String> {
    let path = program.with_file_name(WORKER_EXE);
    if path.is_file() {
        return Ok(());
    }
    Err(format!(
        "{} is missing: build keyxtend-worker into the test build's folder",
        path.display()
    ))
}

/// Makes the data folder beside `program`, so the copy is portable, and checks the product reads it there: the folder,
/// and whether this call made it.
fn own_data(program: &Path) -> Result<(PathBuf, bool), String> {
    let data = program
        .parent()
        .ok_or("the test build has no folder")?
        .join(PORTABLE_DIR);
    let made = !data.exists();
    std::fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
    match product::files(program) {
        Ok(f) if f.dir == data => Ok((data, made)),
        Ok(_) => {
            let e = format!(
                "{} is not the copy's data folder, so it would use another",
                data.display()
            );
            Err(undone(&data, made, e))
        }
        Err(e) => Err(undone(&data, made, e)),
    }
}

/// Error `e`, after deleting the data folder `data` when the session `made` it.
fn undone(data: &Path, made: bool, e: String) -> String {
    match made.then(|| empty(data)) {
        Some(Err(left)) => format!("{e}; {left}"),
        _ => e,
    }
}

/// Deletes folder `dir` of the gate's own data folder, when there is one.
fn empty(dir: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(format!("{}: {e}", dir.display()))
        }
        _ => Ok(()),
    }
}

impl<'a> ClipSession<'a> {
    /// Keeps the test build's settings bytes and empties its history, refused under uiAccess or while a copy runs.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        refuse(
            uiaccess::check().map_err(|e| e.to_string()),
            owncopy::runs(),
        )?;
        let g = &ctx.cfg.g20_product;
        let program = program(ctx)?;
        let (data, made) = own_data(&program)?;
        let side = ctx.out.join(&g.backup);
        let taken = product::files(&program).and_then(|f| Kept::take(&f, &side));
        let kept = match taken {
            Ok((kept, _)) => kept,
            Err(e) => return Err(undone(&data, made, e)),
        };
        empty(&data.join(&g.history))?;
        Ok(Self {
            ctx,
            program,
            data,
            made,
            kept,
            stray: Cell::default(),
        })
    }

    /// A copy on the gate's settings with `history_key`, ready `warm_ms` after its keyboard shows, `drive` run on it,
    /// then ended: what `drive` gave, and how the copy went, none when none started.
    pub fn with_copy<T>(
        &self,
        history_key: bool,
        warm_ms: u64,
        drive: impl FnOnce(&Started) -> Result<T, String>,
    ) -> (Result<T, String>, Option<Ended>) {
        let g = &self.ctx.cfg.g20_product;
        let set = clip_table(&self.kept.table(), history_key, &g.skip_app)
            .and_then(|t| self.kept.write(&t));
        if let Err(e) = set {
            return (Err(e), None);
        }
        let started = match g8start::start(self.ctx, &self.program, &[], None) {
            Ok(s) => s,
            Err(f) => {
                self.note(f.ended.as_ref());
                return (Err(f.why), f.ended);
            }
        };
        let got = ready(&started, warm_ms).and_then(|()| drive(&started));
        let ended = copyend::end(self.ctx, &started.process);
        self.note(Some(&ended));
        (got, Some(ended))
    }

    /// Marks the session stray when a copy that went as `ended` may still run.
    fn note(&self, ended: Option<&Ended>) {
        if !copyend::may_follow(ended.map(|e| e.gone)) {
            self.stray.set(true);
        }
    }

    /// Puts the test build's settings bytes back, then empties its history, or deletes the data folder the session
    /// made; all of it stays while a copy may run.
    pub fn close(self) -> Result<(), String> {
        let stray = self.stray.get();
        self.kept.put_back(stray)?;
        let gone = if self.made {
            self.data.clone()
        } else {
            self.data.join(&self.ctx.cfg.g20_product.history)
        };
        if stray { Ok(()) } else { empty(&gone) }
    }
}

/// Waits until `warm_ms` past the copy's keyboard showing; an error when it never showed.
fn ready(s: &Started, warm_ms: u64) -> Result<(), String> {
    let shown = s.shown_at.ok_or("the keyboard did not show in time")?;
    std::thread::sleep(g8start::warm_left(shown, warm_ms, Instant::now()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use kx_settings::peek_section;

    use super::*;

    #[test]
    fn a_run_under_ui_access_or_beside_a_running_copy_is_refused() {
        assert_eq!(refuse(Ok(false), false), Ok(()));
        let e = refuse(Ok(true), false).expect_err("uiAccess");
        assert!(e.contains("uiAccess") && e.contains("cargo run"), "{e}");
        let e = refuse(Ok(false), true).expect_err("a copy runs");
        assert!(e.contains(&format!("close {NAME}")), "{e}");
        let e = refuse(Err("no token".into()), false).expect_err("unknown");
        assert!(e.contains("no token"), "{e}");
    }

    #[test]
    fn the_runs_settings_are_the_clipboard_defaults_with_its_key_and_skipped_app() {
        let file: Table = "[keyboard]\nrelabel_ms = 700\n[clipboard]\nkeep_days = 3\n"
            .parse()
            .expect("TOML");
        let table = clip_table(&file, false, "notepad.exe").expect("built");
        assert_eq!(table["keyboard"], file["keyboard"], "another section stays");
        let s: ClipSettings = peek_section(Some(&table), ID, &SPEC).expect("reads back");
        let d = ClipSettings::from_defaults().expect("defaults");
        assert!(!s.history_key);
        assert_eq!(s.skip_apps, ["notepad.exe"]);
        assert_eq!(
            (s.keep_days, s.join_breaks),
            (d.keep_days, d.join_breaks),
            "the defaults"
        );
        let on = clip_table(&Table::new(), true, "notepad.exe").expect("built");
        let s: ClipSettings = peek_section(Some(&on), ID, &SPEC).expect("reads back");
        assert!(s.history_key);
    }
}
