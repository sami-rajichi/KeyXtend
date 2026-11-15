//! The engine's settings stay plain and complete, its parts go from the release folder to beside the program, and a
//! stage without the processor part every machine runs is refused.

use super::*;
use crate::test_support::{TempDir, stage_config};

/// The error `validate` gives after `change` is applied to the test engine.
fn invalid(change: impl FnOnce(&mut Engine)) -> Result<(), ConfigError> {
    let mut engine = stage_config().engine;
    change(&mut engine);
    engine.validate()
}

#[test]
fn the_test_engine_passes() {
    assert_eq!(stage_config().engine.validate(), Ok(()));
}

#[test]
fn parts_are_plain_library_names_in_lists_that_hold_some() {
    assert_eq!(
        invalid(|e| e.linked.clear()),
        Err(ConfigError::EmptyList("linked"))
    );
    assert_eq!(
        invalid(|e| e.loaded.clear()),
        Err(ConfigError::EmptyList("loaded"))
    );
    let bad = |n: &str| vec![n.to_string()];
    assert_eq!(
        invalid(|e| e.linked = bad("x/ggml.dll")),
        Err(ConfigError::NotPlain("linked"))
    );
    let not_lib = Err(ConfigError::NotType("loaded", check::LIB_EXT));
    assert_eq!(invalid(|e| e.loaded.push("ggml.exe".to_string())), not_lib);
}

#[test]
fn the_floor_is_one_of_the_loaded_parts() {
    let unlisted = Err(ConfigError::NotListed("floor", "loaded"));
    assert_eq!(
        invalid(|e| e.floor = "ggml-cpu-avx.dll".to_string()),
        unlisted
    );
    assert_eq!(
        invalid(|e| e.floor = "transcribe.dll".to_string()),
        unlisted
    );
}

#[test]
fn the_driver_is_a_plain_name_only_staged_parts_may_import() {
    let driver = |change: fn(&mut Engine)| invalid(change);
    assert_eq!(
        driver(|e| e.driver.lib = "../vulkan-1.dll".to_string()),
        Err(ConfigError::NotPlain("driver"))
    );
    assert_eq!(
        driver(|e| e.driver.by.clear()),
        Err(ConfigError::EmptyList("driver"))
    );
    let stranger = driver(|e| e.driver.by.push("Qt6Gui.dll".to_string()));
    assert_eq!(stranger, Err(ConfigError::NotListed("driver", "engine")));
}

#[test]
fn every_part_goes_from_the_release_folder_to_beside_the_program() {
    let (engine, release) = (stage_config().engine, Path::new("rel"));
    let copy = |name: &str| Copy {
        from: release.join(name),
        to: PathBuf::from(name),
    };
    let want = [
        "transcribe.dll",
        "ggml.dll",
        "ggml-cpu-x64.dll",
        "ggml-vulkan.dll",
    ]
    .map(copy);
    assert_eq!(plan(&engine, release), want);
}

#[test]
fn a_stage_without_the_floor_part_is_refused() {
    let (dir, engine) = (TempDir::new("engine-floor"), stage_config().engine);
    let want = Err(DevError::Stage(StageError::NoFloor(engine.floor.clone())));
    assert_eq!(check(dir.path(), &engine), want);
    std::fs::write(dir.path().join(&engine.floor), b"").unwrap();
    assert_eq!(check(dir.path(), &engine), Ok(()));
}

#[test]
fn only_the_card_part_may_import_the_driver() {
    let driver = stage_config().engine.driver;
    let imports = ["KERNEL32.dll".to_string(), "VULKAN-1.DLL".to_string()];
    assert_eq!(
        guard(Path::new("ggml-vulkan.dll"), &imports, &driver),
        Ok(())
    );
    let by = |file: &str| guard(Path::new(file), &imports, &driver);
    for file in [
        "ggml-base.dll",
        "keyxtend-worker.exe",
        "voice/ggml-vulkan.dll",
    ] {
        let lib = imports[1].clone();
        let want = Err(DevError::Stage(StageError::DriverImport {
            by: paths::text(Path::new(file)),
            lib,
        }));
        assert_eq!(by(file), want, "{file}");
    }
}

#[test]
fn the_stage_list_holds_every_part_beside_the_program_and_every_voice_file_in_its_folder() {
    let listed = crate::devtools::stage::planned(&stage_config());
    let parts = [
        "transcribe.dll",
        "ggml.dll",
        "ggml-cpu-x64.dll",
        "ggml-vulkan.dll",
    ]
    .map(PathBuf::from);
    assert!(parts.iter().all(|p| listed.contains(p)), "{listed:?}");
    let voice = Path::new(kx_platform::speech::VOICE_FILES).join(kx_platform::speech::DETECTOR);
    assert!(listed.contains(&voice), "{listed:?}");
}
