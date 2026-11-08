//! The `Tools` QObject: the tools row and the snip overlay for QML.

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use serde_json::{Value, json};
use spike_core::config::{SpikeConfig, ToolButton};
use spike_core::snip::{self, Snip};
use spike_core::{place, window};

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
        #[qproperty(QString, overlay_title, READ, CONSTANT)]
        #[qproperty(QString, snip_hint, READ, CONSTANT)]
        #[qproperty(QString, snip_size, READ, CONSTANT)]
        #[qproperty(f32, snip_edge, READ, CONSTANT)]
        type Tools = super::ToolsRust;

        /// The tool buttons as JSON: a label and a box each.
        #[qinvokable]
        fn buttons_json(&self) -> QString;

        /// Runs tool button `i`: JSON with a status `note`, the `snip` to show, or `mic` for QML's Voice.
        #[qinvokable]
        fn tool(self: Pin<&mut Self>, i: i32) -> QString;

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
    overlay_title: QString,
    snip_hint: QString,
    snip_size: QString,
    snip_edge: f32,
    cfg: SpikeConfig,
    /// The snip in progress and the window that was in front when it began.
    snip: Option<(Snip, isize)>,
}

impl Default for ToolsRust {
    fn default() -> Self {
        let cfg = crate::bridge::config().clone();
        let t = &cfg.tools;
        Self {
            overlay_title: QString::from(&t.overlay_title),
            snip_hint: QString::from(&t.snip_hint),
            snip_size: QString::from(&t.snip_size),
            snip_edge: t.snip_edge_px,
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

impl qobject::Tools {
    fn buttons_json(&self) -> QString {
        let all: Vec<Value> = place::tool_row(&self.rust().cfg)
            .into_iter()
            .map(|(label, p)| json!({ "label": label, "x": p.x, "y": p.y, "w": p.w, "h": p.h }))
            .collect();
        js(json!(all))
    }

    fn tool(mut self: Pin<&mut Self>, i: i32) -> QString {
        match usize::try_from(i).ok().and_then(ToolButton::at) {
            Some(ToolButton::Snip) => js(self.as_mut().begin_snip()),
            Some(ToolButton::Mic) => js(json!({ "mic": true })),
            None => js(json!({})),
        }
    }

    /// Freezes the screen and writes it for QML to show; JSON with its file URL and physical box.
    fn begin_snip(mut self: Pin<&mut Self>) -> Value {
        let prev = kx_platform_windows::window::foreground().0 as isize;
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

    fn pick(mut self: Pin<&mut Self>) -> QString {
        let dir = self.rust().cfg.tools.snip_dir();
        let mut me = self.as_mut().rust_mut();
        let Some((snip, prev)) = me.snip.as_mut() else {
            return QString::default();
        };
        let Some(note) = snip.pick(&dir) else {
            return QString::default();
        };
        let prev = kx_platform_windows::window::from_raw(*prev);
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
