//! The threads rule (edge case 30), the fit check (edge case 24) and the host's device list.

use super::*;
use std::borrow::Cow;

/// A device of `kind` named `id`, with `total` and `free` MB.
fn found(kind: DeviceKind, id: &str, total: u64, free: u64) -> Found {
    Found {
        kind,
        id: id.into(),
        name: format!("{id} name"),
        total: total * MB,
        free: free * MB,
    }
}

fn card(id: &str, need_mb: u32) -> Target<'_> {
    Target::Card {
        id: Cow::Borrowed(id),
        need_mb,
    }
}

#[test]
fn the_hosts_count_is_taken_past_the_workers_own_reading() {
    assert_eq!(threads(4, 8), 4);
    assert_eq!(threads(8, 8), 8);
    let (host, hybrid_reading) = (14, 10);
    assert_eq!(
        threads(host, hybrid_reading),
        14,
        "a hybrid PC's real cores"
    );
    assert_eq!(threads(16, 8), 16);
}

#[test]
fn only_the_settings_top_caps_the_hosts_count() {
    let top = *kx_ipc::THREADS.end();
    assert_eq!(threads(top, 8), usize::from(top));
    assert_eq!(threads(u8::MAX, 8), usize::from(top));
}

#[test]
fn zero_from_no_host_count_means_the_workers_own_reading() {
    assert_eq!(threads(0, 8), 8);
    assert_eq!(threads(0, 6), 6);
    assert_eq!(threads(0, 0), 1, "a count of none is one core");
}

#[test]
fn the_real_cores_are_at_least_one_and_never_past_the_logical_processors() {
    let logical = thread::available_parallelism().map_or(1, NonZero::get);
    assert!((1..=logical).contains(&cores()));
}

#[test]
fn the_topology_level_counts_the_threads_of_a_core_only_when_it_is_their_level() {
    let smt = SMT_LEVEL << LEVEL_SHIFT;
    assert_eq!(smt_width(2, smt), 2);
    assert_eq!(smt_width(1, smt), 1);
    assert_eq!(smt_width(0, smt), 1, "a zero count is one thread");
    assert_eq!(
        smt_width(16, 2 << LEVEL_SHIFT),
        1,
        "a core level is not threads"
    );
}

#[test]
fn a_hybrid_processor_never_counts_a_cores_second_thread_as_a_core() {
    assert_eq!(width(1, false), 1, "no second threads");
    assert_eq!(width(2, false), 2);
    assert_eq!(width(1, true), HYBRID_SMT, "read on an efficient core");
    assert_eq!(width(2, true), 2, "read on a performance core");
    let (alder_lake, threads) = (14, 20);
    assert!(threads / width(1, true) <= alder_lake);
}

#[test]
fn a_card_short_of_free_memory_now_is_refused_and_the_processor_serves() {
    let cards = [
        found(DeviceKind::Cpu, "CPU", 16_000, 9_000),
        found(DeviceKind::Card, "Vulkan0", 8192, 1000),
    ];
    assert_eq!(
        place(&card("Vulkan0", 1001), &cards),
        (Place::Cpu, Some(VoiceFault::CardRoom))
    );
    assert_eq!(
        place(&card("Vulkan0", 1000), &cards),
        (Place::Card("Vulkan0".into()), None)
    );
}

#[test]
fn a_gone_card_or_the_processor_named_as_one_is_no_card() {
    let cards = [found(DeviceKind::Cpu, "CPU", 16_000, 9_000)];
    for id in ["Vulkan1", "CPU"] {
        assert_eq!(
            place(&card(id, 1), &cards),
            (Place::Cpu, Some(VoiceFault::NoCard)),
            "{id}"
        );
    }
    assert_eq!(place(&Target::Cpu, &cards), (Place::Cpu, None));
}

#[test]
fn built_in_graphics_serve_as_a_card_and_are_listed_as_built_in() {
    let all = [
        found(DeviceKind::BuiltIn, "Vulkan1", 4096, 3000),
        found(DeviceKind::Cpu, "CPU", 16_000, 9_000),
    ];
    assert_eq!(
        place(&card("Vulkan1", 2000), &all),
        (Place::Card("Vulkan1".into()), None)
    );
    let kinds: Vec<DeviceKind> = list(&all).iter().map(|d| d.kind).collect();
    assert_eq!(kinds, [DeviceKind::Cpu, DeviceKind::BuiltIn]);
}

#[test]
fn the_list_puts_the_processor_first_and_drops_ids_the_frames_refuse() {
    let all = [
        found(DeviceKind::Card, "Vulkan0", 8192, 7000),
        found(DeviceKind::Card, "bad id", 8192, 7000),
        found(DeviceKind::Card, &"x".repeat(MAX_TAG_BYTES + 1), 1, 1),
        found(DeviceKind::Cpu, "CPU", 16_000, 9_000),
    ];
    let got = list(&all);
    let ids: Vec<&str> = got.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["CPU", "Vulkan0"]);
    assert_eq!((got[1].total_mb, got[1].free_mb), (8192, 7000));
    assert!(got.iter().all(|d| d.driver.is_empty()));
}

#[test]
fn free_memory_never_passes_the_total_and_long_names_are_cut_cleanly() {
    let mut odd = found(DeviceKind::Card, "Vulkan0", 100, 200);
    odd.name = format!("é{}\n", "a".repeat(MAX_NAME_BYTES));
    let got = list(&[found(DeviceKind::Cpu, "CPU", 1, 1), odd]);
    assert_eq!((got[1].total_mb, got[1].free_mb), (100, 100));
    assert!(got[1].name.len() <= MAX_NAME_BYTES && got[1].name.starts_with('é'));
    assert!(!got[1].name.contains('\n'));
}

#[test]
fn a_name_the_frames_refuse_is_shown_as_the_device_id() {
    let mut odd = found(DeviceKind::Card, "Vulkan0", 1, 1);
    odd.name = "GPU\u{202E}x".into();
    let got = list(&[found(DeviceKind::Cpu, "CPU", 1, 1), odd]);
    assert_eq!(got[1].name, "Vulkan0");
    assert_eq!(VoiceNote::Devices { id: 1, list: got }.check(), Ok(()));
}

#[test]
fn the_list_holds_at_most_the_frames_count_of_devices() {
    let many: Vec<Found> = (0..=MAX_DEVICES)
        .map(|k| found(DeviceKind::Card, &format!("Vulkan{k}"), 1, 1))
        .collect();
    assert_eq!(list(&many).len(), MAX_DEVICES);
}
