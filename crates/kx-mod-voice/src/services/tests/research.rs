//! The one dated research cross-check (final review I2): the shipped rows as their pages read 2026-11-17 give them
//! (those read again, 2026-11-21), each fact under the research file's lettered line. Only these files pin dated
//! facts, so a check-day refresh edits them and the tables; every other test reads the table.

use super::*;
use kx_module_api::ports::Fact;

mod assembly;
mod codes;
mod eleven;
mod gemini;
mod groq;
mod rows;

/// `fact`'s text id and its values by name.
fn values(fact: &Fact) -> (&str, Vec<(&str, &FactValue)>) {
    let args = fact.args.iter().map(|a| (a.name.as_str(), &a.value));
    (fact.id.as_str(), args.collect())
}

/// `cents` in dollars, as a fact holds money.
fn usd(cents: u32) -> FactValue {
    FactValue::Money {
        amount: Hundredths(cents),
        currency: "USD".into(),
    }
}

/// The first reading's day, and check day's for the rows read again then.
const FIRST: (u16, u8, u8) = (2026, 11, 17);
const AGAIN: (u16, u8, u8) = (2026, 11, 21);
/// The rows whose pages were read again on check day; Gemini's waits for its codes' hand check.
const REREAD: [&str; 4] = ["groq", "lemonfox", "elevenlabs", "assemblyai"];

#[test]
fn the_rows_read_again_on_check_day_say_so_and_the_rest_keep_the_first_reading() {
    for row in shipped().rows() {
        let again = REREAD.contains(&row.id.as_str());
        let (y, m, d) = if again { AGAIN } else { FIRST };
        assert_eq!(row.facts.read, Date::new(y, m, d).unwrap(), "{}", row.id);
    }
}

#[test]
fn the_cards_prices_cover_15_and_60_hours_of_sound_a_month() {
    let hours = (shipped().light_hours(), shipped().heavy_hours());
    assert_eq!(hours, (15, 60), "light and heavy use");
}
