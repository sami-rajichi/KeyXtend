//! The speech engine in the stage (ADR-0044): its parts go from the worker's release build to beside the worker, the
//! processor part every machine runs must be there, and only the graphics-card part may import the graphics driver.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::config::each;
use super::{Copy, StageError, check};
use crate::config::ConfigError;
use crate::devtools::{DevError, paths};

/// The engine's parts and the driver its card part needs, from `[workspace.metadata.stage] engine`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Engine {
    /// Parts the worker imports, which the import walk reaches.
    pub linked: Vec<String>,
    /// Parts the engine loads from the worker's folder at run time; the walk starts at each.
    pub loaded: Vec<String>,
    /// The loaded part every processor runs, which the stage must hold.
    pub floor: String,
    /// The graphics driver's library, never staged, and the only parts that may import it.
    pub driver: Driver,
}

/// A library the system's graphics driver brings, and its allowed importers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Driver {
    /// The library's file name.
    pub lib: String,
    /// The staged parts, beside the program, that may import it.
    pub by: Vec<String>,
}

impl Engine {
    /// Rejects empty lists, parts that are not plain library names, a floor off `loaded` and a driver importer that
    /// is not a part.
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        let lists = [
            ("linked", &self.linked),
            ("loaded", &self.loaded),
            ("driver", &self.driver.by),
        ];
        let not_lib = |name: &'static str| ConfigError::NotType(name, check::LIB_EXT);
        for (name, list) in lists {
            each(name, list, paths::is_plain_name, ConfigError::NotPlain)?;
            each(name, list, |v| check::is_lib(Path::new(v)), not_lib)?;
        }
        if !paths::is_plain_name(&self.driver.lib) {
            return Err(ConfigError::NotPlain("driver"));
        }
        if !self.loaded.contains(&self.floor) {
            return Err(ConfigError::NotListed("floor", "loaded"));
        }
        if self.driver.by.iter().any(|b| !self.parts().any(|p| p == b)) {
            return Err(ConfigError::NotListed("driver", "engine"));
        }
        Ok(())
    }

    /// Every part, linked then loaded.
    pub(crate) fn parts(&self) -> impl Iterator<Item = &String> {
        self.linked.iter().chain(&self.loaded)
    }
}

/// The copies of every part from the `release` folder to beside the program.
pub(crate) fn plan(engine: &Engine, release: &Path) -> Vec<Copy> {
    let part = |name: &String| Copy {
        from: release.join(name),
        to: PathBuf::from(name),
    };
    engine.parts().map(part).collect()
}

/// Refuses `stage` when the floor part is not beside the program.
pub(crate) fn check(stage: &Path, engine: &Engine) -> Result<(), DevError> {
    if stage.join(&engine.floor).is_file() {
        Ok(())
    } else {
        Err(StageError::NoFloor(engine.floor.clone()).into())
    }
}

/// Fails if `file` imports the driver but is not a part beside the program that `driver` allows.
pub(crate) fn guard(file: &Path, imports: &[String], driver: &Driver) -> Result<(), DevError> {
    let name = check::key(&file.to_string_lossy());
    let beside = file.parent() == Some(Path::new(""));
    let allowed = beside && driver.by.iter().any(|b| check::key(b) == name);
    match imports
        .iter()
        .find(|lib| check::key(lib) == check::key(&driver.lib))
    {
        Some(lib) if !allowed => {
            let (by, lib) = (paths::text(file), lib.clone());
            Err(StageError::DriverImport { by, lib }.into())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
