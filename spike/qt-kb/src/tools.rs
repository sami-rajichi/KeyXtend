//! The `Tools` QObject: the tools row, the selection pill, quick-fill and the snip overlay for QML.

use std::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use serde_json::{Value, json};
use spike_core::config::{SpikeConfig, ToolButton};
use spike_core::fill;
use spike_core::hold::Pt;
use spike_core::note::{self, Note};
use spike_core::selwatch::{self, PillPx, PillStep, Watch};
use spike_core::snip::{self, Snip};
use spike_core::{inject, place, window};

use crate::bridge::{FACE, ms};

/// The cxx-qt bridge that makes `Tools` a QML type.
#[cxx_qt::bridge]
pub mod qobject {
    // SAFETY: the header and type are cxx-qt-lib's own QString binding, so both sides match.
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt string.
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, pill_title, READ, CONSTANT)]
        #[qproperty(QString, pill_label, READ, CONSTANT)]
        #[qproperty(QString, overlay_title, READ, CONSTANT)]
        #[qproperty(QString, snip_hint, READ, CONSTANT)]
        #[qproperty(QString, snip_size, READ, CONSTANT)]
        #[qproperty(f32, snip_edge, READ, CONSTANT)]
        #[qproperty(f32, pill_width, READ, CONSTANT)]
        #[qproperty(f32, pill_height, READ, CONSTANT)]
        #[qproperty(i32, poll_ms, READ, CONSTANT)]
        type Tools = super::ToolsRust;

        /// The tool buttons as JSON: a label and a box each.
        #[qinvokable]
        fn buttons_json(&self) -> QString;

        /// Runs tool button `i`: JSON with a status `note`, the `snip` to show, or `mic` for QML's Voice.
        #[qinvokable]
        fn tool(self: Pin<&mut Self>, i: i32) -> QString;

        /// JSON: the pill step (`show` or `move` with a physical point, or `hide`) and any new `note`.
        #[qinvokable]
        fn tick(self: Pin<&mut Self>) -> QString;

        /// The pill's Copy: sends Ctrl+C to the app in front; returns a note or empty.
        #[qinvokable]
        fn copy(&self) -> QString;

        /// One click on the overlay: empty while the snip goes on, else its note.
        #[qinvokable]
        fn pick(self: Pin<&mut Self>) -> QString;

        /// Guards every window we show, with the one called `top` above the others; returns an error note or empty.
        #[qinvokable]
        fn guard(&self, top: &QString) -> QString;

        /// Deletes the frozen screen file once QML has loaded it; returns an error note or empty.
        #[qinvokable]
        fn forget_frozen(&self) -> QString;
    }
}

/// Rust side of `Tools`.
pub struct ToolsRust {
    pill_title: QString,
    pill_label: QString,
    overlay_title: QString,
    snip_hint: QString,
    snip_size: QString,
    snip_edge: f32,
    pill_width: f32,
    pill_height: f32,
    poll_ms: i32,
    cfg: SpikeConfig,
    watch: Result<Watch, String>,
    /// Where the pill is now, or `None` while it is hidden.
    shown: Option<Pt>,
    note: Note,
    /// The snip in progress and the window that was in front when it began.
    snip: Option<(Snip, isize)>,
}

impl Default for ToolsRust {
    fn default() -> Self {
        let cfg = crate::bridge::config().clone();
        let t = &cfg.tools;
        let note: Note = Arc::default();
        let watch = Watch::start(t.selection_poll_ms, PillPx(t.pill_px));
        if let Err(e) = &watch
            && let Ok(mut n) = note.lock()
        {
            *n = Some(e.clone());
        }
        Self {
            pill_title: QString::from(&t.pill_title),
            pill_label: QString::from(&t.labels.copy),
            overlay_title: QString::from(&t.overlay_title),
            snip_hint: QString::from(&t.snip_hint),
            snip_size: QString::from(&t.snip_size),
            snip_edge: t.snip_edge_px,
            pill_width: t.pill_px[0],
            pill_height: t.pill_px[1],
            poll_ms: ms(t.selection_poll_ms),
            watch,
            shown: None,
            note,
            snip: None,
            cfg,
        }
    }
}

/// `v` as a QML string.
fn js(v: Value) -> QString {
    QString::from(&v.to_string())
}

