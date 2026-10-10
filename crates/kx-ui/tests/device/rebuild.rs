//! The device-loss steps (ADR-0021): the debug-only trigger marks the shared device lost, a frame finds it, and the
//! windows are made again over the same hub, at the face's place and size, on a device on the adapter picked anew,
//! until `[ui] device_rebuilds` runs out.

use crate::face::HOME;
use crate::rig::{GONE, GUARDED, Outcome, Rig, pick_next, ticker, timer, wait};
use cxx::UniquePtr;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine};
use kx_module_api::ports::Rect;
use kx_module_api::{Notice, keys};
use kx_ui::{ADAPTER_VAR, After, Device, Resize, Shell, UiSettings, drawing, qt_sees};
use tracing::Level;

/// What the guard logs once a run guarded our windows.
const GUARDED_LINE: &str = "windows guarded";
/// What a loss past `[ui] device_rebuilds` warns, and a try at a new device that failed.
const TOO_OFTEN: &str = "too often";
const NOT_MADE: &str = "after the loss";
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

/// With no ticker, a loss alone ends Qt's loop, by the device's own exit, long before the watchdog's.
fn alone(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    tick: &mut UniquePtr<QQmlApplicationEngine>,
) -> Outcome {
    *tick = UniquePtr::null();
    let watchdog = timer(WATCHDOG_MS, WATCHDOG_CODE, false, WATCHDOG_URL)?;
    shell.lose();
    let code = app.as_mut().map(|a| a.exec());
    drop(watchdog);
    *tick = ticker()?;
    match code {
        Some(0) if shell.device().is_some_and(Device::lost) => Ok(()),
        _ => Err(format!("Qt's loop ended with {code:?}, not by the loss")),
    }
}

/// The first loss, during a resize and with no ticker: the device's own exit ends Qt's loop, the resize ends, and the
/// windows come back once, on one shared device made on the second try, as the first pick names a gone adapter; the
/// pin is set again, the keyboard sits at the face's place and the new banner hears the hub. The keyboard's box.
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
    let was = marks(rig);
    alone(app, shell, tick)?;
    pick_next(GONE);
    rebuilt(shell)?;
    back(app, rig, was)?;
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
    match rig.lines.count(Level::WARN, TOO_OFTEN) {
        1 => Ok(()),
        n => Err(format!("{n} warnings of too many rebuilds")),
    }
}
