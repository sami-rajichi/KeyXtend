//! The device test's steps without a loss, each over the running app's windows.

use crate::rebuild::watched;
use crate::rig::{FIRST, GUARDED, Outcome, Rig, wait};
use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;
use kx_theme::Themes;
use kx_theme::assets::FontFiles;
use kx_ui::{
    ADAPTER_VAR, Device, Shell, UiSettings, drawing, font_families, pin_adapter, qt_sees, released,
};
use std::thread;
use tracing::Level;

/// The themes the font step shows in turn: two with bundled fonts, then the one Windows draws, which loads none.
const THEMES: [&str; 3] = ["dolch", "et66", "native"];
/// What a font that did not load logs, and what each warning of a device that could not be shared holds.
const NOT_LOADED: &str = "did not load";
const GRAPHICS: &str = "graphics";
/// What a refused icon warns.
const REFUSED: &str = "icon was refused";

/// The windows drew icons through the provider since it had made `made` textures, and it refused none: each engine,
/// a rebuilt one too, has the provider, as Qt finds no other.
pub(crate) fn iconed(rig: &Rig, made: u64) -> Outcome {
    let drawn = kx_ui::icon_textures().0.saturating_sub(made);
    match rig.lines.count(Level::WARN, REFUSED) {
        0 if drawn > 0 => Ok(()),
        n => Err(format!(
            "{drawn} icons drawn through the provider, {n} refused"
        )),
    }
}

/// The keyboard and its shadow, then the language list and its shadow, which a Loader makes later, all draw on the
/// shared device, and every frame ends on the GUI thread; each has its scene graph errors heard, the icons come from
/// the provider, and no device is made off that thread, where the pin stays.
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
    if device.windows() != windows || devices != 1 || !device.gui_only() {
        return Err(format!(
            "{windows} windows draw on {devices} devices, {} on the shared one, or off the GUI thread",
            device.windows()
        ));
    }
    watched()?;
    iconed(rig, 0)?;
    elsewhere(rig)
}

/// Off the GUI thread no device is made, and the adapter pin stays with no warning.
fn elsewhere(rig: &Rig) -> Outcome {
    let warned = rig.lines.count(Level::WARN, GRAPHICS);
    if !pin_adapter(FIRST) {
        return Err("the adapter was not pinned".into());
    }
    let made = thread::spawn(|| Device::make().is_some()).join();
    let kept = qt_sees(ADAPTER_VAR) && rig.lines.count(Level::WARN, GRAPHICS) == warned;
    match made {
        Ok(false) if kept => Ok(()),
        _ => Err("off the GUI thread a device was made or the pin dropped".into()),
    }
}

/// A theme picked in the settings reaches the windows through the wake alone: it loads the theme's fonts, so Qt has its
/// first Arabic family, and each shown window drops Qt's cached graphics once it drew the new look, then the free heap
/// pages go back once; going back drops those fonts and gives back once more. Nothing went back at idle before.
pub(crate) fn relooked(app: &mut UniquePtr<QGuiApplication>, rig: &mut Rig) -> Outcome {
    let themes = Themes::shipped().map_err(|e| e.to_string())?;
    let look = &themes.find(THEMES[0]).ok_or("no such theme")?.look;
    let arabic = look.arabic_fonts.first().ok_or("no Arabic family")?;
    let (was, before) = (rig.hub.settings(), released());
    if rig.memory.calls() > 0 {
        return Err("free memory went back at idle".into());
    }
    rig.show(UiSettings {
        theme: THEMES[0].to_owned(),
        ..was.clone()
    });
    wait(
        app,
        "the wake loading the fonts, a window dropping its graphics and one give-back",
        || released() > before && font_families().contains(arabic) && rig.memory.calls() == 1,
    )?;
    rig.show(was);
    wait(
        app,
        "the wake dropping the fonts and one more give-back",
        || !font_families().contains(arabic) && rig.memory.calls() == 2,
    )
}

/// Shows theme `id` and loads its fonts; true when that changed what Qt holds.
fn switch(rig: &mut Rig, id: &str) -> bool {
    rig.show(UiSettings {
        theme: id.to_owned(),
        ..rig.hub.settings()
    });
    rig.hub.sync_fonts()
}

/// Qt's font list takes each theme's bundled fonts from the compiled-in resources and drops the others: after each
/// switch it has the theme's first Arabic family, which QML picks for Arabic text, and no bundled family the theme
/// does not name, unless Windows has it too.
pub(crate) fn fonts(rig: &mut Rig) -> Outcome {
    let themes = Themes::shipped().map_err(|e| e.to_string())?;
    let files = FontFiles::shipped().map_err(|e| e.to_string())?;
    let _ = switch(rig, THEMES[THEMES.len() - 1]);
    let windows = font_families();
    for id in THEMES {
        let look = &themes.find(id).ok_or("no such theme")?.look;
        let lists = [&look.latin_fonts, &look.arabic_fonts, &look.mark_fonts];
        let named = |f: &str| lists.iter().any(|l| l.iter().any(|n| n == f));
        if !switch(rig, id) {
            return Err(format!("{id}: the fonts did not change"));
        }
        let qt = font_families();
        let has = |f: &str| qt.iter().any(|q| q == f);
        let stray: Vec<&str> = files
            .families()
            .filter(|f| !named(f) && !windows.iter().any(|w| w == f) && has(f))
            .collect();
        let arabic = look.arabic_fonts.first().ok_or("no Arabic family")?;
        if !has(arabic) || !stray.is_empty() {
            return Err(format!("{id}: Qt lacks {arabic} or still holds {stray:?}"));
        }
    }
    match rig.lines.count(Level::WARN, NOT_LOADED) {
        0 => Ok(()),
        n => Err(format!("{n} bundled fonts did not load")),
    }
}

/// The app's End reaches a `Ui` object made after the last one closed, which hides the tray icon as it ends.
pub(crate) fn ends(app: &mut UniquePtr<QGuiApplication>, rig: &Rig) -> Outcome {
    rig.hub.end_ui();
    wait(app, "the UI ending", || rig.tray.shown().is_empty())
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
