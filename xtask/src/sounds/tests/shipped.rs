//! The shipped sounds a recipe makes, checked against their render.

use super::*;

/// Every shipped sound a recipe of the same name makes is that recipe's render, byte for byte, so a recipe change
/// shows until the file is rendered again; the dictation's start and stop sounds are made so.
#[test]
fn a_shipped_sound_made_from_a_recipe_is_its_render() {
    let book = parse(SHIPPED, &shipped_format()).unwrap();
    let sounds = kx_theme::sounds::Sounds::parse(&shipped_format()).unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/sounds");
    let mut made = Vec::new();
    for file in sounds.files() {
        let stem = file.strip_suffix(&format!(".{EXT}")).unwrap();
        let Some(recipe) = book.recipes.iter().find(|r| r.stems() == [stem]) else {
            continue;
        };
        let sound = render_clip(&book, recipe, stem).unwrap();
        assert_eq!(
            std::fs::read(dir.join(file)).unwrap(),
            sound.bytes,
            "{file}"
        );
        made.push(stem);
    }
    assert_eq!(
        made,
        ["voice-start", "voice-stop"],
        "the files recipes make"
    );
}
