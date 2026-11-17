//! The amber Preparing light against the shipped `themes.toml`: each palette takes the approved mock-up's colours
//! (design/voice-bar.html lines 86-117), Dolch light's ink nudged for AA under the press shade, and it is never the red
//! recording light.

use crate::Themes;
use crate::mode::Mode;
use crate::rgba::Rgba;

/// Each theme's `prep`, `prep_ink` and sculpted skirt (empty when flat), light then dark, as the mock-up gives them.
const MOCK_UP: [(&str, [[&str; 3]; 2]); 3] = [
    (
        "native",
        [["#FFB900", "#1B1B1B", ""], ["#F2B84B", "#2B1D04", ""]],
    ),
    (
        "et66",
        [["#E58A2E", "#1D1B18", ""], ["#E0913A", "#2A1D0E", ""]],
    ),
    (
        "dolch",
        [
            ["#E8A33C", "#333535", "#BF8124"],
            ["#D9952F", "#2A2C2E", "#A9701D"],
        ],
    ),
];

/// The shipped themes.
fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

fn rgba(hex: &str) -> Rgba {
    Rgba::parse(hex).expect(hex)
}

#[test]
fn every_palette_takes_the_mock_ups_amber() {
    let all = load();
    assert_eq!(all.list.len(), MOCK_UP.len(), "every theme is listed");
    for (id, modes) in MOCK_UP {
        let t = all.list.iter().find(|t| t.look.id == id).expect(id);
        for (mode, [prep, ink, skirt]) in Mode::ALL.into_iter().zip(modes) {
            let (p, at) = (t.palette(mode), format!("{id}.{}", mode.key()));
            assert_eq!((p.prep, p.prep_ink), (rgba(prep), rgba(ink)), "{at}");
            let want = (!skirt.is_empty()).then(|| rgba(skirt));
            assert_eq!(p.skirt.as_ref().map(|s| s.prep), want, "{at}");
        }
    }
}

#[test]
fn the_amber_light_is_never_the_red_one() {
    for t in &load().list {
        for mode in Mode::ALL {
            let p = t.palette(mode);
            assert_ne!(p.prep, p.rec, "{}.{}", t.look.id, mode.key());
        }
    }
}
