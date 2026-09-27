//! Watches the focused text of other apps on its own thread and says where the selection pill goes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows::Win32::Foundation::RECT;
use windows::Win32::System::Threading::GetCurrentProcessId;

use crate::hold::Pt;
use crate::screen;
use crate::uia::{self, Pill, Uia};

/// Characters read to tell a selection from a bare caret; the text itself is dropped at once.
const PEEK_CHARS: usize = 1;

/// The pill's width, height and gap from the text, in logical pixels.
#[derive(Clone, Copy, Debug)]
pub struct PillPx(pub [f32; 3]);

impl PillPx {
    /// The same size in physical pixels at `dpi`.
    pub fn at(self, dpi: u32) -> Pill {
        let [w, h, gap] = self.0.map(|v| screen::physical(v, dpi));
        Pill { w, h, gap }
    }
}

/// True when the pill must hide: on our own windows and on password boxes.
fn hides(ours: bool, password: bool) -> bool {
    ours || password
}

/// Where the pill goes next to the selection `boxes`, or `None` when there is no room or no selection.
pub fn place(boxes: &[RECT], screen: &RECT, pill: Pill) -> Option<Pt> {
    uia::anchor(boxes, screen, pill)
}

/// What the face does with its pill on one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PillStep {
    /// Show the hidden pill at this point.
    Show(Pt),
    /// Move the shown pill here.
    Move(Pt),
    /// Hide the pill.
    Hide,
    /// Leave it as it is.
    Stay,
}

/// The step from where the pill is (`before`) to where the watcher puts it (`now`).
pub fn step(before: Option<Pt>, now: Option<Pt>) -> PillStep {
    match (before, now) {
        (None, Some(p)) => PillStep::Show(p),
        (Some(b), Some(p)) if b != p => PillStep::Move(p),
        (Some(_), None) => PillStep::Hide,
        _ => PillStep::Stay,
    }
}

/// A running watcher; it stops once dropped.
pub struct Watch {
    spot: Arc<Mutex<Option<Pt>>>,
    stop: Arc<AtomicBool>,
}

impl Watch {
    /// Starts looking every `poll_ms`; fails when UI Automation cannot start.
    pub fn start(poll_ms: u64, pill: PillPx) -> Result<Watch, String> {
        let spot = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let (ready, started) = sync_channel(1);
        let (out, halt) = (Arc::clone(&spot), Arc::clone(&stop));
        std::thread::spawn(move || {
            let uia = match Uia::new() {
                Ok(u) => u,
                Err(e) => return drop(ready.send(Err(e))),
            };
            let _ = ready.send(Ok(()));
            // SAFETY: a plain query.
            let me = unsafe { GetCurrentProcessId() };
            while !halt.load(Ordering::Relaxed) {
                let at = look(&uia, me, pill);
                if let Ok(mut s) = out.lock() {
                    *s = at;
                }
                std::thread::sleep(Duration::from_millis(poll_ms));
            }
        });
        let started = started
            .recv()
            .map_err(|_| "the selection watcher died".to_string());
        started??;
        Ok(Watch { spot, stop })
    }

    /// Where the pill goes now, in physical pixels, or `None` when it hides.
    pub fn spot(&self) -> Option<Pt> {
        self.spot.lock().ok().and_then(|s| *s)
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        // The thread ends after its current look; a hung app may hold it, so we do not wait.
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// One look at the focused element; a password box is recognised before any text is read.
fn look(uia: &Uia, me: u32, pill: PillPx) -> Option<Pt> {
    let el = uia.focused().ok()?;
    if hides(uia::pid(&el) == me, uia::is_password(&el)) {
        return None;
    }
    let sel = uia::selection(&uia.text_of(&el)?, PEEK_CHARS).ok()??;
    let first = *sel.boxes.first()?;
    let screen = screen::work_area(&first).ok()?;
    place(&sel.boxes, &screen, pill.at(screen::dpi_of(&first)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn the_pill_sits_by_the_selection_and_hides_without_one() {
        let (line, screen) = ([rect(100, 100, 300, 120)], rect(0, 0, 1920, 1080));
        let pill = PillPx([72.0, 32.0, 8.0]).at(96);
        assert_eq!(place(&line, &screen, pill), Some(Pt { x: 164, y: 128 }));
        assert_eq!(place(&[], &screen, pill), None);
    }

    #[test]
    fn the_pill_hides_on_our_windows_and_password_boxes() {
        assert!(!hides(false, false));
        assert!(hides(true, false));
        assert!(hides(false, true));
    }

    #[test]
    fn the_pill_shows_moves_hides_or_stays() {
        let (a, b) = (Pt { x: 1, y: 2 }, Pt { x: 3, y: 4 });
        assert_eq!(step(None, Some(a)), PillStep::Show(a));
        assert_eq!(step(Some(a), Some(b)), PillStep::Move(b));
        assert_eq!(step(Some(a), None), PillStep::Hide);
        assert_eq!(step(Some(a), Some(a)), PillStep::Stay);
        assert_eq!(step(None, None), PillStep::Stay);
    }

    #[test]
    fn the_pill_size_follows_the_monitor_dpi() {
        let p = PillPx([72.0, 32.0, 8.0]).at(120);
        assert_eq!((p.w, p.h, p.gap), (90, 40, 10));
    }
}
