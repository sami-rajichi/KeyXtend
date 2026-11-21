//! Two checks the local models' and the services' tables share: a plain day from TOML, and one line of visible text.

use kx_module_api::is_hidden;
use kx_module_api::ports::Date;
use toml::value::Datetime;

/// The plain day `when` names; `None` for a time or an offset, or a day that does not exist.
pub(crate) fn day(when: &Datetime) -> Option<Date> {
    let date = when
        .date
        .filter(|_| when.time.is_none() && when.offset.is_none())?;
    Date::new(date.year, date.month, date.day)
}

/// True for text that is not empty and holds no hidden mark.
pub(crate) fn one_line(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(is_hidden)
}
