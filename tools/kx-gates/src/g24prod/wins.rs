//! Our copy's windows during a snip, told apart by their titles in any shipped language, and what Windows says of
//! them and of target-window: kept out of captures, on top, over the frozen screens, their frame, and a box's screen.
#![cfg(windows)]

use kx_platform::ui::Screens;
use kx_platform_windows::screen::WinScreens;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromRect,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WS_EX_TOPMOST,
};

use super::judge::{Kind, Layer, Seen, layer, named, short, under};
use crate::g9win::ex_of;
use crate::p12cfg::SnipTitles;
use crate::win::{self, size32};
use crate::{ftl, product};

/// Every shipped language's title of each window G24 tells apart.
#[derive(Debug, Clone)]
pub struct Titles {
    pub frozen: Vec<String>,
    pub capture: Vec<String>,
    pub editor: Vec<String>,
    pub ask: Vec<String>,
    pub keyboard: Vec<String>,
    pub bubble: Vec<String>,
}

impl Titles {
    /// The titles of `t`'s keys in every shipped language.
    pub fn new(t: &SnipTitles) -> Result<Self, String> {
        let mut capture = Vec::new();
        for key in &t.capture {
            capture.extend(ftl::every(key)?);
        }
        Ok(Self {
            frozen: ftl::every(&t.frozen)?,
            capture,
            editor: ftl::every(&t.editor)?,
            ask: ftl::every(&t.ask)?,
            keyboard: ftl::every(&t.keyboard)?,
            bubble: ftl::every(&t.bubble)?,
        })
    }

    /// What a window titled `title` is.
    pub fn kind(&self, title: &str) -> Kind {
        let has = |set: &[String]| set.iter().any(|t| t == title);
        if has(&self.frozen) {
            Kind::Frozen
        } else if has(&self.capture) {
            Kind::Capture
        } else if has(&self.keyboard) {
            Kind::Keyboard
        } else if has(&self.bubble) {
            Kind::Bubble
        } else {
            Kind::Other
        }
    }
}

/// True when window `w` is kept out of screen captures.
#[allow(unsafe_code, reason = "A plain query of a window's capture affinity.")]
pub fn shielded(w: HWND) -> bool {
    let mut got = 0;
    // SAFETY: a plain query into a local; a gone window gives an error.
    let read = unsafe { GetWindowDisplayAffinity(w, &raw mut got) };
    read.is_ok() && got == WDA_EXCLUDEFROMCAPTURE.0
}

/// True when window `w` stays on top of other windows.
pub fn topmost(w: HWND) -> bool {
    ex_of(w) & WS_EX_TOPMOST.0 != 0
}

/// Window `w`'s frame without its shadow, in physical pixels.
#[allow(unsafe_code, reason = "A DWM query into a local.")]
pub fn frame(w: HWND) -> Result<RECT, String> {
    let mut r = RECT::default();
    // SAFETY: the call writes one RECT into a local that outlives it.
    unsafe {
        DwmGetWindowAttribute(
            w,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            (&raw mut r).cast(),
            size32::<RECT>(),
        )
    }
    .map_err(|e| format!("the frame of {}: {e}", win::describe(w)))?;
    Ok(r)
}

/// The whole screen nearest to box `r`, taskbar included.
#[allow(unsafe_code, reason = "A monitor query into a local.")]
pub fn screen(r: &RECT) -> Result<RECT, String> {
    let mut info = MONITORINFO {
        cbSize: size32::<MONITORINFO>(),
        ..Default::default()
    };
    // SAFETY: plain queries; `info` is a local with `cbSize` set, and the nearest monitor always exists.
    let ok =
        unsafe { GetMonitorInfoW(MonitorFromRect(r, MONITOR_DEFAULTTONEAREST), &raw mut info) };
    ok.as_bool()
        .then_some(info.rcMonitor)
        .ok_or_else(|| "Windows did not describe the screen".to_string())
}

/// The centre of box `r`.
pub fn centre(r: &RECT) -> POINT {
    POINT {
        x: r.left + (r.right - r.left) / 2,
        y: r.top + (r.bottom - r.top) / 2,
    }
}

/// True when boxes `a` and `b` share a pixel.
pub fn meet(a: &RECT, b: &RECT) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

/// The shown windows of copy `pid`, topmost first, with what each is.
pub fn ours(pid: u32, titles: &Titles) -> Vec<(HWND, Kind)> {
    let kind = |w: HWND| titles.kind(&win::title(w));
    product::shown_by(pid)
        .into_iter()
        .map(|w| (w, kind(w)))
        .collect()
}

/// The shown windows of copy `pid` as a watch judges them.
pub fn seen(pid: u32, titles: &Titles) -> Vec<Seen> {
    let seen = |(w, kind)| Seen {
        kind,
        shielded: shielded(w),
    };
    ours(pid, titles).into_iter().map(seen).collect()
}

/// Every window a person sees now, top first, as the order check weighs it: copy `pid`'s frozen screens, the windows
/// `above` names, and the rest.
pub fn layers(pid: u32, titles: &Titles, above: impl Fn(HWND) -> bool) -> Vec<(HWND, Layer)> {
    let frozen = |w: HWND| win::owner(w).0 == pid && titles.kind(&win::title(w)) == Kind::Frozen;
    win::seen_windows()
        .into_iter()
        .map(|w| (w, layer(frozen(w), above(w))))
        .collect()
}

/// How many screens Windows shows now, each of which a snip freezes.
pub fn screens() -> usize {
    WinScreens.work_areas().len()
}

/// What is wrong with `order` on `screens` screens: too few frozen screens, then which window lay under which, with
/// their places; ours by title, others by class only.
pub fn faults(pid: u32, order: &[(HWND, Layer)], screens: usize) -> Vec<String> {
    let name = |w: HWND| named(win::owner(w).0 == pid, &win::title(w), &win::class(w));
    let layers: Vec<Layer> = order.iter().map(|&(_, l)| l).collect();
    let line = |(i, top): (usize, usize)| {
        let (low, high) = (name(order[i].0), name(order[top].0));
        format!("{low} at {i} lies under the frozen screen {high} at {top}")
    };
    let few = short(&layers, screens)
        .map(|n| format!("only {n} of {} frozen screens showed", screens.max(1)));
    few.into_iter()
        .chain(under(&layers).into_iter().map(line))
        .collect()
}

/// The first shown window of copy `pid` titled one of `which`.
pub fn shown(pid: u32, which: &[String]) -> Option<HWND> {
    let named = |w: &HWND| which.iter().any(|t| *t == win::title(*w));
    product::shown_by(pid).into_iter().find(named)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_read_with_too_few_frozen_screens_is_a_fault_though_nothing_lies_under_one() {
        let frozen = (HWND::default(), Layer::Frozen);
        assert_eq!(faults(0, &[], 1), ["only 0 of 1 frozen screens showed"]);
        assert_eq!(
            faults(0, &[frozen], 2),
            ["only 1 of 2 frozen screens showed"]
        );
        assert!(faults(0, &[frozen], 1).is_empty());
    }
}
