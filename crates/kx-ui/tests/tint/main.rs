//! The icon provider fills an icon's ink with its tint and keeps every pixel's alpha, serves each icon the tokens name
//! from the compiled-in resources, refuses any other file with one warning and a clear image, gives a zero size one
//! clear pixel, and keeps no texture per step of a fade. Qt wants its app object on the main thread, so this binary
//! runs without the test harness. It reads probes only debug builds have, so a release build of it checks nothing.

#[cfg(debug_assertions)]
mod fade;
#[cfg(debug_assertions)]
#[path = "../hub/logs.rs"]
mod logs;

use std::process::ExitCode;

/// The side the tests ask icons at, in pixels: 20 at 125 %.
#[cfg(debug_assertions)]
const SIDE: i32 = 25;
/// A tint as `Icon.qml` names it, and its channels.
#[cfg(debug_assertions)]
const TINT: (&str, [u32; 3]) = ("005fb8", [0x00, 0x5f, 0xb8]);

/// What a check found wrong.
#[cfg(debug_assertions)]
type Outcome = Result<(), String>;

#[cfg(debug_assertions)]
fn main() -> ExitCode {
    let _app = cxx_qt_lib::QGuiApplication::new();
    let checks: [fn() -> Outcome; 5] = [
        ink_takes_the_tint,
        named_icons_load,
        others_are_refused,
        a_zero_size_is_clear_and_quiet,
        a_fade_keeps_one_texture,
    ];
    match checks.iter().try_for_each(|check| check()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(debug_assertions))]
fn main() -> ExitCode {
    eprintln!("the tint test needs a debug build's probes, so it checks nothing here");
    ExitCode::SUCCESS
}

/// Each icon name the tokens give.
#[cfg(debug_assertions)]
fn names() -> Result<Vec<String>, String> {
    let themes = kx_theme::Themes::shipped().map_err(|e| e.to_string())?;
    Ok(themes
        .icons
        .names()
        .into_iter()
        .map(str::to_owned)
        .collect())
}

/// Icon `name`'s file.
#[cfg(debug_assertions)]
fn file(name: &str) -> String {
    format!("{name}.{}", kx_theme::assets::ICON_EXT)
}

/// A word's alpha and colour channels, premultiplied.
#[cfg(debug_assertions)]
fn channels(word: u32) -> (u32, [u32; 3]) {
    (
        word >> 24,
        [(word >> 16) & 0xff, (word >> 8) & 0xff, word & 0xff],
    )
}

/// What the provider serves for `id` at `side`, and the warnings it logs meanwhile.
#[cfg(debug_assertions)]
fn served_at(id: &str, side: i32) -> (Vec<u32>, Vec<String>) {
    let mut got = Vec::new();
    let warned = logs::warnings_of(|| {
        got = kx_ui::icon_pixels(id, side);
    });
    (got, warned)
}

/// What the provider serves for `id` at `SIDE`, and the warnings it logs meanwhile.
#[cfg(debug_assertions)]
fn served(id: &str) -> (Vec<u32>, Vec<String>) {
    served_at(id, SIDE)
}

/// Every pixel keeps the file's alpha, and where it has ink, its colour is the tint at that alpha, to one step.
#[cfg(debug_assertions)]
fn ink_takes_the_tint() -> Outcome {
    let names = names()?;
    let name = names.first().ok_or("the tokens name no icon")?;
    let mask = kx_ui::icon_mask(&file(name), SIDE);
    let got = kx_ui::icon_pixels(&format!("{}/{}", TINT.0, file(name)), SIDE);
    if mask.len() != got.len()
        || !mask.iter().any(|w| w >> 24 > 0)
        || !mask.iter().any(|w| w >> 24 == 0)
    {
        return Err(format!(
            "{name} gave {} pixels, its file {} with ink and clear ones",
            got.len(),
            mask.len()
        ));
    }
    for (at, (m, g)) in mask.iter().zip(&got).enumerate() {
        let (alpha, rgb) = channels(*g);
        let wanted = TINT.1.map(|c| c * alpha / 0xff);
        let near = rgb.iter().zip(wanted).all(|(c, w)| c.abs_diff(w) <= 1);
        if alpha != m >> 24 || !near {
            return Err(format!("{name} pixel {at} is {g:08x}, its file's {m:08x}"));
        }
    }
    Ok(())
}

/// Every icon the tokens name loads from the resources with ink and no warning.
#[cfg(debug_assertions)]
fn named_icons_load() -> Outcome {
    for name in names()? {
        let (got, warned) = served(&format!("{}/{}", TINT.0, file(&name)));
        if !warned.is_empty() || !got.iter().any(|w| w >> 24 > 0) {
            return Err(format!(
                "{name} did not load from the resources: {warned:?}"
            ));
        }
    }
    Ok(())
}

/// An id outside the icons folder, of another type, with a bad tint or with no file gives a clear image and one
/// warning (edge case 8).
#[cfg(debug_assertions)]
fn others_are_refused() -> Outcome {
    let names = names()?;
    let name = names.first().ok_or("the tokens name no icon")?;
    let tint = TINT.0;
    let ids = [
        format!("{tint}/../{}", file(name)),
        format!("{tint}/..\\{}", file(name)),
        format!("{tint}/lucide/{}", file(name)),
        format!("{tint}/C:{}", file(name)),
        format!("{tint}/%2e%2e%2f{}", file(name)),
        format!("{tint}/.{}", file(name)),
        format!("{tint}/{name}.png"),
        format!("{tint}/{}", file("no-such-icon")),
        format!("zzzzzz/{}", file(name)),
        format!("ff{tint}/{}", file(name)),
        format!("red/{}", file(name)),
        file(name),
    ];
    for id in ids {
        let (got, warned) = served(&id);
        let clear = got.len() == usize::try_from(SIDE * SIDE).unwrap_or_default()
            && got.iter().all(|w| *w == 0);
        if warned.len() != 1 || !clear {
            return Err(format!(
                "{id} was not refused once with a clear image: {warned:?}"
            ));
        }
    }
    Ok(())
}

/// A zero size reads no file: a named icon is one clear pixel with no warning, and a refused one still warns once.
#[cfg(debug_assertions)]
fn a_zero_size_is_clear_and_quiet() -> Outcome {
    let names = names()?;
    let name = names.first().ok_or("the tokens name no icon")?;
    let (got, warned) = served_at(&format!("{}/{}", TINT.0, file(name)), 0);
    let (refused, said) = served_at(&file(name), 0);
    if got != [0] || !warned.is_empty() || refused != [0] || said.len() != 1 {
        return Err(format!(
            "at size 0, {name} gave {got:?} with {warned:?}, a refused id {refused:?} with {said:?}"
        ));
    }
    Ok(())
}

/// A fade's tint steps leave one texture alive.
#[cfg(debug_assertions)]
fn a_fade_keeps_one_texture() -> Outcome {
    let names = names()?;
    let name = names.first().ok_or("the tokens name no icon")?;
    fade::a_fade_keeps_one_texture(name, SIDE)
}
