//! G25 on its own copy of the test build in Notepad, the history key off, as G20 checks ours: its language key cycles
//! Notepad's layouts, its shortcuts layer's keys work, its Win and V keys open Windows' clipboard panel, and its Power
//! key opens the power dialog, which G25 only ever cancels.
#![cfg(windows)]

use kx_mod_keyboard::{KeyboardSettings, LangKey, Role};
use kx_platform_windows::langinfo::lang_id;
use kx_theme::Themes;
use serde_json::{Value, json};
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::apps::{AppKind, Ctx, Opened};
use crate::clipcopy::ClipSession;
use crate::copyend::Ended;
use crate::facetools::click_on;
use crate::modekey::Wait;
use crate::mouse::round_px;
use crate::ownkeys::{self, Real};
use crate::probe::Doc;
use crate::product::Keyboard;
use crate::win::{self, sleep_ms};
use crate::{board, clipkeep, g25layer, g25wins, langs, probe, product, winclip};

/// The app G25 drives.
pub const APPS: [AppKind; 1] = [AppKind::Notepad];
/// Why the language part is skipped with the List design.
const LIST_SKIP: &str = "the List design opens a list on a click";

/// A layout's language id in hex, such as `0409` for English (US); the handle keeps its sign extension.
fn lang(layout: isize) -> String {
    format!("{:04X}", lang_id(langs::id(layout)))
}

/// Waits until `f` holds, up to G25's switch wait.
pub(crate) fn until(ctx: &Ctx, f: impl Fn() -> bool) -> bool {
    let (g, t) = (&ctx.cfg.g25, &ctx.cfg.timing);
    win::poll_until(g.switch_wait_ms, t.poll_ms, || f().then_some(())).is_some()
}

/// A part's result, or its error as a failed result, so one part's error never hides the others.
pub(crate) fn part(result: Result<Value, String>) -> Value {
    result.unwrap_or_else(|e| json!({ "error": e, "ok": false }))
}

/// Brings Notepad back to the front, refused while a window of the power dialog's class shows, which the front's Alt
/// taps could reach.
pub(crate) fn front(ctx: &Ctx, app: &Opened) -> Result<(), String> {
    g25wins::no_dialog(ctx).and_then(|()| probe::front(ctx, app))
}

/// `part` once Notepad is back in front; a part Notepad cannot come back for is skipped with that error.
fn fronted(ctx: &Ctx, app: &Opened, run: impl FnOnce() -> Result<Value, String>) -> Value {
    part(front(ctx, app).and_then(|()| run()))
}

/// How long a product key G25 clicks gets to read on or off: G25's switch wait.
pub(crate) fn wait(ctx: &Ctx) -> Wait {
    Wait {
        lit_ms: ctx.cfg.g25.switch_wait_ms,
        poll_ms: ctx.cfg.timing.poll_ms,
    }
}

/// True once the app's focus shows `layout`.
pub(crate) fn shows(ctx: &Ctx, app: &Opened, layout: isize) -> bool {
    until(ctx, || langs::layout_of(app.hwnd) == Some(layout))
}

/// Asks the app's focus for `layout`; true once it shows it within G25's switch wait.
pub(crate) fn switch_to(ctx: &Ctx, app: &Opened, layout: isize) -> bool {
    langs::ask(app.hwnd, layout).is_ok() && shows(ctx, app, layout)
}

/// The layout one step on from the app's own, as the product steps; `None` when the app's focus has none.
fn next(app: &Opened) -> Option<isize> {
    let cur = langs::layout_of(app.hwnd)?;
    langs::step_from(&langs::installed(), cur, 1)
}

/// Why design `design` cannot be stepped by a click, if it cannot.
fn skip_reason(design: LangKey) -> Option<&'static str> {
    (design == LangKey::List).then_some(LIST_SKIP)
}

/// The middle of the part of language key `r` that steps on: a click on its first `back_share`, from the left, goes
/// back.
fn forward(r: &RECT, back_share: f32) -> POINT {
    let width = f64::from(r.right - r.left);
    POINT {
        x: round_px(f64::from(r.left) + width * (1.0 + f64::from(back_share)) / 2.0),
        y: r.top + (r.bottom - r.top) / 2,
    }
}

/// The product's language key: its keyboard's window, where a click steps on, and how long the keyboard takes to
/// follow the layout in front, which each click waits first, since the key steps from the layout the keyboard shows.
struct LangButton {
    hwnd: HWND,
    at: POINT,
    follow_ms: u64,
}

