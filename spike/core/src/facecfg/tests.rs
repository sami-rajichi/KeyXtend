//! The spike's key and bar config checks, the bar's icons and the font search.

use super::*;

fn cfg() -> crate::config::SpikeConfig {
    crate::config::load().expect("spike.toml loads")
}

fn board() -> BoardSpec {
    BoardSpec::shipped().expect("the product board loads")
}

fn icons() -> Icons {
    kx_theme::Themes::shipped().expect("the tokens load").icons
}

fn presets() -> usize {
    kx_mod_keyboard::KeyboardSettings::from_defaults()
        .expect("the product defaults load")
        .presets
        .len()
}

#[test]
fn every_named_and_side_key_has_a_look() {
    let c = cfg();
    assert_eq!(check_keys(&c.keys, &board(), &icons()), Ok(()));
    let mut keys = c.keys.clone();
    keys.remove("esc");
    let lost = check_keys(&keys, &board(), &icons());
    assert!(lost.is_err(), "Esc lost its look");
}

#[test]
fn a_key_drawing_nothing_is_refused() {
    let c = cfg();
    let mut keys = c.keys.clone();
    keys.get_mut("esc").expect("esc").label = None;
    let bare = check_keys(&keys, &board(), &icons());
    assert!(bare.is_err(), "Esc has no icon");
    let mut no_icon = icons();
    no_icon.keys.remove("mic");
    let bare = check_keys(&c.keys, &board(), &no_icon);
    assert!(bare.is_err(), "nor has the mic now");
}

#[test]
fn a_side_key_needs_an_action_and_a_note_needs_text() {
    let c = cfg();
    let mut keys = c.keys.clone();
    keys.get_mut("mic").expect("mic").action = None;
    assert!(check_keys(&keys, &board(), &icons()).is_err());
    let mut keys = c.keys.clone();
    let power = keys.get_mut("power").expect("power");
    power.action = Some(Action::Note);
    power.note = None;
    assert!(check_keys(&keys, &board(), &icons()).is_err());
}

#[test]
fn the_bar_takes_its_icons_from_icons_toml() {
    let (bar, i) = (cfg().bar, icons());
    let v = bar_json(&bar, &i);
    assert_eq!(v["theme"]["icon"], json!(i.bar.mode));
    assert_eq!(v["theme"]["icon_on"], json!(i.bar.mode_on));
    assert_eq!(v["dpad"][4]["icon"], json!(i.dpad.stop));
    assert_eq!(v["close"]["name"], json!(bar.close.name));
}

#[test]
fn size_labels_match_the_presets() {
    let c = cfg();
    assert_eq!(check_bar(&c.bar, presets()), Ok(()));
    assert!(check_bar(&c.bar, presets() + 1).is_err());
}

#[test]
fn minimise_never_looks_like_a_size_button() {
    let b = icons().bar;
    assert_ne!(b.minimise, b.smaller);
    assert_ne!(b.minimise, b.bigger);
    assert!(!b.bubble.is_empty() && !cfg().bar.bubble_title.is_empty());
}

#[test]
fn font_files_are_found_in_subfolders_by_extension() {
    let dir = std::env::temp_dir().join(format!("kx-fonts-{}", std::process::id()));
    let sub = dir.join("plex");
    std::fs::create_dir_all(&sub).expect("temp folder");
    for f in [sub.join("a.ttf"), dir.join("b.OTF"), dir.join("c.txt")] {
        std::fs::write(f, b"").expect("temp file");
    }
    let mut found = cfg().assets.font_files(&dir);
    found.sort();
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(found, [dir.join("b.OTF"), sub.join("a.ttf")]);
}

#[test]
fn a_folder_link_is_not_followed_so_a_loop_cannot_recurse() {
    let dir = std::env::temp_dir().join(format!("kx-loop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp folder");
    std::fs::write(dir.join("a.ttf"), b"").expect("temp file");
    // A junction needs no special rights, unlike a symbolic link.
    let made = std::process::Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(dir.join("back"))
        .arg(&dir)
        .output()
        .is_ok_and(|o| o.status.success());
    let found = cfg().assets.font_files(&dir);
    let _ = std::fs::remove_dir(dir.join("back"));
    let _ = std::fs::remove_dir_all(&dir);
    assert!(made, "the junction was made");
    assert_eq!(found, [dir.join("a.ttf")]);
}

#[test]
fn an_icon_is_a_file_named_after_it() {
    let c = cfg();
    let p = c.assets.icon(Path::new("base"), "mic");
    assert!(
        p.ends_with(
            Path::new(&c.assets.icons)
                .join("mic")
                .with_extension(&c.assets.icon_ext)
        )
    );
}
