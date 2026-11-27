//! G24's pictures by real clicks and Copy, judged against the gate's own grab just before: the freeze time, a region
//! exactly, a window's frame, the keyboard and bubble out of a full screen, and the editor's Copy at its part's size.
#![cfg(windows)]

use serde_json::{Value, json};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::HSTRING;

use super::judge::{self, BLANK, BubbleSeen, at_scale};
use super::pic::{Pic, differing, middle};
use super::scene::Scene;
use super::snip::Snip;
use super::watch::{Watch, Watched};
use super::{render, wins};
use crate::shot;
use crate::win::{self, ms_since};

/// What a Snip press gave: how soon the frozen screen covered, and whether the keyboard still showed when it came.
pub struct Froze {
    pub covered_ms: Option<u64>,
    pub keyboard: bool,
}

/// A picture snip: its press, what the watch saw until Copy, the picture, and what `during` read.
struct Taken<T> {
    froze: Froze,
    watched: Watched,
    pic: Pic,
    during: T,
}

/// The gate's own grab of `screen`.
fn grab(screen: &RECT) -> Result<Pic, String> {
    Ok(Pic::from_shot(&shot::of_box(screen)?))
}

/// Box `r`'s size.
fn size(r: &RECT) -> (i32, i32) {
    (r.right - r.left, r.bottom - r.top)
}

/// The two corner pixels that pick box `r`.
pub fn corners(r: &RECT) -> [POINT; 2] {
    [
        POINT {
            x: r.left,
            y: r.top,
        },
        POINT {
            x: r.right - 1,
            y: r.bottom - 1,
        },
    ]
}

/// Target-window's centre and the taskbar's, which the frozen screen must both cover.
#[allow(unsafe_code, reason = "A plain lookup by class name.")]
fn cover_points(s: &Snip, frame: &RECT) -> Result<[POINT; 2], String> {
    let class = &s.g.taskbar_class;
    // SAFETY: a plain lookup by class name; the string lives for the call.
    let bar = unsafe { FindWindowW(&HSTRING::from(class), None) }
        .map_err(|e| format!("no taskbar {class}: {e}"))?;
    Ok([wins::centre(frame), wins::centre(&win::rect(bar)?)])
}

/// Presses Snip: how soon the frozen screen covered target-window and the taskbar, none past `[g24] wait_ms`, and
/// whether the keyboard showed as the frozen screen first did.
pub fn freeze(s: &Snip, frame: &RECT) -> Result<Froze, String> {
    let points = cover_points(s, frame)?;
    let t0 = s.press()?;
    let mut first = None;
    let covered_ms = s.until(|| {
        let f = s.frozen()?;
        first.get_or_insert_with(|| wins::shown(s.pid, &s.titles.keyboard).is_some());
        points
            .iter()
            .all(|&p| win::root_at(p) == f)
            .then(|| ms_since(t0))
    });
    Ok(Froze {
        covered_ms,
        keyboard: first.unwrap_or_default(),
    })
}

/// Presses Snip, refused unless the frozen screen covered.
pub fn covered(s: &Snip, frame: &RECT) -> Result<(), String> {
    freeze(s, frame)?
        .covered_ms
        .map(drop)
        .ok_or_else(|| "the frozen screen did not cover".into())
}

/// Snip, way `way`, clicks at `spots`, then `during` once the strip shows, then Copy; the watch runs until Copy.
fn taken<T>(
    s: &Snip,
    sc: &Scene,
    (way, spots): (&str, &[POINT]),
    during: impl FnOnce() -> T,
) -> Result<Taken<T>, String> {
    let watch = Watch::start(s);
    let picked = (|| {
        let froze = freeze(s, &sc.frame)?;
        s.click(way)?;
        spots.iter().try_for_each(|&p| s.on_frozen(p))?;
        s.until(|| s.has(&s.g.ids.copy).then_some(()))
            .ok_or("the strip did not show")?;
        Ok::<_, String>((froze, during()))
    })();
    let watched = watch.stop();
    let (froze, during) = picked?;
    let pic = s.copy_picture(&s.g.ids.copy)?;
    s.ended()?;
    Ok(Taken {
        froze,
        watched,
        pic,
        during,
    })
}

/// The freeze time and the region picked by two corners, whose picture must match the gate's grab exactly.
pub fn region(s: &Snip, sc: &Scene) -> Result<[Value; 2], String> {
    let before = grab(&sc.screen)?;
    let t = taken(s, sc, (&s.g.ids.region, &corners(&sc.region)), || ())?;
    let covered_ms = t.froze.covered_ms;
    let freeze_ok = covered_ms.is_some_and(|ms| ms <= s.g.cover_ms) && t.watched.ok(s.way);
    let freeze = json!({ "covered_ms": covered_ms, "most_ms": s.g.cover_ms, "watch": t.watched, "ok": freeze_ok });
    let (got, want) = (t.pic.size(), size(&sc.region));
    let diff = differing(
        &t.pic.placed(sc.region.left, sc.region.top),
        &before,
        &sc.region,
    );
    let ok = got == want && diff == Some(0.0);
    let region = json!({ "size": got, "want": want, "differing": diff, "dpi": sc.dpi, "ok": ok });
    Ok([freeze, at_scale(region, sc.off.as_deref())])
}

