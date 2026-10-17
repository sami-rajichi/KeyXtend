//! The scroll pad on screen: where each arrow and Stop take a tap, from the pad's box and kx-theme's `[dpad]` shape, as
//! kx-ui's DPad.qml draws them. An arrow's tap point is its icon's centre, inside the upright box readers find it by.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use windows::Win32::Foundation::{POINT, RECT};

/// Each arrow's quarter before the pad's turn, as signs of x and y around the centre (DPad.qml `arrows`).
const QUARTERS: [(Dir, [f64; 2]); 4] = [
    (Dir::Up, [-1.0, -1.0]),
    (Dir::Right, [1.0, -1.0]),
    (Dir::Left, [-1.0, 1.0]),
    (Dir::Down, [1.0, 1.0]),
];

/// The `[dpad]` shape values a tap point needs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shape {
    /// The hub's inset, as a share of the pad.
    pub hub: f64,
    /// Where each arrow sits across the ring, from the hub's edge (0) to the pad's edge (1).
    pub reach: f64,
    /// The arrows' turn, in degrees.
    pub turn_deg: f64,
}

impl Shape {
    /// The shipped tokens' shape.
    pub fn shipped() -> Result<Self, String> {
        let themes = kx_theme::Themes::shipped().map_err(|e| format!("the shipped tokens: {e}"))?;
        let d = &themes.shape.dpad;
        Ok(Self {
            hub: f64::from(d.hub),
            reach: f64::from(d.reach),
            turn_deg: f64::from(d.turn_deg),
        })
    }
}

/// The pad's disc on screen, in physical px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pad {
    x: f64,
    y: f64,
    r: f64,
    shape: Shape,
}

impl Pad {
    /// The pad drawn in box `r` with `shape`; none for a box with no area.
    pub fn of(r: RECT, shape: Shape) -> Option<Self> {
        let (w, h) = (f64::from(r.right - r.left), f64::from(r.bottom - r.top));
        (w > 0.0 && h > 0.0).then(|| Self {
            x: f64::from(r.left) + w / 2.0,
            y: f64::from(r.top) + h / 2.0,
            r: w / 2.0,
            shape,
        })
    }

    /// The pad's centre: its hub, and Stop while it scrolls.
    pub fn centre(&self) -> POINT {
        at(self.x, self.y)
    }

    /// Where the arrow towards `dir` takes a tap: on its quarter's middle line, `reach` of the way across the ring.
    pub fn arrow(&self, dir: Dir) -> POINT {
        let [qx, qy] = QUARTERS
            .iter()
            .find(|(d, _)| *d == dir)
            .map_or([0.0, 0.0], |(_, q)| *q);
        let hub = self.r * (1.0 - 2.0 * self.shape.hub);
        let out = (hub + self.shape.reach * (self.r - hub)) * std::f64::consts::FRAC_1_SQRT_2;
        // Qt turns clockwise on screen, where y grows downwards.
        let (sin, cos) = self.shape.turn_deg.to_radians().sin_cos();
        let (dx, dy) = (qx * cos - qy * sin, qx * sin + qy * cos);
        at(self.x + dx * out, self.y + dy * out)
    }

    /// The distance from the centre to `p`, and the hub's and the pad's radius, for checks.
    #[cfg(test)]
    fn rings(&self, p: POINT) -> (f64, f64, f64) {
        let d = (f64::from(p.x) - self.x).hypot(f64::from(p.y) - self.y);
        (d, self.r * (1.0 - 2.0 * self.shape.hub), self.r)
    }
}

/// The screen pixel nearest (`x`, `y`).
#[allow(clippy::cast_possible_truncation, reason = "Screen points fit in i32.")]
fn at(x: f64, y: f64) -> POINT {
    POINT {
        x: x.round() as i32,
        y: y.round() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A workspace path from this file; a moved file breaks the build here, never silently.
    /// The pad's arrows and their quarters.
    const DPAD_QML: &str = include_str!("../../../crates/kx-ui/qml/DPad.qml");

    /// A 100 px pad at (100, 200).
    const BOX: RECT = RECT {
        left: 100,
        top: 200,
        right: 200,
        bottom: 300,
    };

    fn shape(turn_deg: f64) -> Shape {
        Shape {
            hub: 0.34,
            reach: 0.5,
            turn_deg,
        }
    }

    fn pt(x: i32, y: i32) -> POINT {
        POINT { x, y }
    }

    #[test]
    fn each_arrow_is_tapped_on_its_own_side_halfway_across_the_ring() {
        let pad = Pad::of(BOX, shape(45.0)).expect("a pad");
        assert_eq!(pad.centre(), pt(150, 250));
        // The hub's edge is 16 px out and the pad's 50, so halfway is 33 px out.
        assert_eq!(pad.arrow(Dir::Up), pt(150, 217));
        assert_eq!(pad.arrow(Dir::Down), pt(150, 283));
        assert_eq!(pad.arrow(Dir::Left), pt(117, 250));
        assert_eq!(pad.arrow(Dir::Right), pt(183, 250));
    }

    #[test]
    fn the_turn_turns_the_tap_points_with_the_arrows() {
        let pad = Pad::of(BOX, shape(0.0)).expect("a pad");
        assert_eq!(pad.arrow(Dir::Up), pt(127, 227), "up-left, unturned");
        assert_eq!(pad.arrow(Dir::Down), pt(173, 273));
    }

    #[test]
    fn with_the_shipped_shape_every_tap_lies_on_the_ring_past_the_hub() {
        let pad = Pad::of(BOX, Shape::shipped().expect("tokens")).expect("a pad");
        for dir in Dir::ALL {
            let (d, hub, r) = pad.rings(pad.arrow(dir));
            assert!(hub < d && d < r, "{dir:?}: {d} not in {hub}..{r}");
        }
    }

    #[test]
    fn a_box_with_no_area_is_no_pad() {
        let flat = RECT {
            bottom: BOX.top,
            ..BOX
        };
        assert!(Pad::of(flat, shape(45.0)).is_none());
    }

    #[test]
    fn the_quarters_are_the_ones_dpad_qml_draws() {
        for (dir, [x, y]) in QUARTERS {
            let name = serde_json::to_value(dir).expect("named");
            #[allow(clippy::cast_possible_truncation, reason = "Signs only.")]
            let line = format!("{{ id: {name}, q: [{}, {}] }}", x as i32, y as i32);
            assert!(DPAD_QML.contains(&line), "{line}");
        }
    }
}
