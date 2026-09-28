//! The `Board` QObject: the stage-2 keyboard for QML. spike-core's controller decides; this only turns it into JSON.

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QString, QUrl};
use serde_json::json;
use spike_core::kbctl::{Kb, Tapped};
use spike_core::lookcfg::ModeChoice;
use spike_core::sysui::{self, Watch};
use spike_core::{layout, legend, screen};

/// The cxx-qt bridge that makes `Board` a QML type.
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
        #[qproperty(i32, poll_ms, READ, CONSTANT)]
        #[qproperty(QString, start_note, READ, CONSTANT)]
        type Board = super::BoardRust;

        /// Every key's box, icon, label and name at the current size, with the plate and D-pad.
        #[qinvokable]
        fn view_json(&self) -> QString;

        /// Legends, lights, the language key and the space bar now.
        #[qinvokable]
        fn state_json(self: Pin<&mut Self>) -> QString;

        /// Colours, shapes and sizes of the current theme and mode.
        #[qinvokable]
        fn look_json(&self) -> QString;

        /// Top bar and test strip: buttons, size labels, chips, theme names, font and icon URLs.
        #[qinvokable]
        fn bar_json(&self) -> QString;

        /// Types sample chip `i`; returns a note, empty when it went well.
        #[qinvokable]
        fn chip(self: Pin<&mut Self>, i: i32) -> QString;

        /// A click on key `id`; `back` is the language key's left third. JSON with any `note`, `tool` and `panel`.
        #[qinvokable]
        fn tap(self: Pin<&mut Self>, id: &QString, back: bool) -> QString;

        /// Shows theme `i`.
        #[qinvokable]
        fn set_theme(self: Pin<&mut Self>, i: i32);

        /// Mode `i` of `MODES`: light, dark or follow Windows.
        #[qinvokable]
        fn set_mode(self: Pin<&mut Self>, i: i32);

        /// The mic started or stopped recording; its key follows.
        #[qinvokable]
        fn set_rec(self: Pin<&mut Self>, on: bool);

        /// The top bar's light or dark switch.
        #[qinvokable]
        fn flip_mode(self: Pin<&mut Self>);

        /// One size step; true when the size changed.
        #[qinvokable]
        fn step(self: Pin<&mut Self>, up: bool) -> bool;

        /// Size preset `i`; true when the size changed.
        #[qinvokable]
        fn preset(self: Pin<&mut Self>, i: i32) -> bool;

        /// Starts a resize with the pointer at (`x`, `y`), in Qt units.
        #[qinvokable]
        fn resize_begin(self: Pin<&mut Self>, x: f32, y: f32);

        /// Follows the pointer; true when the size changed.
        #[qinvokable]
        fn resize_at(self: Pin<&mut Self>, x: f32, y: f32) -> bool;

        /// Ends the resize at the size it reached.
        #[qinvokable]
        fn resize_end(self: Pin<&mut Self>);

        /// True when the app in front switched layout.
        #[qinvokable]
        fn relabel(self: Pin<&mut Self>) -> bool;

        /// The pointer in physical pixels, as JSON `[x, y]`.
        #[qinvokable]
        fn cursor(&self) -> QString;

        /// Windows' dark mode, accent or high contrast changed.
        #[qsignal]
        fn look_changed(self: Pin<&mut Self>);
    }

    impl cxx_qt::Threading for Board {}
    impl cxx_qt::Initialize for Board {}
}

/// The modes in the test strip's order, which `spike.toml [bar] modes` labels.
const MODES: [ModeChoice; 3] = [ModeChoice::Light, ModeChoice::Dark, ModeChoice::Auto];

/// Rust side of `Board`.
pub struct BoardRust {
    poll_ms: i32,
    /// Why the keyboard cannot follow Windows' look, or empty.
    start_note: QString,
    kb: Kb,
    /// Keeps the look watcher running while the board lives.
    watch: Option<Watch>,
}

