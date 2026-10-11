//! The device-loss and window-failure steps (ADR-0021): debug-only triggers mark the shared device lost or a window
//! unable to draw, Qt's loop ends, and the windows are made again over the same hub, at the face's place and size, on
//! a device on the adapter picked anew, one try per turn of the loop, until `[ui] device_rebuilds` runs out.

use crate::checks::{adopted, iconed};
use crate::face::HOME;
use crate::rig::{GONE, GUARDED, Outcome, PICK, Rig, TICKS, pick_next, ticker, timer, wait};
use cxx::UniquePtr;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine};
use kx_module_api::ports::Rect;
use kx_module_api::{Notice, keys};
use kx_ui::{ADAPTER_VAR, After, Device, Resize, Shell, UiSettings, drawing, qt_sees, wired};
use tracing::Level;

/// What the guard logs once a run guarded our windows.
const GUARDED_LINE: &str = "windows guarded";
/// What a loss past `[ui] device_rebuilds` warns, a try at a new device that failed, a new shared device, and a window
/// failure past the tries.
const SPENT: &str = "no new graphics device could be made, so each window makes its own";
const NOT_MADE: &str = "could be made yet";
const MADE: &str = "made again on a new graphics device";
const ENDS: &str = "so the app ends";
/// The fewest ticks Qt's loop must run while a try waits, as the wait is many ticks long.
const WAITED: usize = 2;
/// The tries a window failure takes once sharing is off, so at least one waits.
const UNSHARED_TRIES: u32 = 2;
/// The watchdog ends Qt's loop with its own code if the loss does not end it first; its time in ms, and its URL.
const WATCHDOG_MS: u32 = 5_000;
const WATCHDOG_CODE: i32 = 3;
const WATCHDOG_URL: &str = "qrc:/kx-ui-tests/device-watchdog.qml";

/// Marks the shared device lost, as a driver reset would, and waits until a frame finds it.
pub(crate) fn lose(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell) -> Outcome {
    shell.lose();
    wait(app, "a frame finding the device lost", || {
        shell.device().is_some_and(Device::lost)
    })
}

/// Makes the windows of a lost device again.
pub(crate) fn rebuilt(shell: &mut Shell) -> Outcome {
    match shell.rebuilt() {
        After::Again => Ok(()),
        after => Err(format!("the windows were not made again: {after:?}")),
    }
}

/// The marks a step waits past: the keyboard's placements and the guard's runs so far.
pub(crate) fn marks(rig: &Rig) -> (usize, usize) {
    (
        rig.face.placed().len(),
        rig.lines.count(Level::INFO, GUARDED_LINE),
    )
}

/// Waits until the windows made again placed and guarded the keyboard past `was`, and the tray shows its one icon.
pub(crate) fn back(
    app: &mut UniquePtr<QGuiApplication>,
    rig: &Rig,
    was: (usize, usize),
) -> Outcome {
    wait(app, "the rebuilt keyboard placed and guarded", || {
        let now = marks(rig);
        now.0 > was.0 && now.1 > was.1 && drawing().0 >= GUARDED && rig.tray.shown().len() == 1
    })
}

/// The keyboard's last reported box, which must sit at the face's place with a size.
pub(crate) fn kept(rig: &Rig) -> Result<Rect, String> {
    let last = rig.face.placed().last().copied();
    match last {
        Some(r) if r.x == HOME.x && r.y == HOME.y && r.w > 0 && r.h > 0 => Ok(r),
        _ => Err(format!("placed at {last:?}, not at {HOME:?}")),
    }
}

/// With no ticker, `trigger` alone ends Qt's loop, by our own exit, long before the watchdog's.
fn alone(
    app: &mut UniquePtr<QGuiApplication>,
    tick: &mut UniquePtr<QQmlApplicationEngine>,
    trigger: impl FnOnce(),
) -> Outcome {
    *tick = UniquePtr::null();
    let watchdog = timer(WATCHDOG_MS, WATCHDOG_CODE, false, WATCHDOG_URL)?;
    trigger();
    let code = app.as_mut().map(|a| a.exec());
    drop(watchdog);
    *tick = ticker()?;
    match code {
        Some(0) => Ok(()),
        _ => Err(format!("Qt's loop ended with {code:?}, not by the trigger")),
    }
}

/// Runs Qt's loop, then `rebuilt`, while the windows wait for the next try; what came after, and how many waits.
fn settle(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell) -> (After, usize) {
    let mut waits = 0;
    for _ in 0..TICKS {
        app.as_mut().map(|a| a.exec());
        match shell.rebuilt() {
            After::Wait => waits += 1,
            after => return (after, waits),
        }
    }
    (After::Wait, waits)
}

/// The first try fails and hands back at once with no device; Qt's loop then runs tick by tick through `[ui]
/// device_retry_ms`, so nothing sleeps on the GUI thread, and the next try makes the windows again.
fn retried(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell) -> Outcome {
    let first = shell.rebuilt();
    if first != After::Wait || shell.device().is_some() {
        return Err(format!("a failed try gave {first:?}, not a wait"));
    }
    match settle(app, shell) {
        (After::Again, waits) if waits >= WAITED => Ok(()),
        (after, waits) => Err(format!("{after:?} after {waits} ticks of waiting")),
    }
}

/// Every drawn window has `sceneGraphError` heard, so none can end the app.
pub(crate) fn watched() -> Outcome {
    let (windows, heard) = (drawing().0, wired());
    if windows > 0 && heard == windows {
        Ok(())
    } else {
        Err(format!(
            "{heard} of {windows} windows have their scene graph errors heard"
        ))
    }
}

