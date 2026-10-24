//! The Windows platform: its folders, its clock, and each port it hands out working on another thread.

use super::*;
use kx_module_api::Mono;

#[test]
fn the_dirs_are_the_known_folder_ones() {
    let platform = WindowsPlatform::new().unwrap();
    assert_eq!(platform.dirs(), &dirs::current().unwrap());
}

#[test]
fn it_works_as_a_platform_object_with_a_running_clock() {
    let platform: Arc<dyn Platform> = Arc::new(WindowsPlatform::new().unwrap());
    let clock = platform.clock();
    let first = clock.now();
    assert!(first > Mono::default() && clock.now() >= first);
}

#[test]
fn the_injector_is_a_port_other_threads_can_use() {
    let injector = WindowsPlatform::new().unwrap().injector();
    let asked = std::thread::spawn(move || injector.target_app()).join();
    assert!(asked.is_ok(), "the port answers on another thread");
}

#[test]
fn the_ui_ports_answer_on_another_thread() {
    let platform = WindowsPlatform::new().unwrap();
    let (screens, look, guard) = (platform.screens(), platform.look(), platform.guard());
    let answers = std::thread::spawn(move || {
        (
            screens.work_areas(),
            look.ui_language(),
            guard.guard_own(None, None),
        )
    });
    let (areas, language, guarded) = answers.join().unwrap();
    assert!(!areas.is_empty() && !language.is_empty());
    assert_eq!(guarded, Ok(0), "no window of ours is shown");
}

#[test]
fn the_memory_port_gives_back_on_another_thread() {
    let memory = WindowsPlatform::new().unwrap().memory();
    let given = std::thread::spawn(move || memory.decommit()).join();
    assert_eq!(given.ok(), Some(Ok(())));
}

#[test]
fn the_speaker_port_takes_asks_on_another_thread_and_drops_without_sound() {
    use kx_platform::ui::{ClipId, SpeakerConfig};
    let speaker = WindowsPlatform::new().unwrap().speaker();
    let config = SpeakerConfig {
        voices: 1,
        idle_ms: 1,
    };
    // No clip is prepared, so the play builds nothing and needs no audio device; the drop joins the thread.
    let asked = std::thread::spawn(move || {
        speaker.prepare(Vec::new(), config);
        speaker.play(ClipId::new(1), 1.0);
    });
    assert!(asked.join().is_ok());
}

#[test]
fn the_tray_port_names_its_icon_size_on_another_thread() {
    let tray = WindowsPlatform::new().unwrap().tray();
    let px = std::thread::spawn(move || tray.icon_px()).join().unwrap();
    assert!(px > 0);
}

#[test]
fn the_backdrop_port_finds_no_window_we_do_not_show() {
    use kx_platform::ui::{BackdropKind, UiError};
    let platform = WindowsPlatform::new().unwrap();
    let missing = platform
        .backdrop()
        .apply("KeyXtend test: none", BackdropKind::None, false);
    assert_eq!(missing, Err(UiError::NoWindow));
}

#[test]
fn two_platforms_meet_on_one_running_copy() {
    use kx_platform::ui::Running;
    // A test name, so a real copy starting now never reads this test as running.
    let app = format!("KeyXtendTest.{}.platform", std::process::id());
    let first = WindowsPlatform::named(&app).unwrap().instance();
    let second = WindowsPlatform::named(&app).unwrap().instance();
    let claim = first.claim().unwrap();
    assert!(matches!(second.claim(), Err(Running)), "one session name");
    drop(claim);
    assert!(second.claim().is_ok(), "free again once dropped");
    assert_eq!(
        WindowsPlatform::new().unwrap().app,
        APP_DIR_NAME,
        "the app's own name"
    );
}

#[test]
fn the_scroll_target_opens_a_run_on_another_thread_and_drops_it() {
    use kx_module_api::ports::ScrollConfig;
    let _one = crate::scroll::testwin::one_client();
    let target = WindowsPlatform::new().unwrap().scroll_target();
    let config = ScrollConfig {
        check_ms: 250,
        read_ms: 300,
        edge_px: 8,
    };
    let opened = std::thread::spawn(move || target.open(config).map(drop)).join();
    assert_eq!(opened.ok(), Some(Ok(())));
}

#[test]
fn the_shortcuts_port_lists_the_table_on_another_thread_without_sending() {
    let shortcuts = WindowsPlatform::new().unwrap().shortcuts();
    let known = std::thread::spawn(move || shortcuts.known())
        .join()
        .unwrap();
    assert_eq!(known, WinShortcuts.known());
    assert!(known.iter().any(|i| i.id.as_str() == "copy"));
}

#[test]
fn the_power_dialog_port_moves_to_another_thread_without_opening() {
    // Never `open`: that would show the dialog on the owner's screen.
    let dialog = WindowsPlatform::new().unwrap().power_dialog();
    assert!(std::thread::spawn(move || drop(dialog)).join().is_ok());
}

#[test]
fn the_layout_source_is_a_port_that_lists_the_installed_layouts() {
    let layouts = WindowsPlatform::new().unwrap().layouts();
    let list = std::thread::spawn(move || layouts.installed())
        .join()
        .unwrap();
    assert!(!list.is_empty());
}

#[test]
fn the_key_guard_wraps_and_unwraps_on_another_thread() {
    let guard = WindowsPlatform::new().unwrap().key_guard();
    let key = [1; kx_crypto::KEY_LEN];
    let back = std::thread::spawn(move || guard.unwrap(&guard.wrap(&key)?)).join();
    assert_eq!(back.unwrap().map(|k| k.to_vec()), Ok(key.to_vec()));
}

#[test]
fn the_word_packs_port_opens_a_pack_on_another_thread() {
    use kx_module_api::ports::PACK_EXT;
    let dir = kx_test_support::tempdir::TempDir::new("platform-packs").unwrap();
    std::fs::write(dir.path().join(format!("en.{PACK_EXT}")), b"pack").unwrap();
    let packs = WindowsPlatform::new()
        .unwrap()
        .word_packs(vec![dir.path().to_path_buf()]);
    let read = std::thread::spawn(move || packs.open("en").map(|v| v.map(|v| v.bytes().to_vec())));
    assert_eq!(read.join().unwrap(), Ok(Some(b"pack".to_vec())));
}

#[test]
fn the_field_watch_starts_on_another_thread_and_drops() {
    use kx_module_api::ports::FieldConfig;
    let _one = crate::scroll::testwin::one_client();
    let watch = WindowsPlatform::new().unwrap().field_watch();
    let config = FieldConfig { read_ms: 150 };
    // Never a read: the box in front belongs to whoever runs the tests.
    let started = std::thread::spawn(move || watch.watch(config, Box::new(|_| {})).map(drop));
    assert_eq!(started.join().ok(), Some(Ok(())));
}
