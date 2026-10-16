//! G6 scroll: which route scrolls Notepad, Chrome, Explorer and Word, each way, while the pointer rests on the face.
//! It polls the pointer for the last point outside a stand-in face until P5 brings the product's tracker.
#![cfg(windows)]

use kx_platform_windows::screen;
use serde_json::{Value, json};
use spike_core::pt::Pt;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::IUIAutomationScrollPattern;

use crate::apps::{AppKind, Ctx};
use crate::g6track::{self, Track};
use crate::out::say;
use crate::probe::{self, Doc};
use crate::scroll::{self, Dir, Route};
use crate::simuser::guard;
use crate::win::{self, sleep_ms};
use crate::{mouse, probetext};
use spike_core::uia::{self, Uia};

/// The apps G6 probes.
pub const APPS: [AppKind; 4] = [
    AppKind::Notepad,
    AppKind::Chrome,
    AppKind::Explorer,
    AppKind::Word,
];

/// One scroll try.
struct Try {
    route: Route,
    dir: Dir,
    moved: bool,
    share: Option<f64>,
    percent: [Option<(f64, f64)>; 2],
    error: Option<String>,
}

/// Where G6 scrolls: the last point outside the face, the rest point on the face, what UI Automation can scroll there,
/// if anything, and the point as an offset from the document window's top-left.
struct Aim {
    at: Pt,
    rest: Pt,
    sp: Option<IUIAutomationScrollPattern>,
    offset: [i32; 2],
}

/// Refuses a scroll point whose root window `root` is not the document's window `doc`; `there` names that window.
fn on_document(root: HWND, doc: HWND, there: impl FnOnce(HWND) -> String) -> Result<(), String> {
    if root == doc {
        Ok(())
    } else {
        Err(format!(
            "the scroll point is not on the document: {} is there",
            there(root)
        ))
    }
}

/// The routes G6 tries: all of them, or all but UI Automation's when nothing there has a scroll pattern.
fn routes(scrolls: bool) -> Vec<Route> {
    Route::ALL
        .into_iter()
        .filter(|r| scrolls || *r != Route::Uia)
        .collect()
}

/// Glides from the document's centre to rest on the stand-in face, reading the pointer after each step; the last point
/// outside the face must be on the document.
fn target(ctx: &Ctx, ui: &Uia, doc: &Doc) -> Result<Aim, String> {
    let r = win::rect(doc.app.hwnd)?;
    let p = uia::centre(&r);
    guard(&doc.app, &[p])?;
    let work = screen::work_area(&r).map_err(|e| e.to_string())?;
    let f = g6track::face(&r, &work, &ctx.cfg.g6);
    let mut track = Track::new(f);
    for at in g6track::glide(p, &f, ctx.cfg.probes.move_steps) {
        mouse::move_to(POINT { x: at.x, y: at.y })?;
        sleep_ms(ctx.cfg.sim.step_ms);
        let now = screen::cursor().map_err(|e| e.to_string())?;
        track.see(Pt { x: now.x, y: now.y });
    }
    sleep_ms(ctx.cfg.probes.settle_ms);
    let at = track
        .last()
        .ok_or("the pointer never left the stand-in face: the document window is too short")?;
    let root = win::root_at(POINT { x: at.x, y: at.y });
    on_document(root, doc.app.hwnd, win::describe)?;
    let sp = ui.scroller(at);
    let (rest, offset) = (uia::centre(&f), g6track::offset(&r, at));
    Ok(Aim {
        at,
        rest,
        sp,
        offset,
    })
}

/// Scrolls once and compares the window's picture and UIA percent before and after.
fn try_one(
    ctx: &Ctx,
    doc: &Doc,
    (route, dir): (Route, Dir),
    at: (Pt, Pt),
    sp: Option<&IUIAutomationScrollPattern>,
) -> Result<Try, String> {
    let (g, hwnd) = (&ctx.cfg.g6, doc.app.hwnd);
    guard(&doc.app, &[at.0])?;
    let (pic, pct) = (
        scroll::picture(hwnd, ctx.spike.tools.shot_cap())?,
        sp.and_then(uia::percent),
    );
    let done = scroll::act(hwnd, route, dir, at, sp, g);
    sleep_ms(ctx.cfg.probes.settle_ms);
    let (pic2, pct2) = (
        scroll::picture(hwnd, ctx.spike.tools.shot_cap())?,
        sp.and_then(uia::percent),
    );
    let share = scroll::changed_share(&pic, &pic2);
    let seen = share.is_some_and(|s| s > g.min_changed) || scroll::percent_moved(dir, pct, pct2);
    let error = match (done, share) {
        (Err(e), _) => Some(e),
        (Ok(()), None) => Some("the window moved or changed size".to_string()),
        _ => None,
    };
    Ok(Try {
        route,
        dir,
        moved: error.is_none() && seen,
        share,
        percent: [pct, pct2],
        error,
    })
}

