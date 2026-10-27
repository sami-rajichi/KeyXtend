//! What G6 records without judging: Windows' wheel routing, UI Automation's percent lag after one step, and on
//! target-window the wheel turns around a hang and the point a DPI-unaware window receives. Numbers only, never a
//! point: the unaware record keeps the offsets from the aimed point.
#![cfg(windows)]

use std::time::Instant;

use kx_module_api::ports::Dir;
use kx_platform_windows::screen;
use kx_platform_windows::uia::Uia;
use kx_target_window::log::{HUNG, WOKE};
use serde_json::{Value, json};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::HiDpi::{GetDpiForWindow, PhysicalToLogicalPointForPerMonitorDPI};
use windows::Win32::UI::WindowsAndMessaging::{
    MOUSEWHEEL_ROUTING_FOCUS, MOUSEWHEEL_ROUTING_HYBRID, MOUSEWHEEL_ROUTING_MOUSE_POS,
    SPI_GETMOUSEWHEELROUTING, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
};

use crate::apps::{self, Ctx, Opened};
use crate::g5::RCLICK;
use crate::g6scene::Scene;
use crate::out::say;
use crate::scroll::{self, percent_moved};
use crate::testbuild::TestBuild;
use crate::tlog::{self, Wheel};
use crate::win::{self, sleep_ms};
use crate::{g5play, launch, mouseguard, stats};

/// The record of the wheel turns around a hang.
pub const HANG: &str = "hang";
/// The record of the point a DPI-unaware window receives.
pub const UNAWARE: &str = "unaware";
/// G6's records on target-window, by name.
pub const NAMES: [&str; 2] = [HANG, UNAWARE];

/// Windows' wheel routings by `SPI_GETMOUSEWHEELROUTING` value, and their names in the report.
const ROUTINGS: [(u32, &str); 3] = [
    (MOUSEWHEEL_ROUTING_FOCUS, "focus"),
    (MOUSEWHEEL_ROUTING_HYBRID, "hybrid"),
    (MOUSEWHEEL_ROUTING_MOUSE_POS, "pointer"),
];

/// Windows' wheel routing, and whether a sent wheel turn goes to the window under the pointer, as the product's
/// `Send` route needs, or a posted one is its only way.
#[allow(unsafe_code, reason = "A system setting read into a local.")]
pub fn routing() -> Value {
    let mut value = 0u32;
    let data = std::ptr::from_mut(&mut value).cast();
    let keep = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default();
    // SAFETY: the call writes one u32 into a local that outlives it.
    let read = unsafe { SystemParametersInfoW(SPI_GETMOUSEWHEELROUTING, 0, Some(data), keep) };
    let name = ROUTINGS.iter().find(|(v, _)| *v == value).map(|(_, n)| *n);
    let sends = value == MOUSEWHEEL_ROUTING_MOUSE_POS;
    json!({ "read": read.is_ok(), "setting": value, "name": name, "send_reaches_pointer": sends })
}

/// How long UI Automation's percent takes to show one `ScrollPattern` step of G6's own, in ms; it steps back after.
/// None without a pattern, or when the percent never showed it within `lag_wait_ms`.
pub fn lag(s: &Scene) -> Result<Option<u64>, String> {
    let Some(sp) = s.scroller() else {
        return Ok(None);
    };
    mouseguard::allowed()?;
    let before = s.pair();
    scroll::step(sp, Dir::Down)?;
    let began = Instant::now();
    let (wait, poll) = (s.ctx.cfg.g6.lag_wait_ms, s.ctx.cfg.timing.poll_ms);
    let shown = || percent_moved(Dir::Down, before, s.pair()).then(|| win::ms_since(began));
    let lag = win::poll_until(wait, poll, shown);
    scroll::step(sp, Dir::Up)?;
    sleep_ms(s.ctx.cfg.probes.settle_ms);
    Ok(lag)
}

/// How many of `wheels` target-window logged at or after `from_us` and before `to_us`.
pub fn between(wheels: &[Wheel], from_us: i64, to_us: i64) -> usize {
    wheels
        .iter()
        .filter(|w| (from_us..to_us).contains(&w.us))
        .count()
}

/// Takes record `name` on a filled target-window under the test build's pad; it passes when the test build quit.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let uia = Uia::new().map_err(|e| e.to_string())?;
    let build = TestBuild::open(ctx, None)?;
    build.set(RCLICK, false)?;
    let target = open(ctx, name)?;
    let taken = take(ctx, &uia, &target, name);
    let left = apps::clean_up(ctx, target);
    let closed = build.close();
    let taken = taken.map_err(|e| closed.failed(&e, &left))?;
    say!("G6 {name}: {taken}");
    Ok(json!({
        "gate": "G6", "record": name, "pass": closed.pass(true), "taken": taken, "clean_up": left,
        "routing": routing(), "test_build": closed.json(),
    }))
}

