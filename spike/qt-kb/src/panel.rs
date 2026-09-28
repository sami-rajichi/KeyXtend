//! The `PanelData` QObject: the Arabic test panel's texts, its pages, where it goes, and the focus it borrows (gate G7).

use std::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use serde_json::{Value, json};
use spike_core::panelcfg::PanelConfig;
use spike_core::{pager, screen, window};
use windows::Win32::Foundation::HWND;

use crate::bridge::{FACE, config, spot, themes};

/// The cxx-qt bridge that makes `PanelData` a QML type.
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
        type PanelData = super::PanelRust;

        /// The panel's texts, icons, rows per page and direction, as JSON.
        #[qinvokable]
        fn fixed_json(&self) -> QString;

        /// Page `at` of the list as JSON: its `rows` (null pads the last page), `at`, `count` and `label`.
        #[qinvokable]
        fn page_json(&self, at: i32) -> QString;

        /// Where a `w` by `h` panel goes by the keyboard, whose plate is `plate_w` wide (Qt units):
        /// JSON `{x, y}` in physical pixels, or `{note}`.
        #[qinvokable]
        fn spot(&self, plate_w: f64, w: f64, h: f64) -> QString;

        /// Notes the app in front before the panel takes focus.
        #[qinvokable]
        fn opened(self: Pin<&mut Self>);

        /// Gives focus back to that app; call it while the panel is still shown.
        #[qinvokable]
        fn closed(&self);
    }
}

/// Rust side of `PanelData`.
pub struct PanelRust {
    cfg: PanelConfig,
    /// The app in front when the panel opened.
    prev: HWND,
}

impl Default for PanelRust {
    /// Also spares the panel from the focus guard, as it stands in for Settings, the one window that takes focus.
    fn default() -> Self {
        let cfg = config().panel.clone();
        window::spare(&cfg.title);
        Self {
            cfg,
            prev: HWND::default(),
        }
    }
}

/// The fixed part for QML: the settings block plus its direction.
fn fixed(p: &PanelConfig) -> Value {
    let mut v = json!(p);
    v["rtl"] = json!(p.rtl());
    v
}

/// Page `at` for QML; a negative page is the first.
fn page_of(p: &PanelConfig, at: i32) -> Value {
    let pg = pager::page(&p.items, usize::try_from(at).unwrap_or(0), p.rows);
    let label = p.page_label(pg.at, pg.count);
    json!({ "rows": pg.rows, "at": pg.at, "count": pg.count, "label": label })
}

impl qobject::PanelData {
    fn fixed_json(&self) -> QString {
        QString::from(&fixed(&self.rust().cfg).to_string())
    }

    fn page_json(&self, at: i32) -> QString {
        QString::from(&page_of(&self.rust().cfg, at).to_string())
    }

    fn spot(&self, plate_w: f64, w: f64, h: f64) -> QString {
        let size = [w as f32, h as f32];
        let gaps = themes().shape.panel.gap_px;
        let title = config().title(FACE);
        let rtl = self.rust().cfg.rtl();
        spot(screen::pop_for(&title, plate_w as f32, size, gaps, rtl))
    }

    fn opened(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().prev = window::foreground();
    }

    fn closed(&self) {
        window::give_back(self.rust().prev);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> PanelConfig {
        spike_core::config::load().expect("spike.toml loads").panel
    }

    #[test]
    fn the_second_page_holds_the_last_two_rows_then_padding() {
        let p = panel();
        let v = page_of(&p, 1);
        let rows = v["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), p.rows);
        assert_eq!(rows[1], json!(p.items[6]));
        assert!(rows[2..].iter().all(Value::is_null));
        assert_eq!(v["label"], json!(p.page_label(1, 2)));
    }

    #[test]
    fn a_page_before_the_first_is_the_first() {
        assert_eq!(page_of(&panel(), -3)["at"], json!(0));
    }

    #[test]
    fn qml_learns_the_panel_reads_right_to_left() {
        let v = fixed(&panel());
        assert_eq!(v["rtl"], json!(true));
        assert_eq!(v["close"]["icon"], json!(panel().close.icon));
    }
}
