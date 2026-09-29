//! `spike.toml [panel]`: the Arabic test panel for gate G7, a stand-in for the Settings window.

use serde::{Deserialize, Serialize};

use crate::config::SpikeConfig;
use crate::facecfg::BarButton;
use crate::legend::is_arabic;

/// Where the page text puts this page's number.
const PAGE_AT: &str = "{n}";
/// Where the page text puts the number of pages.
const PAGE_COUNT: &str = "{count}";
/// The most rows a list page may have (ADR-0008).
const MOST_ROWS: usize = 5;

/// The panel's texts, icons and list.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PanelConfig {
    /// Header and window title; an Arabic title makes the panel right to left.
    pub title: String,
    /// The header's icon.
    pub icon: String,
    /// The list's rows.
    pub items: Vec<String>,
    /// Rows per page (ADR-0008).
    pub rows: usize,
    /// Page text with `{n}` and `{count}`.
    pub page: String,
    /// The text field's name for screen readers.
    pub field: String,
    /// Shown in the empty text field.
    pub hint: String,
    /// The close, previous-page and next-page buttons.
    pub close: BarButton,
    /// See `close`.
    pub prev: BarButton,
    /// See `close`.
    pub next: BarButton,
}

impl PanelConfig {
    /// Refuses pages with no rows or too many, and a page text without its two numbers.
    pub fn check(&self) -> Result<(), String> {
        if !(1..=MOST_ROWS).contains(&self.rows) {
            return Err(format!("panel: rows must be 1 to {MOST_ROWS}"));
        }
        if !(self.page.contains(PAGE_AT) && self.page.contains(PAGE_COUNT)) {
            return Err(format!("panel: page must hold {PAGE_AT} and {PAGE_COUNT}"));
        }
        Ok(())
    }

    /// The panel reads right to left, as its title is Arabic.
    pub fn rtl(&self) -> bool {
        is_arabic(&self.title)
    }

    /// The page text for page `at` (from 0) of `count`.
    pub fn page_label(&self, at: usize, count: usize) -> String {
        let n = (at + 1).to_string();
        self.page
            .replace(PAGE_AT, &n)
            .replace(PAGE_COUNT, &count.to_string())
    }
}

/// Refuses an empty or shared window title: the focus guard spares the panel, and the harness finds our windows, by title.
pub fn check_titles(cfg: &SpikeConfig) -> Result<(), String> {
    let tools = &cfg.tools;
    let named = [
        &cfg.panel.title,
        &tools.pill_title,
        &tools.overlay_title,
        &cfg.voice.caption.title,
        &cfg.bar.bubble_title,
        &cfg.ring.title,
    ];
    let all: Vec<&String> = std::iter::once(&cfg.keyboard.title).chain(named).collect();
    let shared = all.iter().enumerate().any(|(i, t)| all[..i].contains(t));
    if shared || all.iter().any(|t| t.is_empty()) {
        return Err("window titles must be set and differ from each other".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::legend::is_arabic;

    fn panel() -> super::PanelConfig {
        crate::config::load().expect("spike.toml loads").panel
    }

    #[test]
    fn the_panel_pages_seven_arabic_rows_five_at_a_time() {
        let p = panel();
        assert_eq!((p.rows, p.items.len()), (5, 7));
        assert!(
            p.items
                .iter()
                .chain([&p.title, &p.hint])
                .all(|t| is_arabic(t))
        );
    }

    #[test]
    fn its_buttons_have_icons_and_arabic_names_and_it_reads_right_to_left() {
        let p = panel();
        let buttons = [&p.close, &p.prev, &p.next];
        assert!(
            buttons
                .iter()
                .all(|b| !b.icon.is_empty() && is_arabic(&b.name))
        );
        assert!(!p.icon.is_empty() && is_arabic(&p.field));
        assert!(p.rtl());
        let latin = super::PanelConfig {
            title: "Settings".into(),
            ..panel()
        };
        assert!(!latin.rtl());
    }

    #[test]
    fn the_page_text_counts_from_one() {
        assert_eq!(panel().page_label(1, 2), "2 من 2");
    }

    #[test]
    fn a_title_that_is_empty_or_another_windows_is_refused() {
        let cfg = crate::config::load().expect("spike.toml loads");
        assert_eq!(super::check_titles(&cfg), Ok(()));
        let with = |f: &dyn Fn(&mut crate::config::SpikeConfig)| {
            let mut c = cfg.clone();
            f(&mut c);
            super::check_titles(&c)
        };
        let pill = with(&|c| c.panel.title = c.tools.pill_title.clone());
        assert!(pill.expect_err("pill").contains("title"));
        assert!(
            with(&|c| c.panel.title = c.keyboard.title.clone()).is_err(),
            "the keyboard's"
        );
        assert!(
            with(&|c| c.panel.title = c.ring.title.clone()).is_err(),
            "the ring's"
        );
        assert!(
            with(&|c| c.ring.title = c.bar.bubble_title.clone()).is_err(),
            "ring and bubble"
        );
        assert!(with(&|c| c.ring.title.clear()).is_err(), "untitled windows");
    }

    #[test]
    fn no_rows_too_many_rows_or_a_page_text_without_its_numbers_is_refused() {
        let empty = super::PanelConfig { rows: 0, ..panel() };
        assert!(empty.check().expect_err("no rows").contains("rows"));
        let long = super::PanelConfig { rows: 6, ..panel() };
        assert!(long.check().expect_err("six rows").contains("rows"));
        let bare = super::PanelConfig {
            page: "page".into(),
            ..panel()
        };
        assert!(bare.check().expect_err("no numbers").contains("page"));
    }
}