/// Target-window filled, and DPI-unaware for that record, in front.
fn open(ctx: &Ctx, name: &str) -> Result<Opened, String> {
    let t = ctx.target;
    let mut args = vec![t.fill_arg.clone()];
    if name == UNAWARE {
        args.push(t.unaware_arg.clone());
    }
    launch::to_front(ctx, launch::start_target_with(ctx, &args)?)
}

/// Lays target-window under the pad, glides in, and takes the record.
fn take(ctx: &Ctx, uia: &Uia, target: &Opened, name: &str) -> Result<Value, String> {
    let s = Scene::new(ctx, uia, target)?;
    let kept = s.glide_in()?;
    if name == HANG {
        hang(&s)
    } else {
        unaware(&s, kept)
    }
}

/// Target-window's log as it stands.
fn log(ctx: &Ctx) -> Result<String, String> {
    let path = &ctx.target.log;
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Scrolls down, asks target-window to hang `hang_lead_ms` later, then Stop once it woke: the wheel turns it logged
/// in `burst_ms` before the hang and right after it woke.
fn hang(s: &Scene) -> Result<Value, String> {
    let g = &s.ctx.cfg.g6;
    s.arrow(Dir::Down)?;
    sleep_ms(g.hang_lead_ms);
    g5play::hang_target(s.app.hwnd, g.hang_ms)?;
    sleep_ms(g.hang_ms + g.burst_ms);
    s.stop()?;
    let text = log(s.ctx)?;
    let (hung, woke) = (tlog::marks(&text, HUNG), tlog::marks(&text, WOKE));
    let (Some(&hung), Some(&woke)) = (hung.last(), woke.last()) else {
        return Err("target-window logged no hang".to_string());
    };
    let (wheels, burst) = (tlog::wheels(&text), stats::us_of(g.burst_ms));
    Ok(json!({
        "hang_ms": g.hang_ms, "burst_ms": g.burst_ms, "wheels": wheels.len(),
        "before": between(&wheels, hung - burst, hung), "after": between(&wheels, woke, woke + burst),
    }))
}

/// Scrolls down for `move_ms`, then Stop: how far the first wheel point the unaware window received lies from the
/// aimed point `kept`, in its own pixels and in physical ones.
fn unaware(s: &Scene, kept: POINT) -> Result<Value, String> {
    s.arrow(Dir::Down)?;
    sleep_ms(s.ctx.cfg.g6.move_ms);
    s.stop()?;
    let wheels = tlog::wheels(&log(s.ctx)?);
    let want = logical(s, kept);
    let off = |to: POINT| wheels.first().map(|w| [w.x - to.x, w.y - to.y]);
    let screen_dpi = screen::dpi_of(&win::rect(s.app.hwnd)?);
    Ok(json!({
        "wheels": wheels.len(), "off_logical_px": off(want), "off_physical_px": off(kept),
        "window_dpi": window_dpi(s), "screen_dpi": screen_dpi,
    }))
}

/// Physical point `p` in the pixels of the scene's window, as the product converts it before it posts.
#[allow(unsafe_code, reason = "A point conversion into a local.")]
fn logical(s: &Scene, p: POINT) -> POINT {
    let mut at = p;
    // SAFETY: a local point and a plain query on a window G6 opened; a failure leaves the point.
    let _ = unsafe { PhysicalToLogicalPointForPerMonitorDPI(Some(s.app.hwnd), &raw mut at) };
    at
}

/// The DPI the scene's window runs at; 96 for a DPI-unaware one.
#[allow(unsafe_code, reason = "A plain window query.")]
fn window_dpi(s: &Scene) -> u32 {
    // SAFETY: a plain query on a window G6 opened.
    unsafe { GetDpiForWindow(s.app.hwnd) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(us: i64) -> Wheel {
        Wheel {
            us,
            sideways: false,
            delta: -40,
            x: 0,
            y: 0,
        }
    }

    #[test]
    fn counts_the_wheel_turns_from_the_start_up_to_the_end() {
        let wheels = [at(5), at(10), at(19), at(20), at(30)];
        assert_eq!(between(&wheels, 10, 20), 2);
        assert_eq!(between(&wheels, 0, 100), 5);
        assert_eq!(between(&wheels, 31, 40), 0);
    }

    #[test]
    fn each_routing_has_its_own_name() {
        let names: std::collections::BTreeSet<_> = ROUTINGS.iter().map(|(_, n)| n).collect();
        let values: std::collections::BTreeSet<_> = ROUTINGS.iter().map(|(v, _)| v).collect();
        assert_eq!(
            (names.len(), values.len()),
            (ROUTINGS.len(), ROUTINGS.len())
        );
    }
}
