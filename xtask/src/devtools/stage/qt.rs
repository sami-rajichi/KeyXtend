//! Qt's folders, asked of the qmake the environment names, so the stage takes the Qt the app was built with.

use std::path::PathBuf;

use super::StageConfig;
use crate::devtools::{DevError, paths, ps};

/// qmake's flag that prints its settings, one `KEY:value` per line.
const QUERY: &str = "-query";
/// Separator between a key and its value in that output.
const SEP: char = ':';
/// qmake's keys for Qt's libraries, plugins and QML modules.
const BINS: &str = "QT_INSTALL_BINS";
const PLUGINS: &str = "QT_INSTALL_PLUGINS";
const QML: &str = "QT_INSTALL_QML";

/// Qt's folders that the stage copies from.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct QtDirs {
    /// Qt's libraries.
    pub bins: PathBuf,
    /// Qt's plugins, one subfolder per type.
    pub plugins: PathBuf,
    /// Qt's QML modules.
    pub qml: PathBuf,
}

/// Qt's folders from qmake's query output; each must be absolute.
pub(crate) fn parse(text: &str) -> Result<QtDirs, DevError> {
    let value = |key: &str| {
        text.lines()
            .filter_map(|line| line.trim().split_once(SEP))
            .find(|(k, _)| *k == key)
            .map(|(_, v)| PathBuf::from(v))
            .filter(|p| p.is_absolute())
            .ok_or_else(|| DevError::Failed {
                step: QUERY.to_string(),
                detail: format!("no folder for {key}"),
            })
    };
    Ok(QtDirs {
        bins: value(BINS)?,
        plugins: value(PLUGINS)?,
        qml: value(QML)?,
    })
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

/// Asks the qmake the environment names for Qt's folders.
pub(crate) fn find(
    config: &StageConfig,
    env: impl Fn(&str) -> Option<String>,
) -> Result<QtDirs, DevError> {
    let program = paths::text(&qmake(config, env)?);
    parse(&ps::run(&program, &program, &[QUERY.to_string()])?)
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
            line(BINS, "bin"),
            line(PLUGINS, "plugins"),
            line(QML, "qml"),
        ];
        let dirs = parse(&text.join("\n")).unwrap();
        assert_eq!(dirs.bins, root.join("bin"));
        assert_eq!(dirs.plugins, root.join("plugins"));
        assert_eq!(dirs.qml, root.join("qml"));
    }

    #[test]
    fn parse_names_a_missing_folder() {
        let err = parse("QT_INSTALL_BINS:relative").unwrap_err();
        assert!(err.to_string().contains(BINS), "{err}");
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