/// A still window's picture change, so `min_changed` can be read against it.
fn noise(ctx: &Ctx, doc: &Doc) -> Result<Option<f64>, String> {
    let before = scroll::picture(doc.app.hwnd, ctx.spike.tools.shot_cap())?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    Ok(scroll::changed_share(
        &before,
        &scroll::picture(doc.app.hwnd, ctx.spike.tools.shot_cap())?,
    ))
}

/// What one G6 run gave: every try, the target's offset in the window, whether it had a scroll pattern, and a still
/// window's picture change.
struct Driven {
    tries: Vec<Try>,
    offset: [i32; 2],
    pattern: bool,
    still: Option<f64>,
}

/// Every route in every direction at the target, after the noise baseline; UI Automation's only when it can scroll
/// there.
fn drive(ctx: &Ctx, ui: &Uia, doc: &Doc) -> Result<Driven, String> {
    probe::front(ctx, &doc.app)?;
    let aim = target(ctx, ui, doc)?;
    let still = noise(ctx, doc)?;
    let mut tries = Vec::new();
    for route in routes(aim.sp.is_some()) {
        for dir in Dir::ALL {
            let at = (aim.at, aim.rest);
            tries.push(try_one(ctx, doc, (route, dir), at, aim.sp.as_ref())?);
        }
    }
    Ok(Driven {
        tries,
        offset: aim.offset,
        pattern: aim.sp.is_some(),
        still,
    })
}

/// The directions each route scrolled; passes when some route scrolled both down and up.
fn summary(tries: &[Try]) -> (bool, Value) {
    let mut out = serde_json::Map::new();
    let mut pass = false;
    for route in Route::ALL {
        let dirs: Vec<&str> = tries
            .iter()
            .filter(|t| t.route == route && t.moved)
            .map(|t| t.dir.name())
            .collect();
        pass |= [Dir::Down, Dir::Up]
            .iter()
            .all(|d| dirs.contains(&d.name()));
        out.insert(route.name().to_string(), json!(dirs));
    }
    (pass, Value::Object(out))
}

/// Runs G6 on the app called `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let pr = &ctx.cfg.probes;
    let doc = match kind {
        AppKind::Explorer => {
            let files: Vec<String> = (1..=pr.scroll_files)
                .map(|i| format!("{}{i:04}", pr.scroll_prefix))
                .collect();
            probe::open_folder(ctx, &files, &[])?
        }
        _ => probe::open_text(
            ctx,
            kind,
            &probetext::scroll_text(pr.scroll_lines, pr.scroll_cols),
        )?,
    };
    say!("G6 {name}: {}", win::describe(doc.app.hwnd));
    let (driven, notes) = probe::run_on(ctx, doc, |d| drive(ctx, &uia, d))?;
    let (pass, worked) = summary(&driven.tries);
    let (offset, pattern, still) = (driven.offset, driven.pattern, driven.still);
    let lines: Vec<Value> = driven
        .tries
        .iter()
        .map(|t| {
            json!({ "route": t.route.name(), "dir": t.dir.name(), "moved": t.moved,
            "share": t.share, "percent": t.percent, "error": t.error })
        })
        .collect();
    say!("G6 {name}: pass {pass}; worked {worked}; noise {still:?}");
    Ok(json!({
        "gate": "G6", "app": name, "pass": pass, "worked": worked, "target_offset": offset,
        "scroll_pattern": pattern,
        "noise": still, "tries": lines, "clean_up": notes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(route: Route, dir: Dir, moved: bool) -> Try {
        Try {
            route,
            dir,
            moved,
            share: None,
            percent: [None, None],
            error: None,
        }
    }

    #[test]
    fn a_route_must_scroll_down_and_up_to_pass() {
        let one_way = [
            t(Route::Post, Dir::Down, true),
            t(Route::Send, Dir::Up, true),
        ];
        let (pass, worked) = summary(&one_way);
        assert!(!pass);
        assert_eq!(worked["post"], json!(["down"]));
        let both = [t(Route::Uia, Dir::Down, true), t(Route::Uia, Dir::Up, true)];
        assert!(summary(&both).0);
    }

    #[test]
    fn a_point_on_the_document_with_no_scroll_pattern_still_tries_post_and_send() {
        assert_eq!(routes(true), Route::ALL);
        assert_eq!(routes(false), [Route::Post, Route::Send]);
    }

    #[test]
    fn a_point_off_the_document_fails_naming_what_is_there() {
        let (doc, other) = (
            HWND(std::ptr::without_provenance_mut(1)),
            HWND(std::ptr::without_provenance_mut(2)),
        );
        assert_eq!(on_document(doc, doc, |_| String::new()), Ok(()));
        let e =
            on_document(other, doc, |_| "the taskbar".to_string()).expect_err("off the document");
        assert!(
            e.contains("not on the document") && e.contains("the taskbar"),
            "{e}"
        );
    }
}
