//! The staged worker's edges (`g22 staged`, the plan's edges 28, 29 and 39) on a copy of what `cargo xtask stage`
//! wrote, through the product's own speech host: no screen, no mouse and no sound file, as the staged worker takes none.
//! The copy and edge 39's models folder go at the end, on a failure too; the stage is only read. Numbers only print.
#![cfg(windows)]

mod copy;
mod edges;

use std::path::{Path, PathBuf};
use std::time::Duration;

use kx_mod_voice::VoiceSettings;
use kx_mod_voice::models::{Row, table as models};
use kx_platform::speech::VOICE_FILES;
use kx_platform_windows::worker::WORKER_EXE;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::g22::STAGED;
use crate::g22cfg::{Clip, G22Bench, G22Staged};
use crate::out;
use copy::{Edge, Made};
use edges::Host;

/// What the edges read once: the settings, the stage with its entries, the clip and its default model's file.
struct Run<'a> {
    g: &'a G22Staged,
    b: &'a G22Bench,
    stage: PathBuf,
    entries: Vec<(String, bool)>,
    names: Vec<String>,
    settings: VoiceSettings,
    row: &'static Row,
    clip: &'a Clip,
    model: PathBuf,
}

/// Runs the edges in order on one copy, each deleting more of it, then removes the copy.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let run = Run::of(&ctx.cfg.g22_staged, &ctx.cfg.g22_bench, &ctx.cfg.dir)?;
    let made = Made::new(std::env::temp_dir().join(&run.g.copy))?;
    copy::fill(&run.stage, &made.path, &run.entries, VOICE_FILES)?;
    let dir =
        out::plain(&made.path).map_err(|_| format!("{} could not be read back", run.g.copy))?;
    let host = run.host(&dir);
    let lines: Vec<Value> = Edge::ORDER.iter().map(|&e| run.edge(e, &host)).collect();
    let removed = made.remove();
    let pass = lines.iter().all(|l| l["pass"] == true) && removed.is_ok();
    Ok(json!({
        "gate": "G22", "name": STAGED, "edges": lines, "copy_removed": removed.is_ok(),
        "copy_error": removed.err(), "pass": pass,
    }))
}

impl<'a> Run<'a> {
    /// The run `g` and `b` describe, the stage found from the settings folder `dir`; a missing part is an error.
    fn of(g: &'a G22Staged, b: &'a G22Bench, dir: &Path) -> Result<Self, String> {
        let stage = out::plain(&dir.join(&g.stage))
            .map_err(|e| format!("{e}; run cargo xtask stage first"))?;
        let entries = copy::list(&stage)?;
        let names = copy::files(&entries);
        copy::need(&names, g, WORKER_EXE)?;
        let settings = VoiceSettings::from_defaults().map_err(|e| e.to_string())?;
        let table = models().map_err(ToString::to_string)?;
        let row = table
            .default_for(&g.lang)
            .ok_or("g22_staged.lang has no default model")?;
        let model = PathBuf::from(&b.models_dir).join(&row.file);
        if !model.is_file() {
            return Err(format!("{} is not in g22_bench.models_dir", row.file));
        }
        let clip = b.clips.get(&g.lang).ok_or("g22_staged.lang has no clip")?;
        Ok(Self {
            g,
            b,
            stage,
            entries,
            names,
            settings,
            row,
            clip,
            model,
        })
    }

    /// The edges' host over the copy at `dir`.
    fn host<'h>(&'h self, dir: &'h Path) -> Host<'h> {
        Host {
            copy: dir,
            settings: &self.settings,
            clip: self.clip,
            lang: &self.g.lang,
            row: self.row,
            poll: Duration::from_millis(self.g.poll_ms),
            // The host's goodbye within one answer, then the bench's end wait.
            end_ms: u64::from(self.settings.ask_ms).saturating_add(self.b.end_wait_ms),
            floor: &self.g.floor_part,
            prefix: &self.b.part_prefix,
        }
    }

    /// Edge `edge`'s line: its parts deleted from the copy first, then its reading, or why it has none. It passes only
    /// once its worker ended, so the next edge can delete the parts that worker loaded.
    fn edge(&self, edge: Edge, h: &Host) -> Value {
        eprintln!("g22 staged: edge {}", edge.number());
        let gone = copy::dropped(&self.names, edge, self.g, &self.b.part_prefix);
        let line = copy::delete(h.copy, &gone).and_then(|()| self.read(edge, h));
        let ended = h.let_go();
        let mut line =
            line.unwrap_or_else(|e| json!({ "edge": edge.number(), "error": e, "pass": false }));
        line["pass"] = json!(line["pass"] == true && ended);
        line["worker_ended"] = json!(ended);
        line
    }

    /// Edge `edge`'s reading on the copy as it now is.
    fn read(&self, edge: Edge, h: &Host) -> Result<Value, String> {
        let diff = self.settings.card_diff;
        match edge {
            Edge::NoCard => edges::no_card(h).map(|r| r.json()),
            Edge::Floor => edges::floor(h, Path::new(&self.b.models_dir)).map(|r| r.json(diff)),
            Edge::Plant => self.planted(h),
        }
    }

    /// Edge 39 in a models folder made for it, which goes before the line is written.
    fn planted(&self, h: &Host) -> Result<Value, String> {
        let made = Made::new(Path::new(&self.b.models_dir).join(&self.g.plant))?;
        copy::plant(
            &made,
            &self.model,
            &h.copy.join(&self.g.floor_part),
            &self.g.plant_as,
        )?;
        let read = out::plain(&made.path).and_then(|dir| edges::plant(h, &dir));
        // The worker maps the linked model, so the folder goes only once it ended.
        let ended = h.let_go();
        let removed = made.remove();
        let mut line = read?.json();
        line["pass"] = json!(line["pass"] == true && ended && removed.is_ok());
        line["plant_removed"] = json!(removed.is_ok());
        Ok(line)
    }
}

#[cfg(test)]
mod tests;
