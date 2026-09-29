//! One registered module's section: its version, the migrations it needs, and repair against the defaults.

use crate::merge::{self, Repair};
use crate::names::{ARG_KEYS, ARG_MODULE, KEYS_SEPARATOR, VERSION_KEY};
use kx_module_api::{ModuleId, Notice, SettingsError, SettingsSpec, keys};
use std::cmp::Ordering;
use toml::{Table, Value};

/// A registered module's settings.
#[derive(Debug)]
pub(crate) struct Section {
    /// The module's spec.
    spec: &'static SettingsSpec,
    /// The spec's parsed defaults.
    defaults: Table,
    /// What the module reads: the defaults with the user's kept values, never the version key.
    pub effective: Table,
    /// The kept values that differ from the defaults; what a save writes.
    pub changes: Table,
    /// A section from a newer version, kept word for word; the module then uses its defaults.
    pub newer: Option<Value>,
}

/// A section just read, its notice, and whether the file must be saved to match.
#[derive(Debug)]
pub(crate) struct Read {
    /// The section.
    pub section: Section,
    /// What to tell the user, if anything.
    pub notice: Option<Notice>,
    /// True when something was reset, so the file differs from the store.
    pub repaired: bool,
}

/// How a section's version compares with the spec's.
enum Age {
    /// The spec's version, or no version key.
    Current,
    /// An older version, which the migrations upgrade.
    Older(u32),
    /// A newer version, kept as it is.
    Newer,
    /// Not a positive integer.
    Bad,
}

/// The spec's defaults, or `None` when the spec is broken: bad text, invalid defaults, a version key or a migration gap.
pub(crate) fn defaults_of(spec: &SettingsSpec) -> Option<Table> {
    let defaults: Table = spec.defaults.parse().ok()?;
    let sound = spec.is_consistent()
        && !defaults.contains_key(VERSION_KEY)
        && (spec.validate)(&defaults).is_ok();
    sound.then_some(defaults)
}

/// Reads the module's raw section from the file, if any, against its sound spec and defaults.
pub(crate) fn read(
    id: ModuleId,
    spec: &'static SettingsSpec,
    defaults: Table,
    raw: Option<Value>,
) -> Read {
    let Some(raw) = raw else {
        return Read::quiet(Section::fresh(spec, defaults));
    };
    let Value::Table(mut values) = raw else {
        return Read::bad(id, Section::fresh(spec, defaults), id.as_str().to_owned());
    };
    let named = key_list(&values, id);
    match age(&values, spec.version) {
        Age::Newer => Read::newer(id, Section::fresh(spec, defaults), Value::Table(values)),
        Age::Bad => Read::bad(id, Section::fresh(spec, defaults), named),
        Age::Current => Read::repaired(id, spec, defaults, strip(values)),
        Age::Older(from) => match migrate(&mut values, spec, from) {
            Ok(()) => Read::repaired(id, spec, defaults, strip(values)),
            Err(_) => Read::bad(id, Section::fresh(spec, defaults), named),
        },
    }
}

/// The section's version against the spec's.
fn age(values: &Table, current: u32) -> Age {
    match values.get(VERSION_KEY) {
        None => Age::Current,
        Some(Value::Integer(n)) if *n >= 1 => match u32::try_from(*n) {
            Ok(version) => match version.cmp(&current) {
                Ordering::Equal => Age::Current,
                Ordering::Less => Age::Older(version),
                Ordering::Greater => Age::Newer,
            },
            Err(_) => Age::Newer,
        },
        Some(_) => Age::Bad,
    }
}

/// Runs the migrations from version `from` up to the spec's version, in order.
fn migrate(values: &mut Table, spec: &SettingsSpec, from: u32) -> Result<(), SettingsError> {
    values.remove(VERSION_KEY);
    let done = usize::try_from(from.saturating_sub(1)).unwrap_or(usize::MAX);
    for step in spec.migrations.iter().skip(done) {
        step(values)?;
    }
    Ok(())
}

/// The values without the version key.
fn strip(mut values: Table) -> Table {
    values.remove(VERSION_KEY);
    values
}

/// The section's setting keys for a notice, or the section name when it has none.
fn key_list(values: &Table, id: ModuleId) -> String {
    let named: Vec<&str> = values
        .keys()
        .map(String::as_str)
        .filter(|k| *k != VERSION_KEY)
        .collect();
    if named.is_empty() {
        id.as_str().to_owned()
    } else {
        named.join(KEYS_SEPARATOR)
    }
}

/// A notice about the module, with the module argument first.
fn notice(key: &'static str, id: ModuleId, reset: Option<String>) -> Notice {
    let mut args = vec![(ARG_MODULE, id.to_string())];
    args.extend(reset.map(|keys| (ARG_KEYS, keys)));
    Notice {
        key,
        module: Some(id),
        args,
    }
}

impl Read {
    /// A section with nothing to tell.
    fn quiet(section: Section) -> Self {
        Self {
            section,
            notice: None,
            repaired: false,
        }
    }

    /// A bad section: the defaults, and a reset notice naming `keys`.
    fn bad(id: ModuleId, section: Section, keys: String) -> Self {
        Self {
            section,
            notice: Some(notice(keys::SETTINGS_RESET, id, Some(keys))),
            repaired: true,
        }
    }

    /// A newer section: kept for saving, while the module uses its defaults.
    fn newer(id: ModuleId, mut section: Section, raw: Value) -> Self {
        section.newer = Some(raw);
        Self {
            section,
            notice: Some(notice(keys::SETTINGS_NEWER, id, None)),
            repaired: false,
        }
    }

    /// The user's values repaired one by one against the defaults.
    fn repaired(id: ModuleId, spec: &'static SettingsSpec, defaults: Table, user: Table) -> Self {
        let Repair {
            effective,
            rejected,
        } = merge::repair(&defaults, user, spec.validate);
        let changes = merge::changes(&effective, &defaults);
        let reset = (!rejected.is_empty()).then(|| rejected.join(KEYS_SEPARATOR));
        Self {
            notice: reset.map(|keys| notice(keys::SETTINGS_RESET, id, Some(keys))),
            repaired: !rejected.is_empty(),
            section: Section {
                spec,
                defaults,
                effective,
                changes,
                newer: None,
            },
        }
    }
}

impl Section {
    /// The defaults alone.
    fn fresh(spec: &'static SettingsSpec, defaults: Table) -> Self {
        Self {
            spec,
            effective: defaults.clone(),
            defaults,
            changes: Table::new(),
            newer: None,
        }
    }

    /// The spec's version, written before the changes.
    pub(crate) fn version(&self) -> u32 {
        self.spec.version
    }

    /// Sets `key` when the spec accepts the result, and says whether anything changed.
    pub(crate) fn set(&mut self, key: &str, value: Value) -> Result<bool, SettingsError> {
        let next = merge::with(&self.effective, key, value.clone());
        (self.spec.validate)(&next)?;
        if next == self.effective {
            return Ok(false);
        }
        self.effective = next;
        if self.defaults.get(key) == Some(&value) {
            self.changes.remove(key);
        } else {
            self.changes.insert(key.to_owned(), value);
        }
        Ok(true)
    }
}
