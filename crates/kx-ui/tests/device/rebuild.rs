//! The device-loss steps (ADR-0021): the debug-only trigger marks the shared device lost, a frame finds it, and the
//! windows are made again over the same hub, at the face's place and size, until `[ui] device_rebuilds` runs out.

use crate::face::HOME;
use crate::{GUARDED, Outcome, Rig, wait};
use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;
use kx_module_api::ports::Rect;
use kx_module_api::{Notice, keys};
use kx_ui::{Device, Resize, Shell, UiSettings, drawing};
use tracing::Level;

/// What the guard logs once a run guarded our windows.
const GUARDED_LINE: &str = "windows guarded";
/// What a loss past `[ui] device_rebuilds` warns.
const TOO_OFTEN: &str = "too often";

/// Marks the shared device lost, as a driver reset would, and waits until a frame finds it.
fn lose(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell) -> Outcome {
    shell.lose();
    wait(app, "a frame finding the device lost", || {
        shell.device().is_some_and(Device::lost)
    })
}

/// Makes the windows again after a loss, then waits until the keyboard is placed and guarded again and the tray shows
/// its one icon.
fn rebuild(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell, rig: &Rig) -> Outcome {
    let placed = rig.face.placed().len();
    let guarded = rig.lines.count(Level::INFO, GUARDED_LINE);
    lose(app, shell)?;
    if !shell.rebuilt() {
        return Err("the windows were not made again".into());
    }
    wait(app, "the rebuilt keyboard placed and guarded", || {
        rig.face.placed().len() > placed
            && rig.lines.count(Level::INFO, GUARDED_LINE) > guarded
            && drawing().0 >= GUARDED
            && rig.tray.shown().len() == 1
    })
}

/// The keyboard's last reported box, which must sit at the face's place with a size.
fn kept(rig: &Rig) -> Result<Rect, String> {
    let last = rig.face.placed().last().copied();
    match last {
        Some(r) if r.x == HOME.x && r.y == HOME.y && r.w > 0 && r.h > 0 => Ok(r),
        _ => Err(format!("placed at {last:?}, not at {HOME:?}")),
    }
}

/// A loss during a resize: the resize ends first, the windows come back once on a new shared device, the keyboard sits
/// at the face's place, and the new banner hears the hub; the keyboard's box, for the next loss to keep.
pub(crate) fn once(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &Rig,
) -> Result<Rect, String> {
    let _ = rig.hub.resize(Resize::Begin);
    if !rig.face.resizing() {
        return Err("the resize did not begin".into());
    }
    rebuild(app, shell, rig)?;
    if rig.face.resizing() {
        return Err("the resize still ran after the rebuild".into());
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
    rebuild(app, shell, rig)?;
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
