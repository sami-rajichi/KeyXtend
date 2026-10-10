//! The device test's steps, each over the running app's windows.

use crate::{GUARDED, Outcome, Rig, wait};
use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;
use kx_ui::{Shell, UiSettings, drawing};
use tracing::Level;

/// The theme whose fonts are all bundled, and the one Windows draws.
const BUNDLED: &str = "dolch";
const WINDOWS: &str = "native";
/// What a font that did not load logs.
const NOT_LOADED: &str = "did not load";

/// The keyboard and its shadow, then the language list and its shadow, which a Loader makes later, all draw on the
/// shared device, and every frame ends on the GUI thread.
pub(crate) fn shared(app: &mut UniquePtr<QGuiApplication>, shell: &Shell, rig: &Rig) -> Outcome {
    let device = shell.device().ok_or("no shared device was made")?;
    wait(app, "the keyboard's first frame", || {
        !rig.face.placed().is_empty() && drawing().0 >= GUARDED
    })?;
    let before = drawing().0;
    rig.face.open_list();
    wait(app, "the language list's first frame", || {
        drawing().0 > before
    })?;
    let (windows, devices) = drawing();
    if device.windows() != windows || devices != 1 {
        return Err(format!(
            "{windows} windows draw on {devices} devices, {} on the shared one",
            device.windows()
        ));
    }
    device
        .gui_only()
        .then_some(())
        .ok_or_else(|| "a frame ended off the GUI thread".into())
}

/// Qt's font list takes every bundled file of a theme from the compiled-in resources, and drops them for Native.
pub(crate) fn fonts(rig: &mut Rig) -> Outcome {
    let theme = |id: &str| -> Result<UiSettings, String> {
        let s = UiSettings::from_defaults().map_err(|e| e.to_string())?;
        Ok(UiSettings {
            theme: id.to_owned(),
            ..s
        })
    };
    for id in [BUNDLED, WINDOWS] {
        rig.show(theme(id)?);
        if !rig.hub.sync_fonts() {
            return Err(format!("{id}: the fonts did not change"));
        }
    }
    match rig.lines.count(Level::WARN, NOT_LOADED) {
        0 => Ok(()),
        n => Err(format!("{n} bundled fonts did not load")),
    }
}

/// Without a shared device the app still starts: each window draws on its own device, as Qt makes them alone.
pub(crate) fn own(app: &mut UniquePtr<QGuiApplication>, shell: &Shell, rig: &Rig) -> Outcome {
    let placed = rig.face.placed().len();
    wait(app, "the keyboard's first frame on its own device", || {
        rig.face.placed().len() > placed && drawing().0 >= GUARDED
    })?;
    let (windows, devices) = drawing();
    if shell.device().is_some() || devices != windows {
        return Err(format!("{windows} windows draw on {devices} devices"));
    }
    Ok(())
}
