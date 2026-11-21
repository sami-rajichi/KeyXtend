//! The one dated research cross-check (final review I2): the shipped rows as their pages read 2026-11-17 give them
//! (`AssemblyAI`'s 2026-11-19), each fact under the research file's lettered line. Only these files pin dated facts,
//! so a check-day refresh edits them and the tables; every other test reads the table.

use super::*;
use kx_module_api::ports::Fact;

mod assembly;
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