/// The first loss, during a resize and with no ticker: the device's own exit ends Qt's loop, the resize ends, and the
/// windows come back once, on one shared device made on the second try, after Qt's loop ran through the wait, as the
/// first pick names a gone adapter; the pin is set again, the icons come from the provider and the soft shadows from
/// the adopter again, the keyboard sits at the face's place and the new banner hears the hub. The keyboard's box.
pub(crate) fn once(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &Rig,
    tick: &mut UniquePtr<QQmlApplicationEngine>,
) -> Result<Rect, String> {
    let _ = rig.hub.resize(Resize::Begin);
    if !rig.face.resizing() {
        return Err("the resize did not begin".into());
    }
    let (was, made, shadows) = (marks(rig), kx_ui::icon_textures().0, kx_ui::adopted());
    alone(app, tick, || shell.lose())?;
    if !shell.device().is_some_and(Device::lost) {
        return Err("Qt's loop ended before a frame found the loss".into());
    }
    pick_next(GONE);
    retried(app, shell)?;
    back(app, rig, was)?;
    watched()?;
    iconed(rig, made)?;
    adopted(shadows)?;
    if rig.face.resizing() || !qt_sees(ADAPTER_VAR) || rig.lines.count(Level::WARN, NOT_MADE) != 1 {
        return Err(
            "the resize ran on, or the gone adapter was not tried once with the pin kept".into(),
        );
    }
    let device = shell.device().ok_or("no shared device after the rebuild")?;
    let (windows, devices) = drawing();
    if device.lost() || device.windows() != windows || devices != 1 {
        return Err(format!(
            "{windows} windows draw on {devices} devices after the rebuild"
        ));
    }
    rig.hub.notice(Notice {
        key: keys::SETTINGS_UNSAVED,
        module: None,
        args: Vec::new(),
    });
    wait(app, "the rebuilt banner taking a notice", || {
        !rig.hub.shown_notices().is_empty()
    })?;
    kept(rig)
}

/// A loss past `[ui] device_rebuilds` makes the windows again on their own devices, with one warning, and the keyboard
/// keeps box `was`.
pub(crate) fn past_most(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &mut Rig,
    was: Rect,
) -> Outcome {
    let s = UiSettings::from_defaults().map_err(|e| e.to_string())?;
    rig.show(UiSettings {
        device_rebuilds: 1,
        ..s
    });
    let marks = marks(rig);
    lose(app, shell)?;
    rebuilt(shell)?;
    back(app, rig, marks)?;
    let (windows, devices) = drawing();
    if shell.device().is_some() || devices != windows {
        return Err(format!(
            "{windows} windows draw on {devices} devices past the rebuilds"
        ));
    }
    let now = kept(rig)?;
    if now != was {
        return Err(format!("the box went from {was:?} to {now:?}"));
    }
    match rig.lines.count(Level::WARN, SPENT) {
        1 => Ok(()),
        n => Err(format!("{n} warnings of spent rebuilds")),
    }
}

/// A window whose scene graph cannot start, as when it cannot make its device during a reset, ends Qt's loop through
/// the watch, where Qt would end the app; the windows come back on one shared device, as after a loss.
pub(crate) fn failed(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &Rig,
    tick: &mut UniquePtr<QQmlApplicationEngine>,
) -> Outcome {
    let was = marks(rig);
    alone(app, tick, || shell.fail_window())?;
    rebuilt(shell)?;
    back(app, rig, was)?;
    let device = shell.device().ok_or("no shared device after the failure")?;
    let (windows, devices) = drawing();
    if device.windows() != windows || devices != 1 {
        return Err(format!(
            "{windows} windows draw on {devices} devices after the failure"
        ));
    }
    Ok(())
}

/// A window that cannot draw once `[ui] device_rebuilds` is spent ends the app with a failure and one error.
pub(crate) fn spent(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell, rig: &Rig) -> Outcome {
    shell.fail_window();
    app.as_mut().map(|a| a.exec());
    match (shell.rebuilt(), rig.lines.count(Level::ERROR, ENDS)) {
        (After::Failed, 1) => Ok(()),
        (after, n) => Err(format!("{after:?} with {n} errors, not a failure")),
    }
}

/// In new windows, a frame off the GUI thread turns sharing off for the run and each window takes its own device; a
/// window that cannot draw then gets no shared device, as its tries run out through Qt's loop and the app ends.
pub(crate) fn unshared(app: &mut UniquePtr<QGuiApplication>, rig: &mut Rig) -> Outcome {
    rig.show(UiSettings {
        device_rebuilds: UNSHARED_TRIES,
        ..rig.hub.settings()
    });
    let was = marks(rig);
    let mut shell = Shell::open(None, PICK).ok_or("the root QML did not load")?;
    back(app, rig, was)?;
    let was = marks(rig);
    shell.stray();
    wait(app, "a frame finding the stray frame", || {
        shell.device().is_some_and(Device::lost)
    })?;
    rebuilt(&mut shell)?;
    back(app, rig, was)?;
    watched()?;
    let (made, ends) = (
        rig.lines.count(Level::WARN, MADE),
        rig.lines.count(Level::ERROR, ENDS),
    );
    shell.fail_window();
    let (after, waits) = settle(app, &mut shell);
    let shared = shell.device().is_some() || rig.lines.count(Level::WARN, MADE) != made;
    if after != After::Failed
        || waits == 0
        || shared
        || rig.lines.count(Level::ERROR, ENDS) != ends + 1
    {
        return Err(format!(
            "{after:?} after {waits} waits, a shared device made: {shared}"
        ));
    }
    Ok(())
}
