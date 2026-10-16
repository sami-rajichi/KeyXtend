//! The log format of the test window, shared by its writer (the window) and its reader (the gate runner).

/// Starts comment lines; data lines are `<µs>\t<UTF-16 unit in hex>`.
pub const COMMENT: &str = "#";
/// First line of every run: `# start\t<µs>`.
pub const START: &str = "# start";
/// The text box lost the keyboard focus: `# focus-lost\t<µs>`.
pub const FOCUS_LOST: &str = "# focus-lost";
/// The layout from before `--layout` did not come back at close: `# layout-kept\t<µs>`.
pub const LAYOUT_KEPT: &str = "# layout-kept";
/// Field separator.
pub const SEP: char = '\t';

/// A data line for UTF-16 `unit` received at `us`.
#[must_use]
pub fn data(us: i64, unit: u16) -> String {
    format!("{us}{SEP}{unit:04X}")
}

/// A comment line `mark` at `us`.
#[must_use]
pub fn mark(mark: &str, us: i64) -> String {
    format!("{mark}{SEP}{us}")
}

/// A key went down: `# key\t<µs>\t<virtual key>\t<scan code>`, both in hex; an extended key's scan code is `E0xx`.
pub const KEY: &str = "# key";

/// A key line for virtual key `vk` with scan code `scan`, going down at `us`.
#[must_use]
pub fn key(us: i64, vk: u16, scan: u16) -> String {
    format!("{KEY}{SEP}{us}{SEP}{vk:02X}{SEP}{scan:04X}")
}

/// A mouse press: `# mouse\t<press>\t<µs>\t<x>\t<y>`, in screen pixels.
pub const MOUSE: &str = "# mouse";

/// The mouse events the window logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// The left button went down.
    LeftDown,
    /// The left button came up.
    LeftUp,
    /// The right button went down.
    RightDown,
    /// The right button came up.
    RightUp,
    /// A context-menu request, which the window swallows.
    Menu,
}

impl Press {
    /// Every press, for tests and parsing.
    pub const ALL: [Press; 5] = [
        Self::LeftDown,
        Self::LeftUp,
        Self::RightDown,
        Self::RightUp,
        Self::Menu,
    ];

    /// The name used in the log.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::LeftDown => "ldown",
            Self::LeftUp => "lup",
            Self::RightDown => "rdown",
            Self::RightUp => "rup",
            Self::Menu => "menu",
        }
    }

    /// The press called `name`.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// A mouse line for `press` at `us`, at screen point (`x`, `y`).
#[must_use]
pub fn mouse(press: Press, us: i64, x: i32, y: i32) -> String {
    format!("{MOUSE}{SEP}{}{SEP}{us}{SEP}{x}{SEP}{y}", press.name())
}

/// A wheel turn the box received: `# wheel\t<µs>\t<axis>\t<delta>\t<x>\t<y>`, with the point the message carried as it
/// came; the axis is `UPRIGHT` or `SIDEWAYS`.
pub const WHEEL: &str = "# wheel";
/// The axis of the upright wheel.
pub const UPRIGHT: &str = "v";
/// The axis of the sideways wheel.
pub const SIDEWAYS: &str = "h";
/// A gate's hang began: `# hung\t<µs>`.
pub const HUNG: &str = "# hung";
/// The window woke from a gate's hang and reads its messages again: `# woke\t<µs>`.
pub const WOKE: &str = "# woke";

/// A wheel line for a turn of `delta` at `us`, `sideways` or upright, carrying point (`x`, `y`).
#[must_use]
pub fn wheel(us: i64, sideways: bool, delta: i32, x: i32, y: i32) -> String {
    let axis = if sideways { SIDEWAYS } else { UPRIGHT };
    format!("{WHEEL}{SEP}{us}{SEP}{axis}{SEP}{delta}{SEP}{x}{SEP}{y}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_lines_have_the_documented_shape_and_read_back() {
        assert_eq!(mouse(Press::RightUp, 7, 10, -3), "# mouse\trup\t7\t10\t-3");
        for p in Press::ALL {
            assert_eq!(Press::parse(p.name()), Some(p));
        }
        assert_eq!(Press::parse("wheel"), None);
        assert!(MOUSE.starts_with(COMMENT));
    }

    #[test]
    fn key_lines_have_the_documented_shape() {
        assert_eq!(key(7, 0x0D, 0xE01C), "# key\t7\t0D\tE01C");
        assert_eq!(key(8, 0xA0, 0x2A), "# key\t8\tA0\t002A");
        assert!(KEY.starts_with(COMMENT));
    }

    #[test]
    fn wheel_lines_have_the_documented_shape() {
        assert_eq!(wheel(7, false, -40, 10, -3), "# wheel\t7\tv\t-40\t10\t-3");
        assert_eq!(wheel(8, true, 120, 0, 5), "# wheel\t8\th\t120\t0\t5");
        assert!(
            WHEEL.starts_with(COMMENT) && HUNG.starts_with(COMMENT) && WOKE.starts_with(COMMENT)
        );
        assert_eq!(mark(WOKE, 9), "# woke\t9");
    }

    #[test]
    fn lines_have_the_documented_shape() {
        assert_eq!(data(12, 0x0644), "12\t0644");
        assert_eq!(mark(FOCUS_LOST, 7), "# focus-lost\t7");
        assert!(START.starts_with(COMMENT) && FOCUS_LOST.starts_with(COMMENT));
        assert!(LAYOUT_KEPT.starts_with(COMMENT));
    }
}