/// An error as a QML note; empty when all went well.
fn note_of(r: Result<(), String>) -> QString {
    r.err().map(|e| QString::from(&e)).unwrap_or_default()
}

/// A pill step as JSON for QML.
fn step_json(s: PillStep) -> Value {
    match s {
        PillStep::Show(p) => json!({ "show": [p.x, p.y] }),
        PillStep::Move(p) => json!({ "move": [p.x, p.y] }),
        PillStep::Hide => json!({ "hide": true }),
        PillStep::Stay => json!({}),
    }
}

impl qobject::Tools {
    fn buttons_json(&self) -> QString {
        let all: Vec<Value> = place::tool_row(&self.rust().cfg)
            .into_iter()
            .map(|(label, p)| json!({ "label": label, "x": p.x, "y": p.y, "w": p.w, "h": p.h }))
            .collect();
        js(json!(all))
    }

    fn tool(mut self: Pin<&mut Self>, i: i32) -> QString {
        let t = self.rust().cfg.tools.clone();
        match usize::try_from(i).ok().and_then(ToolButton::at) {
            Some(ToolButton::FillUser) => js(json!({ "note": self.fill_with(t.test_user) })),
            Some(ToolButton::FillPassword) => {
                js(json!({ "note": self.fill_with(t.test_password) }))
            }
            Some(ToolButton::Snip) => js(self.as_mut().begin_snip()),
            Some(ToolButton::Mic) => js(json!({ "mic": true })),
            None => js(json!({})),
        }
    }

    /// Asks Windows Hello on its own thread, then types `value` into the app in front.
    fn fill_with(&self, value: String) -> &'static str {
        fill::start(&self.rust().cfg, FACE, value, &self.rust().note)
    }

    /// Freezes the screen and writes it for QML to show; JSON with its file URL and physical box.
    fn begin_snip(mut self: Pin<&mut Self>) -> Value {
        let prev = window::foreground().0 as isize;
        let t = &self.rust().cfg.tools;
        let dir = t.snip_dir();
        let started = Snip::start(t.shot_cap()).and_then(|s| s.frozen_file(&dir).map(|f| (s, f)));
        let (snip, file) = match started {
            Ok(x) => x,
            // A write that failed halfway may leave part of the private screen behind.
            Err(e) => return json!({ "note": snip::forget_frozen(&dir).err().unwrap_or(e) }),
        };
        let sh = snip.shot();
        let url = QUrl::from_local_file(&QString::from(&*file.to_string_lossy())).to_qstring();
        let v = json!({ "snip": {
            "url": String::from(&url), "x": sh.left, "y": sh.top, "w": sh.width, "h": sh.height,
        } });
        self.as_mut().rust_mut().snip = Some((snip, prev));
        v
    }

    fn tick(mut self: Pin<&mut Self>) -> QString {
        let now = self.rust().watch.as_ref().ok().and_then(Watch::spot);
        let before = std::mem::replace(&mut self.as_mut().rust_mut().shown, now);
        let mut v = step_json(selwatch::step(before, now));
        v["note"] = json!(note::take(&self.rust().note));
        js(v)
    }

    fn copy(&self) -> QString {
        note_of(inject::combo(&self.rust().cfg.tools.copy_keys))
    }

    fn pick(mut self: Pin<&mut Self>) -> QString {
        let dir = self.rust().cfg.tools.snip_dir();
        let mut me = self.as_mut().rust_mut();
        let Some((snip, prev)) = me.snip.as_mut() else {
            return QString::default();
        };
        let Some(note) = snip.pick(&dir) else {
            return QString::default();
        };
        let prev = window::from_raw(*prev);
        me.snip = None;
        window::bring_back(prev, 0);
        QString::from(&note)
    }

    fn guard(&self, top: &QString) -> QString {
        note_of(window::guard_with_top(&String::from(top)).map(drop))
    }

    fn forget_frozen(&self) -> QString {
        note_of(snip::forget_frozen(&self.rust().cfg.tools.snip_dir()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_steps_reach_qml_as_small_json() {
        let p = Pt { x: 5, y: 7 };
        assert_eq!(step_json(PillStep::Show(p)), json!({ "show": [5, 7] }));
        assert_eq!(step_json(PillStep::Move(p)), json!({ "move": [5, 7] }));
        assert_eq!(step_json(PillStep::Hide), json!({ "hide": true }));
        assert_eq!(step_json(PillStep::Stay), json!({}));
    }
}
