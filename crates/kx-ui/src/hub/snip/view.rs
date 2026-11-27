//! Snip's view as QML reads it: the frozen screens by provider URL, the stage's window with each key by id, the close
//! question, a note and the panel by id, the capture windows to keep hidden, and whether a system dialog sits in front.

use super::edit::{self, Edit};
use super::pics;
use crate::hub::wiped;
use crate::i18n::keys::{
    snip_bar_id, snip_dim_id, snip_handle_id, snip_keep_id, snip_keyboard_id, snip_note_id,
    snip_rec_id, snip_spot_id, snip_strip_id,
};
use kx_module_api::Zeroizing;
use kx_module_api::ports::{
    self, CloseAsk, Countdown, Ends, FrozenScreen, Keycap, Picking, Point, RecordAsk, Recording,
    Rect, SnipPage, SnipSheet, SnipStage, Speed, Taken, VideoSheet,
};
use serde::Serialize;

/// What Snip's windows draw, every key and reason by id.
#[derive(Serialize)]
pub(super) struct View<'a> {
    round: u32,
    frozen: Vec<Frozen>,
    stage: &'static str,
    pick: Option<Pick<'a>>,
    strip: Option<Strip>,
    count: Option<Count>,
    record: Option<Record>,
    edit: Option<Edit<'a>>,
    ask: Option<Ask>,
    note: Option<&'static str>,
    panel: Option<Panel>,
    /// The capture windows QML keeps hidden, as their shield was refused.
    hide: Vec<&'static str>,
    /// A system dialog sits in front, so Snip's windows leave the top.
    low: bool,
}

/// A frozen screen, its pixels by URL.
#[derive(Serialize)]
struct Frozen {
    rect: Rect,
    dpi: u32,
    src: String,
}

/// A key: its id, whether it is on, and why it stays dimmed.
#[derive(Serialize)]
struct Cap {
    key: &'static str,
    on: bool,
    dim: Option<&'static str>,
}

/// Picking: the bar's screen and keys, the first corner, the screen a video region keeps to, and Window's windows.
#[derive(Serialize)]
struct Pick<'a> {
    screen: Rect,
    keys: Vec<Cap>,
    corner: Option<Point>,
    within: Option<Rect>,
    windows: &'a [Rect],
}

/// A part taken, its strip, a recorded video's speed with its length and the key waiting at it, and a region's frame.
#[derive(Serialize)]
struct Strip {
    part: Rect,
    screen: Rect,
    spot: &'static str,
    keys: Vec<Cap>,
    length_ms: Option<u32>,
    speed: Option<u8>,
    done: Option<&'static str>,
    waits: Option<&'static str>,
    frame: Option<Frame>,
}

/// A region's or Crop's frame: the box it keeps within, its least side, and the handle held by id and where picked up.
#[derive(Serialize)]
pub(super) struct Frame {
    within: Rect,
    min: u16,
    grip: Option<Grip>,
}

/// A handle held.
#[derive(Serialize)]
struct Grip {
    handle: &'static str,
    at: Point,
}

/// The countdown.
#[derive(Serialize)]
struct Count {
    part: Rect,
    left_s: u8,
}

/// Recording, with the record bar.
#[derive(Serialize)]
struct Record {
    part: Rect,
    screen: Rect,
    spot: &'static str,
    elapsed_ms: u32,
    live: bool,
    keys: Vec<Cap>,
}

/// The close question.
#[derive(Serialize)]
struct Ask {
    save_dim: Option<&'static str>,
}

/// The Snip panel's page; the folder may name the user, so its text is wiped.
#[derive(Serialize)]
#[serde(tag = "tab", rename_all = "lowercase")]
enum Panel {
    Capture {
        keyboard: &'static str,
        #[serde(serialize_with = "wiped::maybe_text")]
        folder: Option<Zeroizing<String>>,
    },
    Video {
        countdown_s: u8,
        countdown: Limits,
        fps: u8,
        /// The frame rates the recorder offers, which the panel's two choices name.
        rates: [u8; 2],
        max_min: u16,
        longest: Limits,
        mic_on: bool,
        sound_on: bool,
    },
}

/// Where a stepper stands.
#[derive(Serialize)]
struct Limits {
    low: bool,
    high: bool,
}

