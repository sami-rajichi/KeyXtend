//! Qt's folders, asked of the qmake the environment names, so the stage takes the Qt the app was built with.

use std::path::PathBuf;

use super::{StageConfig, StageError};
use crate::devtools::{DevError, paths, ps};

/// qmake's flag that prints its settings, one `KEY:value` per line.
const QUERY: &str = "-query";
/// Separator between a key and its value in that output.
const SEP: char = ':';
/// qmake's key for Qt's version.
const VERSION: &str = "QT_VERSION";
/// qmake's keys for Qt's libraries, plugins and QML modules.
const BINS: &str = "QT_INSTALL_BINS";
const PLUGINS: &str = "QT_INSTALL_PLUGINS";
const QML: &str = "QT_INSTALL_QML";

/// Qt's version and the folders that the stage copies from.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct QtDirs {
    /// Qt's version, as qmake reports it.
    pub version: String,
    /// Qt's libraries.
    pub bins: PathBuf,
    /// Qt's plugins, one subfolder per type.
    pub plugins: PathBuf,
    /// Qt's QML modules.
    pub qml: PathBuf,
}

/// Qt's version and folders from qmake's query output; each folder must be absolute.
pub(crate) fn parse(text: &str) -> Result<QtDirs, DevError> {
    let missing = |key: &str| DevError::Failed {
        step: QUERY.to_string(),
        detail: format!("no value for {key}"),
    };
    let value = |key: &str| {
        text.lines()
            .filter_map(|line| line.trim().split_once(SEP))
            .find(|(k, v)| *k == key && !v.is_empty())
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| missing(key))
    };
    let dir = |key: &str| {
        let path = PathBuf::from(value(key)?);
        path.is_absolute()
            .then_some(path)
            .ok_or_else(|| missing(key))
    };
    Ok(QtDirs {
        version: value(VERSION)?,
        bins: dir(BINS)?,
        plugins: dir(PLUGINS)?,
        qml: dir(QML)?,
    })
}

/// Keeps `dirs` only if they belong to the Qt version `config` names.
pub(crate) fn reviewed(dirs: QtDirs, config: &StageConfig) -> Result<QtDirs, DevError> {
    if dirs.version == config.qt_version {
        Ok(dirs)
    } else {
        Err(StageError::QtVersion {
            found: dirs.version,
            want: config.qt_version.clone(),
        }
        .into())
    }
}

/// The qmake that `config.qmake_env` names; it must be an absolute path.
pub(crate) fn qmake(
    config: &StageConfig,
    env: impl Fn(&str) -> Option<String>,
) -> Result<PathBuf, DevError> {
    let key = &config.qmake_env;
    env(key)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| DevError::MissingEnv(key.clone()))
}

/// Asks the qmake the environment names for Qt's folders, and refuses any Qt but the reviewed one.
pub(crate) fn find(
    config: &StageConfig,
    env: impl Fn(&str) -> Option<String>,
) -> Result<QtDirs, DevError> {
    let program = paths::text(&qmake(config, env)?);
    reviewed(
        parse(&ps::run(&program, &program, &[QUERY.to_string()])?)?,
        config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::stage_config;

    #[test]
    fn parse_finds_the_three_folders() {
        let root = std::env::temp_dir().join("qt");
        let line = |key: &str, dir: &str| format!("{key}{SEP}{}", paths::text(&root.join(dir)));
        let text = [
            "QT_SYSROOT:".to_string(),
            format!("{VERSION}{SEP}6.11.2"),
            line(BINS, "bin"),
            line(PLUGINS, "plugins"),
            line(QML, "qml"),
        ];
        let dirs = parse(&text.join("\n")).unwrap();
        assert_eq!(dirs.bins, root.join("bin"));
        assert_eq!(dirs.plugins, root.join("plugins"));
        assert_eq!(dirs.qml, root.join("qml"));
        assert_eq!(dirs.version, "6.11.2");
    }

    #[test]
    fn only_the_reviewed_qt_is_staged() {
        let root = std::env::temp_dir().join("qt");
        let dirs = |version: &str| QtDirs {
            version: version.to_string(),
            bins: root.join("bin"),
            plugins: root.join("plugins"),
            qml: root.join("qml"),
        };
        let config = stage_config();
        assert_eq!(reviewed(dirs("6.11.2"), &config), Ok(dirs("6.11.2")));
        let err = StageError::QtVersion {
            found: "6.12.0".to_string(),
            want: "6.11.2".to_string(),
        };
        assert_eq!(reviewed(dirs("6.12.0"), &config), Err(err.into()));
    }

    #[test]
    fn parse_names_a_missing_folder_or_version() {
        let err = parse(&format!(
            "{VERSION}{SEP}6.11.2
{BINS}{SEP}relative"
        ))
        .unwrap_err();
        assert!(err.to_string().contains(BINS), "{err}");
        let err = parse("").unwrap_err();
        assert!(err.to_string().contains(VERSION), "{err}");
    }

    #[test]
    fn qmake_comes_only_from_an_absolute_variable() {
        let config = stage_config();
        let missing = DevError::MissingEnv("QMAKE".to_string());
        assert_eq!(qmake(&config, |_| None), Err(missing));
        assert!(qmake(&config, |_| Some("qmake.exe".to_string())).is_err());
        let exe = std::env::temp_dir().join("qmake.exe");
        assert_eq!(qmake(&config, |_| Some(paths::text(&exe))), Ok(exe));
    }
}