impl Default for BoardRust {
    fn default() -> Self {
        let cfg = crate::bridge::config().clone();
        let themes = crate::bridge::themes().clone();
        Self {
            poll_ms: crate::bridge::ms(cfg.size.poll_ms),
            start_note: QString::default(),
            kb: Kb::new(cfg, themes, sysui::read()),
            watch: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::Board {
    fn initialize(mut self: Pin<&mut Self>) {
        let qt = self.qt_thread();
        let watch = sysui::watch(move |look| {
            let _ = qt.queue(move |mut b| {
                b.as_mut().rust_mut().kb.set_sys(look);
                b.look_changed();
            });
        });
        match watch {
            Ok(w) => self.as_mut().rust_mut().watch = Some(w),
            Err(e) => self.as_mut().rust_mut().start_note = QString::from(&e),
        }
    }
}

/// `v` as a QML string.
fn js(v: &impl serde::Serialize) -> QString {
    QString::from(&serde_json::to_string(v).unwrap_or_default())
}

/// `path` as a file URL QML can load.
fn url(path: &std::path::Path) -> String {
    QUrl::from_local_file(&QString::from(&path.display().to_string()))
        .to_qstring()
        .to_string()
}

impl qobject::Board {
    fn view_json(&self) -> QString {
        js(&self.rust().kb.view())
    }

    fn state_json(mut self: Pin<&mut Self>) -> QString {
        js(&self.as_mut().rust_mut().kb.state())
    }

    fn look_json(&self) -> QString {
        let kb = &self.rust().kb;
        let mut v = serde_json::to_value(kb.look()).unwrap_or_default();
        v["theme"] = json!(kb.theme());
        v["mode"] = json!(MODES.iter().position(|&m| m == kb.mode()));
        js(&v)
    }

    fn bar_json(&self) -> QString {
        let kb = &self.rust().kb;
        let c = kb.cfg();
        let themes: Vec<&str> = kb
            .themes()
            .list
            .iter()
            .map(|t| t.look.name.as_str())
            .collect();
        let mut bar = serde_json::to_value(&c.bar).unwrap_or_default();
        let chips = c.bar.chips.iter();
        bar["chips"] = chips
            .map(
                |ch| json!({ "text": ch.text, "icon": ch.icon, "ar": legend::is_arabic(&ch.text) }),
            )
            .collect();
        js(&json!({
            "bar": bar,
            "themes": themes,
            "fonts": c.assets.font_files(&c.resolve(&c.assets.fonts)).iter().map(|p| url(p)).collect::<Vec<_>>(),
            "icons": url(&c.resolve(&c.assets.icons)),
            "iconExt": c.assets.icon_ext,
            "tipMs": c.look.tip_ms,
        }))
    }

    fn tap(mut self: Pin<&mut Self>, id: &QString, back: bool) -> QString {
        let id = id.to_string();
        let kb = &mut self.as_mut().rust_mut().kb;
        let plan = kb.plan(&id, layout::caps_on(), back);
        js(&match kb.run(&id, plan) {
            Ok(Tapped::Done) => json!({}),
            Ok(Tapped::Note(n)) => json!({ "note": n }),
            Ok(Tapped::Tool(t)) => json!({ "tool": t.index() }),
            Ok(Tapped::Panel) => json!({ "panel": true }),
            Err(e) => json!({ "note": e }),
        })
    }

    fn set_theme(mut self: Pin<&mut Self>, i: i32) {
        let i = usize::try_from(i).unwrap_or(usize::MAX);
        self.as_mut().rust_mut().kb.set_theme(i);
    }

    fn set_mode(mut self: Pin<&mut Self>, i: i32) {
        if let Some(&m) = usize::try_from(i).ok().and_then(|i| MODES.get(i)) {
            self.as_mut().rust_mut().kb.set_mode(m);
        }
    }

    fn set_rec(mut self: Pin<&mut Self>, on: bool) {
        self.as_mut().rust_mut().kb.set_rec(on);
    }

    fn chip(mut self: Pin<&mut Self>, i: i32) -> QString {
        let i = usize::try_from(i).unwrap_or(usize::MAX);
        let r = self.as_mut().rust_mut().kb.chip(i);
        QString::from(&r.err().unwrap_or_default())
    }

    fn flip_mode(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().kb.flip_mode();
    }

    fn step(mut self: Pin<&mut Self>, up: bool) -> bool {
        self.as_mut().rust_mut().kb.step(up)
    }

    fn preset(mut self: Pin<&mut Self>, i: i32) -> bool {
        let i = usize::try_from(i).unwrap_or(usize::MAX);
        self.as_mut().rust_mut().kb.preset(i)
    }

    fn resize_begin(mut self: Pin<&mut Self>, x: f32, y: f32) {
        self.as_mut().rust_mut().kb.resize_begin((x, y));
    }

    fn resize_at(mut self: Pin<&mut Self>, x: f32, y: f32) -> bool {
        self.as_mut().rust_mut().kb.resize_at((x, y))
    }

    fn resize_end(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().kb.resize_end();
    }

    fn relabel(mut self: Pin<&mut Self>) -> bool {
        self.as_mut().rust_mut().kb.relabel()
    }

    fn cursor(&self) -> QString {
        js(&screen::cursor().map(|p| [p.x, p.y]).ok())
    }
}
