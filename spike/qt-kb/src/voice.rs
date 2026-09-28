//! The `Voice` QObject: the Mic button and the caption bar for QML; the worker's events come back on the Qt thread.

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use serde_json::{Value, json};
use spike_core::hold::Pt;
use spike_core::legend::is_arabic;
use spike_core::voice::{self, Caption, Event, Session, Update};

/// The cxx-qt bridge that makes `Voice` a QML type.
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
        #[qproperty(QString, bar_title, READ, CONSTANT)]
        #[qproperty(f32, bar_width, READ, CONSTANT)]
        #[qproperty(f32, bar_height, READ, CONSTANT)]
        type Voice = super::VoiceRust;

        /// The Mic button: starts or stops recording in the language of the app in front.
        #[qinvokable]
        fn click(self: Pin<&mut Self>);

        /// A caption to show: JSON with `text`, `hide_ms` (null keeps it up), `rec` (red dot), `ar` (Arabic font), the physical point `at`, and any `note`.
        /// A note alone comes as JSON with only `note`.
        #[qsignal]
        fn caption(self: Pin<&mut Self>, json: QString);
    }

    impl cxx_qt::Threading for Voice {}
    impl cxx_qt::Initialize for Voice {}
}

/// Rust side of `Voice`.
pub struct VoiceRust {
    bar_title: QString,
    bar_width: f32,
    bar_height: f32,
    /// The bar's width, height and bottom gap, for placing it.
    px: [f32; 3],
    /// Voice for this face, started once QML made the object.
    session: Option<Session>,
}

impl Default for VoiceRust {
    fn default() -> Self {
        let c = &crate::bridge::config().voice.caption;
        Self {
            bar_title: QString::from(&c.title),
            bar_width: c.px[0],
            bar_height: c.px[1],
            px: c.px,
            session: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::Voice {
    /// Starts the worker at launch, so the model is loaded before the first click.
    fn initialize(mut self: Pin<&mut Self>) {
        let thread = self.qt_thread();
        let back = move |e: Event| drop(thread.queue(move |q| q.on_event(&e)));
        let args: Vec<String> = std::env::args().skip(1).collect();
        let s = Session::start(crate::bridge::config(), &args, back);
        self.as_mut().rust_mut().session = Some(s);
    }
}

/// A caption as JSON for QML, placed at `at` (physical pixels), with an optional status `note`; `ar` picks the Arabic font.
fn caption_json(c: &Caption, at: Pt, note: Option<String>) -> Value {
    let ar = is_arabic(&c.text);
    json!({ "text": c.text, "hide_ms": c.hide_ms, "rec": c.rec, "ar": ar, "at": [at.x, at.y], "note": note })
}

impl qobject::Voice {
    fn click(mut self: Pin<&mut Self>) {
        let u = self
            .as_mut()
            .rust_mut()
            .session
            .as_mut()
            .map(Session::click);
        self.show(u.unwrap_or_default());
    }

    /// Types the words into the app in front and tells QML what the caption bar shows.
    fn on_event(mut self: Pin<&mut Self>, e: &Event) {
        let u = self
            .as_mut()
            .rust_mut()
            .session
            .as_mut()
            .map(|s| s.on_event(e));
        self.show(u.unwrap_or_default());
    }

    /// Sends the caption, placed on the screen in front, or a lone note to QML.
    fn show(self: Pin<&mut Self>, u: Update) {
        let px = self.rust().px;
        let place = || voice::bar_place(px).map(|(p, _)| p);
        if let Some(v) = update_json(u, place) {
            self.caption(QString::from(&v.to_string()));
        }
    }
}

/// What QML gets for `u`: the caption placed by `place`, or just the note; `None` when there is neither.
fn update_json(u: Update, place: impl FnOnce() -> Result<Pt, String>) -> Option<Value> {
    let Some(c) = u.caption else {
        return u.note.map(|n| json!({ "note": n }));
    };
    let (at, place_err) = match place() {
        Ok(p) => (p, None),
        Err(e) => (Pt { x: 0, y: 0 }, Some(e)),
    };
    Some(caption_json(&c, at, u.note.or(place_err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captions_reach_qml_as_small_json() {
        let c = Caption {
            text: "مرحبا".into(),
            hide_ms: Some(4000),
            rec: false,
        };
        let v = caption_json(&c, Pt { x: 5, y: 7 }, None);
        assert_eq!(
            v,
            json!({ "text": "مرحبا", "hide_ms": 4000, "rec": false, "ar": true, "at": [5, 7], "note": null })
        );
        let stay = Caption {
            text: "…".into(),
            hide_ms: None,
            rec: true,
        };
        assert_eq!(caption_json(&stay, Pt { x: 0, y: 0 }, None)["ar"], false);
        assert_eq!(
            caption_json(&stay, Pt { x: 0, y: 0 }, None)["hide_ms"],
            Value::Null
        );
    }

    #[test]
    fn a_note_without_a_caption_still_reaches_qml() {
        let here = || Ok(Pt { x: 0, y: 0 });
        let note = Update {
            caption: None,
            note: Some("typing failed".into()),
        };
        assert_eq!(
            update_json(note, here),
            Some(json!({ "note": "typing failed" }))
        );
        assert_eq!(update_json(Update::default(), here), None);
    }
}
