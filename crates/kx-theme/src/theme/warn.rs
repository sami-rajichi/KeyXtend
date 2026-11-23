//! The warning colours against the shipped `themes/*.toml` (ADR-0047's cards and key page): each palette takes the
//! approved mock-up's pair (design/voice-bar.html lines 86-117), Native's being Fluent's Caution pair, and the warning is
//! never the panel itself.

use crate::Themes;
use crate::mode::Mode;
use crate::rgba::Rgba;

/// Each theme's `warn` and `warn_ink`, light then dark, as the mock-up gives them.
const MOCK_UP: [(&str, [[&str; 2]; 2]); 3] = [
    ("native", [["#FFF4CE", "#9D5D00"], ["#433519", "#FCE100"]]),
    ("et66", [["#F7E6C4", "#8A5400"], ["#4A3C24", "#F0BE5A"]]),
    ("dolch", [["#F1DFB8", "#865000"], ["#4B3E26", "#EDB956"]]),
];

/// The shipped themes.
fn load() -> Themes {
    Themes::shipped().expect("tokens load")
}

fn rgba(hex: &str) -> Rgba {
    Rgba::parse(hex).expect(hex)
}

#[test]
fn every_palette_takes_the_mock_ups_warning_pair() {
    let all = load();
    assert_eq!(all.list.len(), MOCK_UP.len(), "every theme is listed");
    for (id, modes) in MOCK_UP {
        let t = all.list.iter().find(|t| t.look.id == id).expect(id);
        for (mode, [warn, ink]) in Mode::ALL.into_iter().zip(modes) {
            let (p, at) = (t.palette(mode), format!("{id}.{}", mode.key()));
            assert_eq!((p.warn, p.warn_ink), (rgba(warn), rgba(ink)), "{at}");
            assert_ne!(p.warn, p.pop_bg, "{at}: a warning stands out of its panel");
        }
    }
}
