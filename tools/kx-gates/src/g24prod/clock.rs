//! The record bar's clock as a screen reader hears it, in every shipped language: it holds at 0:00 until the first
//! recorded frame and counts from it, so the gate stops a recording once it reads the length asked.
#![cfg(windows)]

use kx_module_api::ports::S_PER_MIN;

use crate::ftl;
use crate::p12cfg::SnipClock;

/// Qt's places in the time text for the minutes and the seconds.
const MIN_ARG: &str = "%1";
const SEC_ARG: &str = "%2";
/// The seconds' digits, as kx-ui's timer pads them.
const SEC_DIGITS: usize = 2;
/// Seconds in a minute.
const PER_MIN: u64 = S_PER_MIN.unsigned_abs();

/// Each shipped language's word, join and time text of the clock.
pub struct Clock(Vec<(String, String, String)>);

/// How far the clock was read: never, short of the time asked (a worker yet to record), or at that time or later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Read {
    Never,
    Short,
    Reached,
}

impl Clock {
    /// The clock's texts by the keys of `c`.
    pub fn new(c: &SnipClock) -> Result<Self, String> {
        let [word, join, time] = [&c.word, &c.join, &c.time].map(|k| ftl::every(k));
        let texts = word?.into_iter().zip(join?).zip(time?);
        Ok(Self(texts.map(|((w, j), t)| (w, j, t)).collect()))
    }

    /// The clock's name at `secs` in every shipped language.
    pub fn at(&self, secs: u64) -> Vec<String> {
        let each = |(w, j, t): &(String, String, String)| name((w, j, t), secs);
        self.0.iter().map(each).collect()
    }

    /// What `names` show of the clock against `secs`, a time up to `late` counting as reached.
    pub fn read(&self, names: &[String], secs: u64, late: u64) -> Read {
        let shows = |t: u64| self.at(t).iter().any(|n| names.contains(n));
        if (secs..=late).any(shows) {
            Read::Reached
        } else if (0..secs).any(shows) {
            Read::Short
        } else {
            Read::Never
        }
    }
}

/// The clock's name at `secs` from one language's word, join and time text: minutes, then two-digit seconds.
fn name((word, join, time): (&str, &str, &str), secs: u64) -> String {
    let (min, sec) = (secs / PER_MIN, secs % PER_MIN);
    let sec = format!("{sec:0SEC_DIGITS$}");
    let time = time
        .replacen(MIN_ARG, &min.to_string(), 1)
        .replacen(SEC_ARG, &sec, 1);
    format!("{word}{join}{time}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::load;

    /// kx-ui's record bar, which names its clock, and the timer that writes it.
    const RECORD: &str = include_str!("../../../../crates/kx-ui/qml/SnipRecord.qml");
    const TIMER: &str = include_str!("../../../../crates/kx-ui/qml/VoiceTimer.qml");

    #[test]
    fn the_clock_reads_its_word_then_minutes_and_two_digit_seconds() {
        assert_eq!(name(("Recording", " ", "%1:%2"), 5), "Recording 0:05");
        assert_eq!(
            name(("Enregistrement", " ", "%1:%2"), 75),
            "Enregistrement 1:15"
        );
        assert_eq!(name(("جارٍ التسجيل", " ", "%1:%2"), 30), "جارٍ التسجيل 0:30");
    }

    #[test]
    fn a_read_tells_a_clock_never_read_from_one_short_of_its_time() {
        let clock = Clock(vec![("Recording".into(), " ".into(), "%1:%2".into())]);
        let read = |names: &[&str]| {
            let names: Vec<String> = names.iter().map(ToString::to_string).collect();
            clock.read(&names, 5, 7)
        };
        assert_eq!(read(&[]), Read::Never);
        assert_eq!(read(&["Stop"]), Read::Never, "no clock among them");
        assert_eq!(
            read(&["Recording 0:00"]),
            Read::Short,
            "held before the first frame"
        );
        assert_eq!(read(&["Recording 0:04"]), Read::Short);
        assert_eq!(read(&["Recording 0:05"]), Read::Reached);
        assert_eq!(read(&["Recording 0:07"]), Read::Reached, "a missed poll");
        assert!(Read::Never < Read::Short && Read::Short < Read::Reached);
    }

    #[test]
    fn the_clock_is_named_in_every_shipped_language() {
        let clock = Clock::new(&load().expect("loads").g24.clock).expect("the texts");
        let names = clock.at(5);
        assert_eq!(names.len(), 3, "{names:?}");
        assert!(names.contains(&"Recording 0:05".to_string()), "{names:?}");
    }

    #[test]
    fn kx_ui_names_its_clock_as_the_gate_reads_it() {
        let c = load().expect("loads").g24.clock;
        let named = format!(
            "glow.res.tr(\"{}\") + glow.res.tr(\"{}\") + text",
            c.word, c.join
        );
        assert!(RECORD.contains(&named), "{named}");
        assert!(RECORD.contains(&format!("format: glow.res.tr(\"{}\")", c.time)));
        assert!(TIMER.contains(&format!("readonly property int digits: {SEC_DIGITS}")));
        assert!(TIMER.contains(&format!("readonly property int sPerMin: {PER_MIN}")));
    }
}
