//! The variables Qt reads at start. Another program of the same user could set them to load its own code into the
//! uiAccess process, so the app clears them before Qt starts (see `harden_qt`).

use std::ffi::{OsStr, OsString};

/// The name prefixes of the variables Qt, Qt Quick and the QML engine read.
pub const QT_PREFIXES: [&str; 4] = ["QT_", "QML", "QSG_", "QV4_"];

/// True for a variable Qt may read: a plain ASCII name that starts with a prefix, in any case, as Windows ignores case.
#[must_use]
pub fn is_qt(name: &OsStr) -> bool {
    let Some(name) = name.to_str().filter(|n| n.is_ascii()) else {
        return false;
    };
    let upper = name.to_ascii_uppercase();
    QT_PREFIXES.iter().any(|p| upper.starts_with(p))
}

/// The names in `names` that Qt may read, as text.
#[must_use]
pub fn qt_names(names: impl IntoIterator<Item = OsString>) -> Vec<String> {
    let qt = names.into_iter().filter(|n| is_qt(n));
    qt.filter_map(|n| n.into_string().ok()).collect()
}

#[cfg(test)]
mod tests;
