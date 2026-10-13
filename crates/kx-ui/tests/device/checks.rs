//! The device test's steps without a loss, each over the running app's windows.

use crate::rebuild::watched;
use crate::rig::{FIRST, GUARDED, Outcome, Rig, TRAY_AT, ticks, wait};
use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;
use kx_theme::Themes;
use kx_theme::assets::FontFiles;
use kx_ui::i18n::keys::WINDOW_KEYBOARD;
use kx_ui::{
    ADAPTER_VAR, Device, FrameLog, Shell, UiSettings, draw_window, drawing, font_families,
    mid_frame_draws, pin_adapter, qt_sees, released, released_hidden, wakes,
};
use std::thread;
use tracing::Level;

/// The themes the font step shows in turn: two with bundled fonts, then the one Windows draws, which loads none. The
/// look steps switch between the first two.
const THEMES: [&str; 3] = ["dolch", "et66", "native"];
/// Ticks a step runs to see that nothing happened meanwhile: many frames long.
const QUIET: usize = 10;
/// What a font that did not load logs, and what each warning of a device that could not be shared holds.
const NOT_LOADED: &str = "did not load";
const GRAPHICS: &str = "graphics";
/// What a refused icon warns.
const REFUSED: &str = "icon was refused";
/// A title none of our windows has.
const NO_WINDOW: &str = "no such window";
/// What the frame log logs for each frame.
const FRAME_LINE: &str = "frame timed";

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

/// The soft shadows took their materials through the app's adopter since it had adopted `was`: each engine, a rebuilt
/// one too, serves it, as a missing one would only warn in QML and leak again.
pub(crate) fn adopted(was: usize) -> Outcome {
    let now = kx_ui::adopted();
    if now > was {
        Ok(())
    } else {
        Err(format!(
            "the adopter took {now} materials, not more than {was}"
        ))
    }
}

/// The keyboard and its shadow, then the language list and its shadow, which a Loader makes later, all draw on the
/// shared device, and every frame ends on the GUI thread; each has its scene graph errors heard, the icons come from
/// the provider, the soft shadows from the adopter, and no device is made off that thread, where the pin stays.
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
    adopted(0)?;
    drawn_now(rig)?;
    frames_logged(rig)?;
    elsewhere(rig)
}

/// While the frame log runs, a frame drawn logs its line; once it stops, none does.
fn frames_logged(rig: &Rig) -> Outcome {
    let title = rig.hub.i18n().text(WINDOW_KEYBOARD);
    let log = FrameLog::start().ok_or("the frame log did not start with drawing pinned")?;
    let before = rig.lines.count(Level::INFO, FRAME_LINE);
    let drawn = draw_window(&title);
    let logged = rig.lines.count(Level::INFO, FRAME_LINE);
    drop(log);
    let again = draw_window(&title);
    let quiet = rig.lines.count(Level::INFO, FRAME_LINE) == logged;
    if drawn && again && logged > before && quiet {
        return Ok(());
    }
    Err("the frame log missed a frame while it ran, or logged one after it stopped".into())
}

/// A press frame drawn at once (`press_frame = "now"`) finds the shown keyboard by its title, and no window by another;
/// asked again from inside that frame, it refuses, so no frame starts within another.
fn drawn_now(rig: &Rig) -> Outcome {
    let title = rig.hub.i18n().text(WINDOW_KEYBOARD);
    if !draw_window(&title) || draw_window(NO_WINDOW) {
        return Err(
            "the keyboard was not drawn at once by its title, or a window by a title none has"
                .into(),
        );
    }
    match mid_frame_draws(&title).as_slice() {
        [false] => Ok(()),
        answers => Err(format!(
            "asked from inside a frame, drawing at once answered {answers:?}"
        )),
    }
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
/// pages go back once; going back drops those fonts and gives back once more. Nothing went back at idle before, and a
/// wake that keeps the look drops and gives back nothing.
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
    )?;
    kept_look(app, rig)
}

/// A wake that leaves the look as it was, as the same settings handed again, asks no window to drop its graphics, and
/// a few ticks later none did and nothing was given back.
fn kept_look(app: &mut UniquePtr<QGuiApplication>, rig: &mut Rig) -> Outcome {
    let (handled, relooks) = wakes();
    let (calls, dropped) = (rig.memory.calls(), released());
    rig.show(rig.hub.settings());
    wait(app, "the wake of the same settings", || wakes().0 > handled)?;
    ticks(app, QUIET);
    let asked = wakes().1.saturating_sub(relooks);
    let gave = rig.memory.calls().saturating_sub(calls);
    let freed = released().saturating_sub(dropped);
    if asked == 0 && gave == 0 && freed == 0 {
        return Ok(());
    }
    Err(format!(
        "a wake that kept the look asked {asked} times for a release; {freed} windows released, {gave} give-backs ran"
    ))
}

/// A window that hides before it draws a new look still drops its graphics, and the free pages still go back once:
/// a theme lands while the tray menu is open, and in the same tick another program comes to the front, which shuts it.
pub(crate) fn hid(app: &mut UniquePtr<QGuiApplication>, rig: &mut Rig) -> Outcome {
    let was = rig.hub.settings();
    rig.show(UiSettings {
        tray_grace_ms: 0,
        ..was.clone()
    });
    let shown = drawing().0;
    rig.tray.click(TRAY_AT);
    wait(app, "the tray menu's first frame", || drawing().0 > shown)?;
    let (calls, hidden) = (rig.memory.calls(), released_hidden());
    rig.show(UiSettings {
        theme: THEMES[1].to_owned(),
        ..rig.hub.settings()
    });
    rig.tray.away();
    wait(
        app,
        "the menu dropping its graphics while hidden, and one give-back",
        || released_hidden() > hidden && rig.memory.calls() == calls + 1,
    )?;
    settled(app, rig, calls + 1)?;
    rig.show(was);
    wait(app, "the first theme back, and one more give-back", || {
        rig.memory.calls() == calls + 2
    })?;
    settled(app, rig, calls + 2)
}

/// A few ticks later the give-backs still number `calls`, so each look change gave back once, and the app is quiet.
fn settled(app: &mut UniquePtr<QGuiApplication>, rig: &Rig, calls: usize) -> Outcome {
    ticks(app, QUIET);
    match rig.memory.calls() {
        n if n == calls => Ok(()),
        n => Err(format!("{n} give-backs ran where {calls} were due")),
    }
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
