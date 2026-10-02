//! One registered module's section: its version, the migrations it needs, and repair against the defaults.

use crate::merge::{self, Repair};
use crate::names::{MAX_KEY_CHARS, MAX_NAMED_KEYS, VERSION_KEY};
use kx_module_api::{ModuleId, Notice, NoticeArg, SettingsError, SettingsSpec, args, keys};
use std::cmp::Ordering;
use std::panic;
use toml::{Table, Value};

/// A notice's arguments, as name and value.
type Args = Vec<(&'static str, NoticeArg)>;

/// Why a change was refused: it names a key the module's defaults lack.
const NOT_A_SETTING: &str = "a key is not one of the module's settings";

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

/// Checks the spec and reads the module's raw section; `None` when the spec is broken or the module's code panics.
pub(crate) fn guarded_read(
    id: ModuleId,
    spec: &'static SettingsSpec,
    raw: Option<Value>,
) -> Option<Read> {
    let attempt = || defaults_of(spec).map(|defaults| read(id, spec, defaults, raw));
    panic::catch_unwind(attempt).ok().flatten()
}

/// The spec's defaults, or `None` when the spec is broken: bad text, invalid defaults, a version key or a migration gap.
fn defaults_of(spec: &SettingsSpec) -> Option<Table> {
    let defaults: Table = spec.defaults.parse().ok()?;
    let sound = spec.is_consistent()
        && !defaults.contains_key(VERSION_KEY)
        && (spec.validate)(&defaults).is_ok();
    sound.then_some(defaults)
}

/// Reads the module's raw section from the file, if any, against its sound spec and defaults.
fn read(id: ModuleId, spec: &'static SettingsSpec, defaults: Table, raw: Option<Value>) -> Read {
    let Some(raw) = raw else {
        return Read::quiet(Section::fresh(spec, defaults));
    };
    let Value::Table(mut values) = raw else {
        return Read::bad(id, Section::fresh(spec, defaults), reset_args(&[]));
    };
    let named = key_list(&values);
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

/// Notice arguments naming the section's setting keys.
fn key_list(values: &Table) -> Args {
    let named: Vec<&str> = values
        .keys()
        .map(String::as_str)
        .filter(|k| *k != VERSION_KEY)
        .collect();
    reset_args(&named)
}

/// Notice arguments naming at most `MAX_NAMED_KEYS` of the `keys` short enough to show, and counting the rest.
/// A notice that can name none counts at least the one bad value, so its text always says something.
fn reset_args(keys: &[&str]) -> Args {
    let shown: Vec<String> = keys
        .iter()
        .filter(|k| k.chars().count() <= MAX_KEY_CHARS)
        .take(MAX_NAMED_KEYS)
        .map(|k| (*k).to_owned())
        .collect();
    let left = keys.len() - shown.len();
    let more = if shown.is_empty() { left.max(1) } else { left };
    let mut named = vec![(args::KEYS, NoticeArg::List(shown))];
    if more > 0 {
        named.push((args::MORE, NoticeArg::Count(more)));
    }
    named
}

/// A notice about the module, with the module argument before `extra`.
fn notice(key: &'static str, id: ModuleId, extra: Args) -> Notice {
    let mut all = vec![(args::MODULE, NoticeArg::Text(id.to_string()))];
    all.extend(extra);
    Notice {
        key,
        module: Some(id),
        args: all,
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

    /// A bad section: the defaults, and a reset notice with `named`.
    fn bad(id: ModuleId, section: Section, named: Args) -> Self {
        Self {
            section,
            notice: Some(notice(keys::SETTINGS_RESET, id, named)),
            repaired: true,
        }
    }

    /// A newer section: kept for saving, while the module uses its defaults.
    fn newer(id: ModuleId, mut section: Section, raw: Value) -> Self {
        section.newer = Some(raw);
        Self {
            section,
            notice: Some(notice(keys::SETTINGS_NEWER, id, Vec::new())),
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
        let names: Vec<&str> = rejected.iter().map(String::as_str).collect();
        let reset = (!rejected.is_empty()).then(|| reset_args(&names));
        Self {
            notice: reset.map(|named| notice(keys::SETTINGS_RESET, id, named)),
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

    /// Sets every key of `values` when the spec accepts them together, and says whether anything changed.
    /// A key the defaults lack is refused, as a load's repair would reset it, even when the validator lets it through.
    pub(crate) fn set_all(&mut self, values: Table) -> Result<bool, SettingsError> {
        let mut next = self.effective.clone();
        next.extend(values.clone());
        (self.spec.validate)(&next)?;
        if values.keys().any(|key| !self.defaults.contains_key(key)) {
            return Err(SettingsError::Invalid(NOT_A_SETTING.to_owned()));
        }
        if next == self.effective {
            return Ok(false);
        }
        self.effective = next;
        for (key, value) in values {
            if self.defaults.get(&key) == Some(&value) {
                self.changes.remove(&key);
            } else {
                self.changes.insert(key, value);
            }
        }
        Ok(true)
    }
}
