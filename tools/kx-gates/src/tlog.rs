//! Reads the target-window log, whose format is in `kx_target_window::log`, after the last start.
#![cfg(windows)]

use kx_target_window::log::{
    COMMENT, FOCUS_LOST, KEY, LAYOUT_KEPT, MOUSE, Press, SEP, SIDEWAYS, START, UPRIGHT, WHEEL,
};

/// Radix of the logged UTF-16 units.
const HEX: u32 = 16;

/// One logged UTF-16 unit and when it arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// QPC time in microseconds.
    pub us: i64,
    /// The UTF-16 code unit.
    pub unit: u16,
}

/// The units logged since the last start; bad lines are skipped.
pub fn parse(log: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            units.clear();
            continue;
        }
        if line.starts_with(COMMENT) {
            continue;
        }
        let mut parts = line.trim().split(SEP);
        let us = parts.next().and_then(|t| t.parse::<i64>().ok());
        let unit = parts.next().and_then(|u| u16::from_str_radix(u, HEX).ok());
        if let (Some(us), Some(unit)) = (us, unit) {
            units.push(Unit { us, unit });
        }
    }
    units
}

/// Focus losses logged since the last start, at or after `since_us`; one without a time counts too.
pub fn focus_lost(log: &str, since_us: i64) -> usize {
    let mut lost = 0;
    for line in log.lines() {
        if line.starts_with(START) {
            lost = 0;
        } else if let Some(rest) = line.strip_prefix(FOCUS_LOST) {
            let us = rest
                .strip_prefix(SEP)
                .and_then(|t| t.trim().parse::<i64>().ok());
            lost += usize::from(us.is_none_or(|us| us >= since_us));
        }
    }
    lost
}

/// One logged mouse press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouse {
    /// What happened.
    pub press: Press,
    /// QPC time in microseconds.
    pub us: i64,
    /// Screen point in physical pixels: x.
    pub x: i32,
    /// Screen point in physical pixels: y.
    pub y: i32,
}

/// The mouse presses logged since the last start; bad lines are skipped.
pub fn mouse(log: &str) -> Vec<Mouse> {
    let mut found = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            found.clear();
        } else if let Some(rest) = line.trim().strip_prefix(MOUSE) {
            found.extend(mouse_fields(rest));
        }
    }
    found
}

/// The fields after `# mouse`: press, µs, x and y.
fn mouse_fields(rest: &str) -> Option<Mouse> {
    let mut f = rest.strip_prefix(SEP)?.split(SEP);
    let press = Press::parse(f.next()?)?;
    let us = f.next()?.parse().ok()?;
    let (x, y) = (f.next()?.parse().ok()?, f.next()?.parse().ok()?);
    Some(Mouse { press, us, x, y })
}

/// One logged key going down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyDown {
    /// QPC time in microseconds.
    pub us: i64,
    /// The virtual key.
    pub vk: u16,
    /// The scan code; an extended key's is `E0xx`.
    pub scan: u16,
}

/// The keys logged going down since the last start; bad lines are skipped.
pub fn keys(log: &str) -> Vec<KeyDown> {
    let mut found = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            found.clear();
        } else if let Some(rest) = line.trim().strip_prefix(KEY) {
            found.extend(key_fields(rest));
        }
    }
    found
}

/// The fields after `# key`: µs, virtual key and scan code.
fn key_fields(rest: &str) -> Option<KeyDown> {
    let mut f = rest.strip_prefix(SEP)?.split(SEP);
    let us = f.next()?.parse().ok()?;
    let vk = u16::from_str_radix(f.next()?, HEX).ok()?;
    let scan = u16::from_str_radix(f.next()?, HEX).ok()?;
    Some(KeyDown { us, vk, scan })
}

/// One wheel turn the box logged, with the point its message carried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wheel {
    /// QPC time in microseconds.
    pub us: i64,
    /// The sideways wheel, not the upright one.
    pub sideways: bool,
    /// The turn, signed as Windows signs it.
    pub delta: i32,
    /// The point's x, as the message carried it.
    pub x: i32,
    /// The point's y, as the message carried it.
    pub y: i32,
}

/// The wheel turns logged since the last start; bad lines are skipped.
pub fn wheels(log: &str) -> Vec<Wheel> {
    let mut found = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            found.clear();
        } else if let Some(rest) = line.trim().strip_prefix(WHEEL) {
            found.extend(wheel_fields(rest));
        }
    }
    found
}

/// The fields after `# wheel`: µs, axis, delta, x and y.
fn wheel_fields(rest: &str) -> Option<Wheel> {
    let mut f = rest.strip_prefix(SEP)?.split(SEP);
    let us = f.next()?.parse().ok()?;
    let sideways = match f.next()? {
        SIDEWAYS => true,
        UPRIGHT => false,
        _ => return None,
    };
    let delta = f.next()?.parse().ok()?;
    let (x, y) = (f.next()?.parse().ok()?, f.next()?.parse().ok()?);
    Some(Wheel {
        us,
        sideways,
        delta,
        x,
        y,
    })
}

/// When `mark`, such as `# woke`, was logged since the last start, in order; bad lines are skipped.
pub fn marks(log: &str, mark: &str) -> Vec<i64> {
    let mut found = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            found.clear();
        } else if let Some(rest) = line.trim().strip_prefix(mark) {
            found.extend(rest.strip_prefix(SEP).and_then(|t| t.parse::<i64>().ok()));
        }
    }
    found
}

/// True when target-window's last run could not put back the layout from before `--layout`.
pub fn layout_kept(log: &str) -> bool {
    let last = log.rfind(START).map_or(log, |at| &log[at..]);
    last.lines().any(|l| l.starts_with(LAYOUT_KEPT))
}

/// The text the units spell; broken surrogates become U+FFFD.
pub fn decode(units: &[Unit]) -> String {
    let raw: Vec<u16> = units.iter().map(|u| u.unit).collect();
    String::from_utf16_lossy(&raw)
}

/// Target-window's mouse presses logged at or after `since` (µs).
pub fn mouse_since(ctx: &crate::apps::Ctx, since: i64) -> Result<Vec<Mouse>, String> {
    let path = &ctx.target.log;
    let log = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(mouse(&log).into_iter().filter(|m| m.us >= since).collect())
}

#[cfg(test)]
mod tests;
