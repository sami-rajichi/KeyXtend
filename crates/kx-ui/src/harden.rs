//! The variables Qt reads at start. Another program of the same user could set them to load its own code into the
//! uiAccess process, so the app clears them before Qt starts (see `harden_qt`).

/// The name prefixes of the variables Qt, Qt Quick and the QML engine read.
pub const QT_PREFIXES: [&str; 4] = ["QT_", "QML", "QSG_", "QV4_"];

/// Ends each name in a list of names, as it ends a C string.
pub const NAME_END: u8 = 0;

/// True for a variable Qt may read: a name that starts with a prefix, in any case, as Windows ignores case. The rest
/// may be any bytes, so every name with a Qt prefix goes, whatever the code page made of it.
#[must_use]
pub fn is_qt(name: &[u8]) -> bool {
    QT_PREFIXES.iter().any(|p| {
        name.get(..p.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(p.as_bytes()))
    })
}

/// The names in `list`, each ended by `NAME_END`, that Qt may read, in order.
#[must_use]
pub fn qt_names(list: &[u8]) -> Vec<&[u8]> {
    list.split(|b| *b == NAME_END)
        .filter(|n| is_qt(n))
        .collect()
}

#[cfg(test)]
mod tests;
