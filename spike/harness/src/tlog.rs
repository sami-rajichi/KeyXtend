//! Reads the target-window log, whose format is in `spike_core::targetlog`, after the last start.

use spike_core::targetlog::{COMMENT, FOCUS_LOST, SEP, START};

/// Radix of the logged UTF-16 units.
const HEX: u32 = 16;

/// One logged UTF-16 unit and when it arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// QPC time in microseconds.
    pub us: i64,
    /// The UTF-16 code unit.
    pub unit: u16,
}

/// The units logged since the last start; bad lines are skipped.
pub fn parse(log: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    for line in log.lines() {
        if line.starts_with(START) {
            units.clear();
            continue;
        }
        if line.starts_with(COMMENT) {
            continue;
        }
        let mut parts = line.trim().split(SEP);
        let us = parts.next().and_then(|t| t.parse::<i64>().ok());
        let unit = parts.next().and_then(|u| u16::from_str_radix(u, HEX).ok());
        if let (Some(us), Some(unit)) = (us, unit) {
            units.push(Unit { us, unit });
        }
    }
    units
}

/// Focus losses logged since the last start, at or after `since_us`; one without a time counts too.
pub fn focus_lost(log: &str, since_us: i64) -> usize {
    let mut lost = 0;
    for line in log.lines() {
        if line.starts_with(START) {
            lost = 0;
        } else if let Some(rest) = line.strip_prefix(FOCUS_LOST) {
            let us = rest
                .strip_prefix(SEP)
                .and_then(|t| t.trim().parse::<i64>().ok());
            lost += usize::from(us.is_none_or(|us| us >= since_us));
        }
    }
    lost
}

/// The text the units spell; broken surrogates become U+FFFD.
pub fn decode(units: &[Unit]) -> String {
    let raw: Vec<u16> = units.iter().map(|u| u.unit).collect();
    String::from_utf16_lossy(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_units_after_the_last_start() {
        let log = "# start\t1\n10\t0041\n# start\t20\n30\t0644\n# focus-lost\t30\n31\t0627\n";
        let units = parse(log);
        assert_eq!(
            units,
            vec![
                Unit {
                    us: 30,
                    unit: 0x0644
                },
                Unit {
                    us: 31,
                    unit: 0x0627
                }
            ]
        );
        assert_eq!(decode(&units), "لا");
    }

    #[test]
    fn decodes_a_surrogate_pair_and_skips_bad_lines() {
        let log = "# start\t0\n5\tD83D\nnot a line\n6\tDE00\r\n7\t20AC\n8\tZZZZ\n";
        let units = parse(log);
        assert_eq!(units.len(), 3);
        assert_eq!(decode(&units), "\u{1F600}€");
    }

    #[test]
    fn counts_focus_losses_after_the_last_start_from_a_time() {
        let log = "# start\t0\n# focus-lost\t50\n# start\t100\n# focus-lost\t150\n\
                   200\t0041\n# focus-lost\t300\r\n# focus-lost\n";
        assert_eq!(focus_lost(log, 200), 2);
        assert_eq!(focus_lost(log, 0), 3);
        assert_eq!(focus_lost("# start\t0\n10\t0041\n", 0), 0);
    }
}
