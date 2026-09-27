//! The target-window log format, shared by its writer (target-window) and its reader (the harness).

/// Starts comment lines; data lines are `<µs>\t<UTF-16 unit in hex>`.
pub const COMMENT: &str = "#";
/// First line of every run: `# start\t<µs>`.
pub const START: &str = "# start";
/// The text box lost the keyboard focus: `# focus-lost\t<µs>`.
pub const FOCUS_LOST: &str = "# focus-lost";
/// Field separator.
pub const SEP: char = '\t';

/// A data line for UTF-16 `unit` received at `us`.
pub fn data(us: i64, unit: u16) -> String {
    format!("{us}{SEP}{unit:04X}")
}

/// A comment line `mark` at `us`.
pub fn mark(mark: &str, us: i64) -> String {
    format!("{mark}{SEP}{us}")
}

/// A mouse press: `# mouse\t<press>\t<µs>\t<x>\t<y>`, in screen pixels.
pub const MOUSE: &str = "# mouse";

/// The mouse events target-window logs.
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
    /// A context-menu request, which target-window swallows.
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
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// A mouse line for `press` at `us`, at screen point (`x`, `y`).
pub fn mouse(press: Press, us: i64, x: i32, y: i32) -> String {
    format!("{MOUSE}{SEP}{}{SEP}{us}{SEP}{x}{SEP}{y}", press.name())
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
    fn lines_have_the_documented_shape() {
        assert_eq!(data(12, 0x0644), "12\t0644");
        assert_eq!(mark(FOCUS_LOST, 7), "# focus-lost\t7");
        assert!(START.starts_with(COMMENT) && FOCUS_LOST.starts_with(COMMENT));
    }
}
