//! The copy G20, `g8 clip` and G25 start: the plain kx-gates starts the test build without its gate flag, on the data
//! folder beside it (`clipdata`), never the owner's. Its settings bytes are kept and come back, its history emptied.
#![cfg(windows)]

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::Instant;

use kx_mod_clipboard::{ClipSettings, ID, SPEC};
use kx_platform::CLIP_DIR;
use kx_platform_windows::uiaccess;
use kx_platform_windows::worker::WORKER_EXE;
use kx_settings::VERSION_KEY;
use toml::{Table, Value};

use crate::apps::Ctx;
use crate::cfgfile::P4_FILE;
use crate::copyend::{self, Ended};
use crate::g8start::{self, Started};
use crate::out::say;
use crate::ownset::Kept;
use crate::product::{self, NAME};
use crate::{clipdata, owncopy};

/// The copy's session: the test build, its data folder, and its settings bytes kept until `close`.
pub struct ClipSession<'a> {
    ctx: &'a Ctx<'a>,
    /// The test build's program.
    pub program: PathBuf,
    data: PathBuf,
    kept: Kept,
    /// Set once a copy could not be ended, so the backup stays.
    stray: Cell<bool>,
    /// The settings came back first from a sidecar a stopped run left.
    pub recovered: bool,
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

/// How the sidecar left by a copy that may still run comes back.
const LATER: &str =
    "the next G20, g8 clip or G25 start of the same kx-gates restores from it once no copy runs";

impl<'a> ClipSession<'a> {
    /// Readies the test build's data folder and keeps its settings bytes, then empties its history; refused under
    /// uiAccess or while a copy runs.
    pub fn open(ctx: &'a Ctx<'a>) -> Result<Self, String> {
        refuse(
            uiaccess::check().map_err(|e| e.to_string()),
            owncopy::runs(),
        )?;
        let (g, program) = (&ctx.cfg.g20_product, program(ctx)?);
        let data = clipdata::own(&program, &g.marker)?;
        let side = ctx.out.join(&g.backup);
        let taken = product::files(&program).and_then(|f| Kept::take_as(&f, &side, LATER));
        let (kept, recovered) = taken.map_err(|e| clipdata::undone(&data, &g.marker, e))?;
        if let Err(e) = clipdata::remove(&data.join(CLIP_DIR)) {
            let back = kept.put_back(false).err().map(|b| format!("; {b}"));
            let e = format!("{e}{}", back.unwrap_or_default());
            return Err(clipdata::undone(&data, &g.marker, e));
        }
        if recovered {
            say!(
                "the test build's settings came back first from {}, which a stopped run left",
                side.display()
            );
        }
        let stray = Cell::default();
        Ok(Self {
            ctx,
            program,
            data,
            kept,
            stray,
            recovered,
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

    /// Puts the test build's settings bytes back, then deletes the data folder when it is marked, else empties its
    /// history; while a copy may run, the sidecar and the folder stay.
    pub fn close(self) -> Result<(), String> {
        let g = &self.ctx.cfg.g20_product;
        self.kept.put_back(self.stray.get())?;
        clipdata::finish(&self.data, &g.marker)
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
    use kx_settings::{Files, peek_section};
    use kx_test_support::tempdir::TempDir;

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

    #[test]
    fn a_clip_sidecar_that_stays_names_the_next_clip_gate_as_its_way_back() {
        let dir = TempDir::new("gates-clipcopy").expect("a temp folder");
        let data = dir.path().join("data");
        std::fs::create_dir(&data).expect("the data folder");
        let (files, side) = (Files::new(&data), dir.path().join("side.json"));
        let (kept, recovered) = Kept::take_as(&files, &side, LATER).expect("kept");
        assert!(!recovered);
        let e = kept.put_back(true).expect_err("a copy may still run");
        assert!(e.contains(LATER) && !e.contains("close"), "{e}");
        assert!(side.exists(), "the next start restores from it");
    }
}