/// `page` as QML reads it, with the pictures of `generation` and the windows to `hide`; it is `low` while the page says
/// a system dialog shows.
pub(super) fn of<'a>(page: &'a SnipPage, generation: u64, hide: Vec<&'static str>) -> View<'a> {
    let mut view = View {
        round: page.round,
        frozen: frozen(page, generation),
        stage: stage_name(&page.stage),
        pick: None,
        strip: None,
        count: None,
        record: None,
        edit: None,
        ask: page.ask.map(|a: CloseAsk| Ask {
            save_dim: a.save_dim.map(snip_dim_id),
        }),
        note: page.note.map(snip_note_id),
        panel: page.panel.as_ref().map(panel),
        hide,
        low: page.dialog,
    };
    match &page.stage {
        SnipStage::Idle => {}
        SnipStage::Pick(p) => view.pick = Some(pick(p)),
        SnipStage::Strip(t) => view.strip = Some(strip(t)),
        SnipStage::Count(c) => view.count = Some(count(*c)),
        SnipStage::Record(r) => view.record = Some(record(r)),
        SnipStage::Edit(e) => {
            view.edit = Some(edit::of(e, pics::src(generation, page.frozen.len())));
        }
    }
    view
}

/// `page`'s frozen screens, each picture by its URL in `generation`.
fn frozen(page: &SnipPage, generation: u64) -> Vec<Frozen> {
    let screen = |(place, f): (usize, &FrozenScreen)| Frozen {
        rect: f.rect(),
        dpi: f.dpi(),
        src: pics::src(generation, place),
    };
    page.frozen.iter().enumerate().map(screen).collect()
}

/// The stage's name, which says which window shows.
pub(super) const fn stage_name(stage: &SnipStage) -> &'static str {
    match stage {
        SnipStage::Idle => "idle",
        SnipStage::Pick(_) => "pick",
        SnipStage::Strip(_) => "strip",
        SnipStage::Count(_) => "count",
        SnipStage::Record(_) => "record",
        SnipStage::Edit(_) => "edit",
    }
}

/// `caps` with their ids by `id`.
fn keys<K: Copy>(caps: &[Keycap<K>], id: fn(K) -> &'static str) -> Vec<Cap> {
    let key = |c: &Keycap<K>| Cap {
        key: id(c.key),
        on: c.on,
        dim: c.dim.map(snip_dim_id),
    };
    caps.iter().map(key).collect()
}

fn pick(p: &Picking) -> Pick<'_> {
    Pick {
        screen: p.screen,
        keys: keys(&p.keys, snip_bar_id),
        corner: p.corner,
        within: p.within,
        windows: &p.windows,
    }
}

fn strip(t: &Taken) -> Strip {
    Strip {
        part: t.part.rect(),
        screen: t.screen,
        spot: snip_spot_id(t.spot),
        keys: keys(&t.keys, snip_strip_id),
        length_ms: t.length_ms,
        speed: t.speed.map(Speed::times),
        done: t.done.map(snip_keep_id),
        waits: t.waits.map(snip_strip_id),
        frame: t.frame.map(frame),
    }
}

/// A frame with its held handle by id.
pub(super) fn frame(f: ports::Frame) -> Frame {
    let grip = f.grip.map(|g| Grip {
        handle: snip_handle_id(g.handle),
        at: g.at,
    });
    Frame {
        within: f.within,
        min: f.min,
        grip,
    }
}

fn count(c: Countdown) -> Count {
    Count {
        part: c.part.rect(),
        left_s: c.left_s,
    }
}

fn record(r: &Recording) -> Record {
    Record {
        part: r.part.rect(),
        screen: r.screen,
        spot: snip_spot_id(r.spot),
        elapsed_ms: r.elapsed_ms,
        live: r.live,
        keys: keys(&r.keys, snip_rec_id),
    }
}

fn panel(sheet: &SnipSheet) -> Panel {
    let ends = |s: Ends| Limits {
        low: s.low,
        high: s.high,
    };
    match sheet {
        SnipSheet::Capture(c) => Panel::Capture {
            keyboard: snip_keyboard_id(c.keyboard),
            folder: c
                .folder
                .as_ref()
                .map(|f| Zeroizing::new(f.expose().display().to_string())),
        },
        SnipSheet::Video(v) => {
            let VideoSheet {
                countdown_s,
                fps,
                max_min,
                mic_on,
                sound_on,
                ..
            } = *v;
            Panel::Video {
                countdown_s,
                countdown: ends(v.countdown),
                fps,
                rates: RecordAsk::FPS,
                max_min,
                longest: ends(v.longest),
                mic_on,
                sound_on,
            }
        }
    }
}