/// Target-window's frame, by a click on it: the frame's size, and its pixels but the caret's as the gate grabbed them.
pub fn window(s: &Snip, sc: &Scene) -> Result<Value, String> {
    let before = grab(&sc.screen)?;
    let frame = wins::frame(s.app.hwnd)?;
    let t = taken(s, sc, (&s.g.ids.window, &[wins::centre(&frame)]), || ())?;
    let (got, want) = (t.pic.size(), size(&frame));
    let diff = differing(&t.pic.placed(frame.left, frame.top), &before, &frame);
    let ok = got == want && diff.is_some_and(|d| d <= s.g.frame_share) && t.watched.ok(s.way);
    Ok(
        json!({ "size": got, "want": want, "differing": diff, "most": s.g.frame_share, "watch": t.watched, "ok": ok }),
    )
}

/// The bubble's box while it shows, and its own drawing within `[g24] wait_ms`, else why there is none.
fn bubble_now(s: &Snip) -> Option<(RECT, Result<Pic, String>)> {
    let b = wins::shown(s.pid, &s.titles.bubble)?;
    let drawn =
        render::drawn_within(b, s.g.wait_ms).and_then(|d| d.ok_or_else(|| BLANK.to_string()));
    Some((win::rect(b).ok()?, drawn))
}

/// What `pic` holds at the bubble's box: against the grab `before` where it lies clear of the keyboard, and against
/// the bubble's own drawing in the middle of its box, which the round bubble fills.
fn bubble_seen(
    pic: &Pic,
    before: &Pic,
    sc: &Scene,
    (b, drawn): (RECT, Result<Pic, String>),
) -> BubbleSeen {
    let clear = (!wins::meet(&b, &sc.keyboard))
        .then(|| differing(pic, before, &b))
        .flatten();
    let (like, drawing) = match drawn {
        Ok(d) => (differing(pic, &d, &middle(&b)).map(|d| 1.0 - d), None),
        Err(why) => (None, Some(why)),
    };
    BubbleSeen {
        clear,
        like,
        drawing,
    }
}

/// The whole screen, by a click on target-window: the keyboard's place must not hold the keyboard the gate grabbed,
/// nor may it show with the freeze, and the bubble's place must hold neither the bubble nor a change.
pub fn full(s: &Snip, sc: &Scene) -> Result<Value, String> {
    let before = grab(&sc.screen)?;
    let t = taken(s, sc, (&s.g.ids.full, &[wins::centre(&sc.frame)]), || {
        bubble_now(s)
    })?;
    let pic = t.pic.placed(sc.screen.left, sc.screen.top);
    let same = differing(&pic, &before, &sc.keyboard).map(|d| 1.0 - d);
    let keyboard_out = judge::keyboard_out(same, t.froze.keyboard, s.g.same_most);
    let seen = t.during.map(|b| bubble_seen(&pic, &before, sc, b));
    let bubble = judge::bubble(s.way, seen, (s.g.frame_share, s.g.same_most));
    let ok = pic.size() == size(&sc.screen)
        && keyboard_out
        && bubble["ok"] == true
        && t.watched.ok(s.way);
    Ok(json!({
        "size": pic.size(), "want": size(&sc.screen), "keyboard_same": same, "most": s.g.same_most,
        "keyboard_at_freeze": t.froze.keyboard, "bubble": bubble, "watch": t.watched, "ok": ok,
    }))
}

/// The editor's part, between logical pixels, through the editor: its Copy must give exactly the part's size.
pub fn edit(s: &Snip, sc: &Scene) -> Result<Value, String> {
    let ids = &s.g.ids;
    covered(s, &sc.frame)?;
    s.click(&ids.region)?;
    corners(&sc.edit).iter().try_for_each(|&p| s.on_frozen(p))?;
    s.click(&ids.edit)?;
    s.wait_for(&s.titles.editor, true, "editor")?;
    let pic = s.copy_picture(&ids.keep_copy)?;
    close_editor(s)?;
    let (got, want) = (pic.size(), size(&sc.edit));
    let v = json!({ "size": got, "want": want, "dpi": sc.dpi, "ok": got == want });
    Ok(at_scale(v, sc.off.as_deref()))
}

/// Closes the editor, answering Don't save when the question shows, and waits for the snip to end.
pub fn close_editor(s: &Snip) -> Result<(), String> {
    s.click(&s.g.ids.edit_close)?;
    let gone = || wins::shown(s.pid, &s.titles.editor).is_none();
    let asks = s.until(|| {
        if s.has(&s.g.ids.drop) {
            Some(true)
        } else {
            gone().then_some(false)
        }
    });
    if asks == Some(true) {
        s.click(&s.g.ids.drop)?;
    }
    s.ended()
}
