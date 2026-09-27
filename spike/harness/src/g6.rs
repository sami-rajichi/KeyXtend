//! G6 scroll: which route scrolls Notepad, Chrome, Explorer and Word, each way, while the pointer rests on the face.

use serde_json::{Value, json};
use spike_core::hold::{Act, Pt};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::Accessibility::IUIAutomationScrollPattern;

use crate::apps::{AppKind, Ctx};
use crate::assist;
use crate::probe::{self, Doc};
use crate::scroll::{self, Dir, Route};
use crate::simuser::{self, guard};
use crate::uia::{self, Uia};
use crate::win::{self, sleep_ms};

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

/// The stand-in face: a square at the desktop's top-left corner.
fn face(size: i32) -> RECT {
    let (x, y, _, _) = win::desktop();
    RECT {
        left: x,
        top: y,
        right: x + size,
        bottom: y + size,
    }
}

/// Points at the window's centre, then rests on the face; returns the hook's last point outside it, and the rest point.
fn target(ctx: &Ctx, doc: &Doc) -> Result<(Pt, Pt), String> {
    let p = uia::centre(&win::rect(doc.app.hwnd)?);
    guard(&doc.app, &[p])?;
    let f = face(ctx.cfg.g6.face_px);
    let rest = uia::centre(&f);
    let assist = simuser::start_assist(ctx, vec![f])?;
    assist::as_user(&[Act::Move(p)])?;
    sleep_ms(ctx.cfg.sim.step_ms);
    assist::as_user(&[Act::Move(rest)])?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    let last = assist.live().last_outside;
    assist.stop()?;
    Ok((last.ok_or("the hook saw no point outside the face")?, rest))
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
    let (pic, pct) = (scroll::picture(hwnd)?, sp.and_then(uia::percent));
    let done = scroll::act(hwnd, route, dir, at, sp, g);
    sleep_ms(ctx.cfg.probes.settle_ms);
    let (pic2, pct2) = (scroll::picture(hwnd)?, sp.and_then(uia::percent));
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
    let before = scroll::picture(doc.app.hwnd)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    Ok(scroll::changed_share(
        &before,
        &scroll::picture(doc.app.hwnd)?,
    ))
}

/// Every route in every direction at the target, after the noise baseline.
fn drive(ctx: &Ctx, uia: &Uia, doc: &Doc) -> Result<(Vec<Try>, Pt, Option<f64>), String> {
    probe::front(ctx, &doc.app)?;
    let at = target(ctx, doc)?;
    let still = noise(ctx, doc)?;
    let sp = uia.scroller(at.0);
    let mut tries = Vec::new();
    for route in Route::ALL {
        for dir in Dir::ALL {
            tries.push(try_one(ctx, doc, (route, dir), at, sp.as_ref())?);
        }
    }
    Ok((tries, at.0, still))
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
            &probe::scroll_text(pr.scroll_lines, pr.scroll_cols),
        )?,
    };
    println!("G6 {name}: {}", win::describe(doc.app.hwnd));
    let ((tries, at, still), notes) = probe::run_on(ctx, doc, |d| drive(ctx, &uia, d))?;
    let (pass, worked) = summary(&tries);
    let lines: Vec<Value> = tries
        .iter()
        .map(|t| {
            json!({ "route": t.route.name(), "dir": t.dir.name(), "moved": t.moved,
            "share": t.share, "percent": t.percent, "error": t.error })
        })
        .collect();
    println!("G6 {name}: pass {pass}; worked {worked}; noise {still:?}");
    Ok(json!({
        "gate": "G6", "app": name, "pass": pass, "worked": worked, "target": [at.x, at.y],
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
}
