//! Losses with the keyboard closed, minimised, or a theme picked as the windows close or wait to drop their graphics:
//! the windows made again leave the keyboard as the user left it and draw the theme's fonts, and memory goes back.

use crate::rebuild::{back, kept, lose, marks, rebuilt};
use crate::rig::{Outcome, Rig, wait};
use cxx::UniquePtr;
use cxx_qt_lib::QGuiApplication;
use kx_platform::ui::Command;
use kx_platform_fake::GuardCall;
use kx_theme::Themes;
use kx_ui::{Mode, Shell, UiSettings, font_families};

/// The theme picked while the windows close, then the one picked just before a loss.
const PICKED: &str = "dolch";
const WAITED: &str = "et66";

/// How many cuts and guard runs the guard took; only the shown keyboard cuts itself.
fn guarded(rig: &Rig) -> (usize, usize) {
    let calls = rig.guard.calls();
    let cuts = calls.iter().filter(|c| matches!(c, GuardCall::Shape(..)));
    let runs = calls
        .iter()
        .filter(|c| matches!(c, GuardCall::GuardOwn(..)));
    (cuts.count(), runs.count())
}

/// A loss while the keyboard is `mode`, as Close or Minimise keeps it: once a guard run of the windows made again
/// came, only a minimised keyboard placed itself, for its bubble, none showed, and the mode stayed. Then Show brings
/// it back.
fn left(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell, rig: &Rig, mode: Mode) -> Outcome {
    rig.hub.keep_mode(mode);
    let ((cuts, runs), before) = (guarded(rig), rig.face.placed().len());
    let bubble = mode == Mode::Minimised;
    lose(app, shell)?;
    rebuilt(shell)?;
    wait(app, "a guard run of the windows made again", || {
        rig.face.placed().len() == before + usize::from(bubble) && guarded(rig).1 > runs
    })?;
    if guarded(rig).0 != cuts || rig.hub.mode() != mode || rig.tray.shown().len() != 1 {
        return Err(format!(
            "{mode:?}: the keyboard did not stay as it was left"
        ));
    }
    if bubble {
        kept(rig)?;
    }
    rig.hub.command(Command::Show);
    wait(app, "Show bringing the keyboard back", || {
        guarded(rig).0 > cuts && rig.hub.mode() == Mode::Shown
    })
}

/// A loss after Close: the keyboard stays away while the banner made again shows, until Show.
pub(crate) fn away(app: &mut UniquePtr<QGuiApplication>, shell: &mut Shell, rig: &Rig) -> Outcome {
    left(app, shell, rig, Mode::Away)
}

/// A loss after Minimise: the keyboard is placed where it was and stays behind its bubble, until Show.
pub(crate) fn minimised(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &Rig,
) -> Outcome {
    left(app, shell, rig, Mode::Minimised)
}

/// A theme picked after a loss, whose wake the closing windows never run: the windows made again load its fonts
/// first, so Qt has its first Arabic family and nothing is left to load.
pub(crate) fn theme(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &mut Rig,
) -> Outcome {
    let themes = Themes::shipped().map_err(|e| e.to_string())?;
    let look = &themes.find(PICKED).ok_or("no such theme")?.look;
    let arabic = look.arabic_fonts.first().ok_or("no Arabic family")?;
    let was = marks(rig);
    lose(app, shell)?;
    rig.show(UiSettings {
        theme: PICKED.to_owned(),
        ..rig.hub.settings()
    });
    rebuilt(shell)?;
    back(app, rig, was)?;
    if rig.hub.sync_fonts() || !font_families().contains(arabic) {
        return Err(format!(
            "the windows made again did not load {PICKED}'s fonts first"
        ));
    }
    Ok(())
}

/// A theme picked just before a loss: the windows that wait to drop their graphics after it go before their frame, and
/// the free memory still goes back once.
pub(crate) fn waiting(
    app: &mut UniquePtr<QGuiApplication>,
    shell: &mut Shell,
    rig: &mut Rig,
) -> Outcome {
    let (was, calls) = (marks(rig), rig.memory.calls());
    rig.show(UiSettings {
        theme: WAITED.to_owned(),
        ..rig.hub.settings()
    });
    lose(app, shell)?;
    rebuilt(shell)?;
    back(app, rig, was)?;
    wait(
        app,
        "one give-back after the windows that waited went",
        || rig.memory.calls() == calls + 1,
    )
}
