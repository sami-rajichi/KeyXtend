//! The speech engine in the stage: the parts it loads at run time start the walk and are not unused, only the card part
//! reaches for the graphics driver, a part planted off the list is refused (edge 39), and the summary gives the engine's
//! and the voice files' shares.

use super::budget::sized;
use super::*;
use crate::test_support::noise;
use kx_platform::speech::{DETECTOR, VOICE_FILES};

/// The worker and the test engine's parts, each importing what the real build's does.
const ENGINE: [(&str, &[&str]); 5] = [
    (WORKER, &["transcribe.dll", "kernel32.dll"]),
    ("transcribe.dll", &["ggml.dll"]),
    ("ggml.dll", &["vcruntime140.dll"]),
    ("ggml-cpu-x64.dll", &["ggml.dll"]),
    ("ggml-vulkan.dll", &["ggml.dll", "vulkan-1.dll"]),
];

/// A stage of the program, the worker and the engine, plus `more`.
fn engine_stage(more: &[(&str, &[&str])]) -> Fake {
    let files: Vec<(&str, &[&str])> = [(PROGRAM, &[][..])]
        .into_iter()
        .chain(ENGINE)
        .chain(more.iter().copied())
        .collect();
    Fake::new(&files)
}

#[test]
fn loaded_parts_nothing_imports_pass_and_the_card_part_reaches_the_driver_the_system_lacks() {
    assert!(engine_stage(&[]).check().is_ok());
}

#[test]
fn a_loaded_parts_imports_are_walked_too() {
    let mut stage = engine_stage(&[]);
    stage.imports.insert(
        "ggml-cpu-x64.dll".to_string(),
        vec!["missing.dll".to_string()],
    );
    let (by, lib) = (shown("ggml-cpu-x64.dll"), "missing.dll".to_string());
    assert_eq!(
        refusal(stage.check()),
        Some(StageError::Unresolved { by, lib })
    );
}

#[test]
fn any_other_file_importing_the_driver_is_refused() {
    let mut stage = engine_stage(&[]);
    stage
        .imports
        .insert("ggml.dll".to_string(), vec!["vulkan-1.dll".to_string()]);
    let (by, lib) = (shown("ggml.dll"), "vulkan-1.dll".to_string());
    assert_eq!(
        refusal(stage.check()),
        Some(StageError::DriverImport { by, lib })
    );
}

#[test]
fn a_part_planted_beside_the_program_or_in_the_voice_folder_is_refused() {
    let planted = engine_stage(&[("ggml-cpu-zen9.dll", &["ggml.dll"])]);
    assert_eq!(
        refusal(planted.check()),
        Some(StageError::Unused(shown("ggml-cpu-zen9.dll")))
    );
    let rel = format!("{VOICE_FILES}/ggml-cpu-x64.dll");
    let planted = engine_stage(&[(rel.as_str(), &["ggml.dll"])]);
    assert_eq!(
        refusal(planted.check()),
        Some(StageError::Unplanned(shown(&rel)))
    );
}

#[test]
fn the_summary_gives_the_engines_and_the_voice_files_shares() {
    let rel = format!("{VOICE_FILES}/{DETECTOR}");
    let stage = engine_stage(&[(rel.as_str(), &[])]);
    std::fs::write(stage.dir.path().join("ggml-vulkan.dll"), noise(96)).unwrap();
    std::fs::write(
        paths::join_rel(stage.dir.path().to_path_buf(), &rel),
        noise(32),
    )
    .unwrap();
    let summary = stage.check().unwrap();
    let empty = |n: u64| {
        (0..n).fold(Size::default(), |mut s, _| {
            s.add(sized(&[]));
            s
        })
    };
    let mut engine = empty(3);
    engine.add(sized(&noise(96)));
    assert_eq!((summary.engine, summary.voice), (engine, sized(&noise(32))));
}