impl LangButton {
    /// The language key of the product keyboard on screen, whose copy has `settings`.
    fn find(ctx: &Ctx, settings: &KeyboardSettings) -> Result<Self, String> {
        let kb = Keyboard::find()?;
        let shape = Themes::shipped().map_err(|e| format!("theme {e}"))?.shape;
        Ok(Self {
            hwnd: kb.hwnd,
            at: forward(&kb.rect(&board::id_of(Role::Lang)?)?, shape.lang.back_share),
            follow_ms: product::follow_after(settings, ctx.cfg.timing.poll_ms),
        })
    }
}

/// The language key to click, or why the language part from layout `from` cannot run, as its result.
fn ready(ctx: &Ctx, from: &str) -> Result<LangButton, Value> {
    let fail = |e: String| json!({ "from": from, "error": e, "ok": false });
    let settings = product::keyboard().map_err(fail)?;
    if let Some(why) = skip_reason(settings.lang_key) {
        let design = settings.lang_key;
        return Err(json!({ "from": from, "skipped": why, "lang_key": design, "ok": false }));
    }
    LangButton::find(ctx, &settings).map_err(fail)
}

/// One click on the product's language key: the layout the product steps to and whether the app took it, or the
/// error.
fn step(ctx: &Ctx, app: &Opened, key: &LangButton) -> Value {
    sleep_ms(key.follow_ms);
    let Some(want) = next(app) else {
        return json!({ "ok": false, "error": "no layout for the app's focus" });
    };
    match ownkeys::click(&Real, app, &|| click_on(key.hwnd, key.at)) {
        Ok(()) => json!({ "to": lang(want), "ok": shows(ctx, app, want) }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

/// Clicks the language key once per installed layout, so the list wraps; the first layout is always put back.
fn layouts(ctx: &Ctx, app: &Opened) -> Value {
    let Some(start) = langs::layout_of(app.hwnd) else {
        return json!({ "error": "no layout for the app's focus", "ok": false });
    };
    let count = langs::installed().len();
    if count < 2 {
        return json!({ "from": lang(start), "skipped": "needs two layouts", "ok": false });
    }
    let key = match ready(ctx, &lang(start)) {
        Ok(key) => key,
        Err(why) => return why,
    };
    let steps: Vec<Value> = (0..count).map(|_| step(ctx, app, &key)).collect();
    let wrapped = langs::layout_of(app.hwnd) == Some(start);
    let restored = wrapped || switch_to(ctx, app, start);
    let all = steps.iter().all(|s| s["ok"] == true);
    json!({
        "from": lang(start), "steps": steps, "wrapped": wrapped, "restored": restored,
        "follow_ms": key.follow_ms, "ok": all && wrapped,
    })
}

/// Every part in turn on Notepad, which comes back to the front before each, else the part is skipped; a part's error
/// fails that part only. Power goes last, alone on screen.
fn drive(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    front(ctx, app)?;
    let layouts = layouts(ctx, app);
    let layer = front(ctx, app).and_then(|()| clipkeep::keep(ctx.cfg, || g25layer::run(ctx, app)));
    let (shortcuts, switch_app) = layer.unwrap_or_else(|e| (part(Err(e.clone())), part(Err(e))));
    let window = fronted(ctx, app, || g25wins::clip_window(ctx, app));
    let power = fronted(ctx, app, || g25wins::power(ctx, app));
    let parts = [&layouts, &shortcuts, &switch_app, &window, &power];
    let pass = parts.iter().all(|p| p["ok"] == true);
    Ok(json!({
        "gate": "G25", "app": app.kind.name(), "pass": pass, "layouts": layouts, "shortcuts": shortcuts,
        "switch_app": switch_app, "clipboard_window": window, "power": power,
    }))
}

/// The result of the parts `got` on G25's own copy: it passes only when they did, the copy quit by itself and the test
/// build's settings came `back`.
pub(crate) fn on_copy(
    got: Result<Value, String>,
    ended: Option<&Ended>,
    back: &Result<(), String>,
) -> Value {
    let mut v = got.unwrap_or_else(|e| json!({ "gate": "G25", "error": e, "pass": false }));
    let pass = v["pass"] == true && ended.is_some_and(Ended::ok) && back.is_ok();
    v["copy"] = json!(ended.map(Ended::line));
    v["settings_put_back"] = json!(back.is_ok());
    v["settings_error"] = json!(back.as_ref().err());
    v["pass"] = json!(pass);
    v
}

/// Runs G25 on `name` on its own copy of the test build with `[g25] history_key` off, refused under
/// uiAccess, while a product copy runs or a window of the power dialog's class shows. Its copied text leaves Windows
/// clipboard history, and the test build's settings come back, even when the run fails.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    g25wins::no_dialog(ctx)?;
    let session = ClipSession::open(ctx)?;
    let (g, since) = (&ctx.cfg.g25, winclip::now());
    let copy =
        |doc: &Doc| Ok(session.with_copy(g.history_key, g.ready_ms, |_| drive(ctx, &doc.app)));
    let result = probe::open_text(ctx, kind, &g.text).and_then(|doc| probe::run_on(ctx, doc, copy));
    let history = probe::forget_copies(ctx, since, &|t| winclip::has_prefix(t, &[&g.text]));
    let recovered = session.recovered;
    let back = session.close();
    let ((got, ended), notes) = result.map_err(|e| format!("{e}; history removed: {history}"))?;
    let mut v = on_copy(got, ended.as_ref(), &back);
    v["settings_recovered"] = json!(recovered);
    v["history_removed"] = history;
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use kx_mod_clipboard::{ClipSettings, ID, SPEC};
    use kx_settings::peek_section;
    use toml::Table;

    use super::*;
    use crate::clipcopy::clip_table;
    use crate::copyend::Gone;

    #[test]
    fn the_click_lands_in_the_middle_of_the_part_that_steps_on() {
        let key = RECT {
            left: 100,
            top: 10,
            right: 400,
            bottom: 40,
        };
        let share = 0.3333;
        let p = forward(&key, share);
        assert_eq!((p.x, p.y), (300, 25));
        assert!(
            f64::from(p.x) > 100.0 + 300.0 * f64::from(share),
            "past the back part"
        );
        assert_eq!(forward(&key, 0.0).x, 250, "no back part: the middle");
    }

    #[test]
    fn only_the_list_design_is_skipped_since_a_click_opens_the_list() {
        assert_eq!(skip_reason(LangKey::Carousel), None);
        assert_eq!(skip_reason(LangKey::Cycle), None);
        assert!(skip_reason(LangKey::List).is_some());
    }

    #[test]
    fn g25s_own_copy_runs_with_the_history_key_off() {
        let c = crate::config::load().expect("loads");
        let t = clip_table(&Table::new(), c.g25.history_key, &c.g20_product.skip_app);
        let s: ClipSettings =
            peek_section(Some(&t.expect("built")), ID, &SPEC).expect("reads back");
        assert!(!s.history_key, "Win and V reach Windows' own history");
        assert!(c.g25.ready_ms > 0);
    }

    #[test]
    fn g25_passes_only_when_its_parts_its_copy_and_the_settings_did() {
        let quit = Ended {
            gone: Gone::Quit,
            note: None,
            released: Vec::new(),
        };
        let good = || Ok(json!({ "pass": true }));
        assert_eq!(on_copy(good(), Some(&quit), &Ok(()))["pass"], true);
        assert_eq!(on_copy(good(), None, &Ok(()))["pass"], false, "no copy");
        let back = on_copy(good(), Some(&quit), &Err("locked".into()));
        assert_eq!(
            (back["pass"].as_bool(), back["settings_error"].as_str()),
            (Some(false), Some("locked"))
        );
        let failed = on_copy(Err("no keyboard".into()), Some(&quit), &Ok(()));
        assert_eq!(
            (failed["pass"].as_bool(), failed["error"].as_str()),
            (Some(false), Some("no keyboard"))
        );
        let crashed = Ended {
            gone: Gone::Crashed(Some(1)),
            ..quit
        };
        assert_eq!(on_copy(good(), Some(&crashed), &Ok(()))["pass"], false);
    }

    #[test]
    fn lang_is_the_low_word_in_hex() {
        assert_eq!(lang(0x0409_0409), "0409");
        assert_eq!(
            lang(isize::try_from(0xF002_1C01_u32).expect("fits")),
            "1C01"
        );
        let extended = isize::try_from(0xFFFF_FFFF_F029_1C01_u64.cast_signed()).expect("64-bit");
        assert_eq!(lang(extended), "1C01", "a sign-extended handle");
    }
}
