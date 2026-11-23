//! The `[voice]` writes of a cloud act, as the tests lay them over the settings and read their keys.

use super::super::super::Do;
use super::super::super::rig::Rig;

/// Applies the `[voice]` writes in `dos`, as the settings would after the actor wrote them.
pub(in super::super) fn apply(rig: &mut Rig, dos: &[Do]) {
    for done in dos {
        match done {
            Do::Set(key, value) => rig.settings = rig.settings.with(key, value.clone()).unwrap(),
            Do::SetAll(values) => rig.settings = rig.settings.with_all(values).unwrap(),
            _ => {}
        }
    }
}

/// Writes `text` as `[voice]` table `key`.
pub(in super::super) fn put(rig: &mut Rig, key: &str, text: &str) {
    let value = toml::Value::Table(text.parse().unwrap());
    rig.settings = rig.settings.with(key, value).unwrap();
}

/// The `[voice]` keys of each write in `dos`, in order, each write's keys sorted.
pub(in super::super) fn written(dos: &[Do]) -> Vec<Vec<&str>> {
    dos.iter().filter_map(write_keys).collect()
}

/// The `[voice]` keys `done` writes, sorted; `None` for an effect that writes none.
fn write_keys(done: &Do) -> Option<Vec<&str>> {
    let mut keys = match done {
        Do::Set(key, _) => vec![*key],
        Do::SetAll(values) => values.keys().map(String::as_str).collect(),
        _ => return None,
    };
    keys.sort_unstable();
    Some(keys)
}
