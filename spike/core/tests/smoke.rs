//! Smoke checks that need no window and send no input.

#[test]
fn settings_load_and_every_key_has_a_label() {
    let cfg = spike_core::config::load().unwrap();
    let rows = spike_core::layout::rows(&cfg.keyboard, spike_core::layout::foreground_layout());
    assert_eq!(rows.len(), cfg.keyboard.rows.len());
    for key in rows.iter().flatten() {
        assert!(!key.label.is_empty(), "no label for {:#x}", key.code);
    }
    assert!(!spike_core::layout::installed_layouts().is_empty());
}
