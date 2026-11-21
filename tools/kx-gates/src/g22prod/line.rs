//! One line of G22 on the product: each clip dictated on a copy of its own, with the line's waits, and judged by who
//! must write its text, a device or a cloud service. A passing language adds where its time went; a failed one the
//! copy's marked lines and, on a cloud line, its faults' kinds.

use std::path::Path;
use std::time::Duration;

use kx_mod_voice::VoiceSettings;
use serde_json::{Value, json};

use super::stages;
use super::wrote::{self, Writer};
use super::{Case, GATE, Said, say, why};
use crate::apps::Ctx;
use crate::budget::now_us;
use crate::copyend::{self, Ended};
use crate::freshcopy::Fresh;
use crate::g22;
use crate::g22bench::within;
use crate::product::NAME;
use crate::typebox;

/// A dictation's waits: before Dictation once the layout is in front, after the bar shows beside the clip and the
/// bench's tail, and the longest for the text after Stop.
#[derive(Debug, Clone, Copy)]
pub(super) struct Waits {
    pub(super) before_ms: u64,
    pub(super) open_ms: u64,
    pub(super) text: Duration,
}

/// What a line dictates with: its waits and who must write its texts.
pub(super) struct Line {
    pub(super) waits: Waits,
    pub(super) want: Writer,
}

impl Line {
    /// The local line: a card's wait when one is picked, the load after the bar, the speech host's own wait for the
    /// text, and the device `[g22_product] device` allows.
    pub(super) fn local(ctx: &Ctx, s: &VoiceSettings) -> Result<Self, String> {
        let g = &ctx.cfg.g22_product;
        let pick = g.pick().ok_or("g22_product.device is no device pick")?;
        let waits = Waits {
            before_ms: g.dictate_wait_ms(),
            open_ms: g.open_ms,
            text: within(s),
        };
        Ok(Self {
            waits,
            want: Writer::Device(pick),
        })
    }

    /// The cloud line of service `id`: no card wait, `[g22_cloud]`'s waits, and only that service may write.
    pub(super) fn cloud(ctx: &Ctx, id: &str) -> Self {
        let c = &ctx.cfg.g22_cloud;
        let waits = Waits {
            before_ms: 0,
            open_ms: c.open_ms,
            text: Duration::from_millis(c.text_wait_ms),
        };
        Self {
            waits,
            want: Writer::Service(id.to_owned()),
        }
    }
}

/// `cases` on copies of the gate build at `program`, on a data folder `ready` readies first and the run removes at the
/// end: the line's fields, which pass when every language passed and the folder went.
pub(super) fn on_copy(
    ctx: &Ctx,
    program: &Path,
    run: (&[&Case], &VoiceSettings, &Line),
    ready: impl FnOnce(&Fresh) -> Result<(), String>,
) -> Result<Value, String> {
    let fresh = Fresh::open_at(ctx, GATE, program.to_path_buf())?;
    let (lines, ended) = match ready(&fresh) {
        Ok(()) => dictate_all(ctx, &fresh, run),
        Err(e) => (vec![json!({ "error": e, "pass": false })], None),
    };
    let removed = fresh.close(ended.as_ref());
    let pass = !lines.is_empty() && lines.iter().all(|l| l["pass"] == true) && removed.is_ok();
    Ok(json!({
        "langs": lines, "copy": ended.as_ref().map(Ended::line),
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

/// Each case on a copy of its own, while each copy before it ended: the lines, and how the last copy went.
fn dictate_all(
    ctx: &Ctx,
    fresh: &Fresh,
    (cases, s, line): (&[&Case], &VoiceSettings, &Line),
) -> (Vec<Value>, Option<Ended>) {
    let (mut lines, mut last) = (Vec::new(), None);
    for case in cases {
        let (out, ended) = dictate(ctx, fresh, case, s, line);
        lines.push(out);
        let over = copyend::may_follow(ended.as_ref().map(|e| e.gone));
        last = ended;
        if !over {
            break;
        }
    }
    (lines, last)
}

/// `case` on a copy of its own: its line, which passes only when the copy quit and the line's writer wrote the text,
/// and how the copy went.
fn dictate(
    ctx: &Ctx,
    fresh: &Fresh,
    case: &Case,
    s: &VoiceSettings,
    line: &Line,
) -> (Value, Option<Ended>) {
    let g = &ctx.cfg.g22_product;
    eprintln!("{} {NAME}: {}", g22::GATE, case.lang);
    let (args, since) = ([g22::sound_flag(), case.path.clone()], now_us());
    let drive = |pid| one(ctx, pid, case, &line.waits);
    let (got, ended) = fresh.with_args(&args, g.ready_ms, Some(&g.warn_line), drive);
    let log = fresh.log();
    let read = (log.as_ref().map_err(Clone::clone)).and_then(|l| wrote::read(l, since));
    let (writer, fit) = wrote::judge(&read, &line.want);
    let mut out = got.map_or_else(
        |e| json!({ "lang": case.lang, "error": e, "pass": false }),
        |l| l.json_with(s.card_diff),
    );
    let quit = ended.as_ref().is_some_and(Ended::ok);
    out["pass"] = json!(out["pass"] == true && quit && fit);
    out["copy_quit"] = json!(quit);
    if let (Some(to), Value::Object(from)) = (out.as_object_mut(), writer) {
        to.extend(from);
    }
    if let Ok(log) = &log {
        tell(ctx, &mut out, (log, since), line);
    }
    (out, ended)
}

/// Adds what the copy's `log` says since the language began to its line `out`: where its time went once it passed,
/// else the marked lines and, on a cloud line, its faults' kinds.
fn tell(ctx: &Ctx, out: &mut Value, (log, since): (&str, i64), line: &Line) {
    let g = &ctx.cfg.g22_product;
    if out["pass"] == true {
        let timing = stages::read(log, since);
        out["timing"] = timing.map_or_else(|e| json!({ "error": e }), |t| json!(t));
        return;
    }
    out["log"] = json!(why::marked(log, since, &g.why_marks));
    if matches!(line.want, Writer::Service(_)) {
        out["faults"] = json!(wrote::faults(log, since));
    }
}

/// One case in a fresh target-window on the copy `pid`: its line, and whether the box's first layout came back.
fn one(ctx: &Ctx, pid: u32, case: &Case, waits: &Waits) -> Result<Done, String> {
    let boxed = typebox::in_target(ctx, &[], |t| say::dictate(ctx, pid, t, case, waits));
    Ok(Done {
        said: boxed.got?,
        back: boxed.back,
        left: boxed.left,
    })
}

/// A case's end: its reading, whether the layout came back and what clean-up let go.
struct Done {
    said: Said,
    back: bool,
    left: Vec<String>,
}

impl Done {
    /// Its line: the reading's, with the layout's return and the clean-up, which the pass needs too.
    fn json_with(&self, diff: u8) -> Value {
        let mut v = self.said.json(diff);
        v["pass"] = json!(v["pass"] == true && self.back);
        v["layout_back"] = json!(self.back);
        v["clean_up"] = json!(self.left);
        v
    }
}
