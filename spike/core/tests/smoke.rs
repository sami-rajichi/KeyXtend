//! Smoke checks that need no window and send no input.

#[test]
fn settings_load_and_every_key_has_a_label() {
    let cfg = spike_core::config::load().unwrap();
    let front = kx_platform_windows::front::layout();
    let rows = spike_core::layout::rows(&cfg.keyboard, front);
    assert_eq!(rows.len(), cfg.keyboard.rows.len());
    for key in rows.iter().flatten() {
        assert!(!key.label.is_empty(), "no label for {:#x}", key.code);
    }
    let layouts = kx_platform_windows::WinLayouts;
    assert!(!kx_module_api::ports::LayoutSource::installed(&layouts).is_empty());
}
