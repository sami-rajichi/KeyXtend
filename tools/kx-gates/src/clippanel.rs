//! The product's clipboard panel as a screen reader meets it: its window, its rows by gate id in place order, its
//! search line and its buttons' labels, and a real click on a part only while the panel is on top there.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    TreeScope_Descendants, UIA_CONTROLTYPE_ID, UIA_ControlTypePropertyId,
    UIA_ListItemControlTypeId, UIA_TextControlTypeId,
};

use crate::apps::Ctx;
use crate::keymap::{KeyRead, Reader, centre, read_one};
use crate::p8cfg::G20Product;
use crate::{mouse, product, win};

/// True when box `r` has an area on screen.
fn has_area(r: &RECT) -> bool {
    r.right > r.left && r.bottom > r.top
}

/// The names of `items` whose id is `prefix` and a place, in place order; one without area on screen is left out.
pub fn placed(items: &[KeyRead], prefix: &str) -> Vec<String> {
    let place = |k: &KeyRead| k.id.strip_prefix(prefix)?.parse::<usize>().ok();
    let mut rows: Vec<(usize, &str)> = items
        .iter()
        .filter(|k| has_area(&k.rect))
        .filter_map(|k| place(k).map(|p| (p, k.name.as_str())))
        .collect();
    rows.sort_by_key(|(p, _)| *p);
    rows.into_iter().map(|(_, name)| name.to_string()).collect()
}

/// A reader of the product's clipboard panel, by the gate ids `[g20_product]` names.
pub struct Panel<'a> {
    ctx: &'a Ctx<'a>,
    reader: Reader,
    /// Longest wait for the panel to open or close, from the gate's own settings.
    wait_ms: u64,
}

impl<'a> Panel<'a> {
    /// A reader for the panel of the product that runs now, which waits up to `wait_ms` for it to open or close.
    pub fn new(ctx: &'a Ctx<'a>, wait_ms: u64) -> Result<Self, String> {
        let reader = Reader::new()?;
        Ok(Self {
            ctx,
            reader,
            wait_ms,
        })
    }

    /// The panel's ids.
    fn g(&self) -> &G20Product {
        &self.ctx.cfg.g20_product
    }

    /// The shown product window whose list has an area on screen; none while the panel is closed.
    pub fn window(&self) -> Option<HWND> {
        let list = self.g().id(&self.g().list);
        let open = |w: &HWND| self.reader.rect(*w, &list).is_ok_and(|r| has_area(&r));
        product::shown().into_iter().find(open)
    }

    /// The panel's window, or an error that says it is closed.
    fn open(&self) -> Result<HWND, String> {
        self.window()
            .ok_or_else(|| "the clipboard panel is not open".to_string())
    }

    /// Waits up to the panel's wait for it to be `open`, or closed.
    pub fn wait(&self, open: bool) -> Result<(), String> {
        let wait = self.wait_ms;
        let now = || (self.window().is_some() == open).then_some(());
        let what = if open { "open" } else { "close" };
        win::poll_until(wait, self.ctx.cfg.timing.poll_ms, now)
            .ok_or_else(|| format!("the clipboard panel did not {what} within {wait} ms"))
    }

    /// Every element of control type `kind` in the open panel, as a screen reader meets it.
    #[allow(
        unsafe_code,
        reason = "UI Automation calls on objects this client made."
    )]
    fn all(&self, kind: UIA_CONTROLTYPE_ID) -> Result<Vec<KeyRead>, String> {
        let (hwnd, auto) = (self.open()?, self.reader.auto());
        let err = |e: windows::core::Error| format!("UI Automation: {e}");
        let mut items = Vec::new();
        // SAFETY: plain calls on live UI Automation objects; a dead window gives an error.
        unsafe {
            let cond = auto
                .CreatePropertyCondition(UIA_ControlTypePropertyId, &VARIANT::from(kind.0))
                .map_err(err)?;
            let found = auto
                .ElementFromHandle(hwnd)
                .map_err(err)?
                .FindAll(TreeScope_Descendants, &cond)
                .map_err(err)?;
            for i in 0..found.Length().map_err(err)? {
                items.push(read_one(&found.GetElement(i).map_err(err)?)?);
            }
        }
        Ok(items)
    }

    /// The rows' accessible names on the page shown, in place order.
    pub fn rows(&self) -> Result<Vec<String>, String> {
        Ok(placed(
            &self.all(UIA_ListItemControlTypeId)?,
            &self.g().id(&self.g().row),
        ))
    }

    /// The names of the panel's texts on screen, such as the pager's page number.
    pub fn texts(&self) -> Result<Vec<String>, String> {
        let shown = self
            .all(UIA_TextControlTypeId)?
            .into_iter()
            .filter(|k| has_area(&k.rect));
        Ok(shown.map(|k| k.name).collect())
    }

    /// The search line's text, which its description holds.
    pub fn search(&self) -> Result<String, String> {
        self.reader
            .described(self.open()?, &self.g().id(&self.g().search_line))
    }

    /// The accessible name of the panel's button `part`, such as Insert with its count.
    pub fn label(&self, part: &str) -> Result<String, String> {
        let id = self.g().id(part);
        let found = self
            .reader
            .walk(self.open()?)?
            .into_iter()
            .find(|k| k.id == id);
        found
            .map(|k| k.name)
            .ok_or_else(|| format!("no {id} on the clipboard panel"))
    }

    /// Clicks the panel's part `id` with the real pointer, refused unless the panel is the window on top there.
    fn click_id(&self, id: &str) -> Result<(), String> {
        let hwnd = self.open()?;
        let r = self.reader.rect(hwnd, id)?;
        let at = centre(&r);
        if !has_area(&r) || win::root_at(at) != hwnd {
            return Err(format!(
                "{id} is covered or off screen, so a click would miss it"
            ));
        }
        mouse::click_at(at)
    }

    /// Clicks the panel's button or line `part`.
    pub fn click(&self, part: &str) -> Result<(), String> {
        self.click_id(&self.g().id(part))
    }

    /// Clicks the row at `place` on the page shown, which ticks it.
    pub fn click_row(&self, place: usize) -> Result<(), String> {
        self.click_id(&self.g().row_id(place))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(id: &str, name: &str, wide: i32) -> KeyRead {
        let rect = RECT {
            left: 0,
            top: 0,
            right: wide,
            bottom: 10,
        };
        KeyRead {
            id: id.into(),
            name: name.into(),
            rect,
            invokable: false,
        }
    }

    #[test]
    fn rows_come_in_place_order_and_only_with_an_area() {
        let items = [
            read("clip.row-2", "third", 10),
            read("clip.row-0", "first", 10),
            read("clip.list", "History", 10),
            read("clip.row-1", "second", 10),
            read("clip.row-3", "hidden", 0),
            read("clip.row-x", "no place", 10),
            read("", "padding", 10),
        ];
        assert_eq!(placed(&items, "clip.row-"), ["first", "second", "third"]);
        assert!(placed(&items, "words.row-").is_empty());
    }

    #[test]
    fn rows_ten_and_up_follow_nine() {
        let items = [
            read("clip.row-10", "ten", 10),
            read("clip.row-9", "nine", 10),
        ];
        assert_eq!(placed(&items, "clip.row-"), ["nine", "ten"]);
    }
}
