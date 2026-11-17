//! The staged edges' pure parts: what the copy takes from the stage, what each edge deletes from it, and each edge's
//! verdict.

use kx_mod_voice::score::Wrong;
use kx_module_api::ports::{CardId, Device, DeviceKind, Memory};

use super::copy::{Edge, copied, dropped, need};
use super::edges::{Floor, NoCard, Plant, first_card};
use crate::config::load;
use crate::g22cfg::G22Staged;

const PREFIX: &str = "ggml-cpu-";
const WORKER: &str = "keyxtend-worker.exe";

fn staged() -> G22Staged {
    load().expect("the settings load").g22_staged
}

fn names(n: &[&str]) -> Vec<String> {
    n.iter().map(ToString::to_string).collect()
}

/// A staged worker's folder, as the stage writes it.
fn stage() -> Vec<String> {
    names(&[
        WORKER,
        "keyxtend.exe",
        "transcribe.dll",
        "ggml.dll",
        "ggml-cpu-x64.dll",
        "ggml-cpu-haswell.dll",
        "GGML-CPU-SSE42.DLL",
        "ggml-vulkan.dll",
    ])
}

#[test]
fn the_copy_takes_every_file_and_only_the_voice_folder() {
    let entries = [
        ("keyxtend-worker.exe".to_string(), false),
        ("voice".to_string(), true),
        ("qml".to_string(), true),
        ("ggml.dll".to_string(), false),
    ];
    let got = copied(&entries, "voice");
    let want = [entries[0].clone(), entries[1].clone(), entries[3].clone()];
    assert_eq!(got, want);
}

#[test]
fn the_edges_run_plant_floor_then_no_card_each_deleting_its_parts() {
    let g = staged();
    assert_eq!(Edge::ORDER.map(Edge::number), [39, 29, 28]);
    assert!(
        dropped(&stage(), Edge::Plant, &g, PREFIX).is_empty(),
        "the whole stage"
    );
    let floor = dropped(&stage(), Edge::Floor, &g, PREFIX);
    assert_eq!(
        floor,
        names(&["ggml-cpu-haswell.dll", "GGML-CPU-SSE42.DLL"])
    );
    assert_eq!(
        dropped(&stage(), Edge::NoCard, &g, PREFIX),
        std::slice::from_ref(&g.card_part)
    );
}

#[test]
fn a_stage_without_the_worker_the_card_part_or_the_floor_part_is_refused() {
    let g = staged();
    assert_eq!(need(&stage(), &g, WORKER), Ok(()));
    for gone in [WORKER, "ggml-vulkan.dll", "ggml-cpu-x64.dll"] {
        let left: Vec<String> = stage().into_iter().filter(|n| n != gone).collect();
        let e = need(&left, &g, WORKER).expect_err(gone);
        assert!(e.contains(gone), "{e}");
    }
}

fn device(kind: DeviceKind) -> Device {
    Device {
        kind,
        name: String::new(),
        memory: Memory::default(),
        cores: 0,
        built_in: false,
        driver: String::new(),
    }
}

fn card(id: &str) -> Device {
    device(DeviceKind::Card(CardId::new(id).expect("an id")))
}

#[test]
fn edge_28_passes_with_the_processor_and_no_card() {
    let cpu = device(DeviceKind::Cpu);
    assert!(NoCard::of(std::slice::from_ref(&cpu)).pass());
    assert!(
        !NoCard::of(&[cpu.clone(), card("vk0")]).pass(),
        "a card offered"
    );
    assert!(!NoCard::of(&[]).pass(), "no processor");
    assert_eq!(
        first_card(&[cpu, card("vk1"), card("vk0")]),
        CardId::new("vk1")
    );
}

fn wrong(words: u16) -> Wrong {
    Wrong { words, total: 14 }
}

#[test]
fn edge_29_passes_when_the_processor_read_the_clip_within_card_diff() {
    let good = Floor {
        wrong: Some(wrong(1)),
        card_wrong: Some(wrong(9)),
        loaded: false,
    };
    assert!(
        good.pass(1),
        "the card's text and the module list are readings"
    );
    assert!(
        !Floor {
            wrong: Some(wrong(2)),
            ..good
        }
        .pass(1)
    );
    assert!(
        !Floor {
            wrong: None,
            ..good
        }
        .pass(1),
        "no rules, no verdict"
    );
}

#[test]
fn edge_39_passes_only_when_the_list_saw_the_copys_parts_and_none_planted() {
    let good = Plant {
        snaps: 3,
        failed: 0,
        copy_parts: 4,
        planted: 0,
    };
    assert!(good.pass());
    assert!(
        !Plant { planted: 1, ..good }.pass(),
        "the planted part loaded"
    );
    assert!(
        !Plant {
            copy_parts: 0,
            ..good
        }
        .pass(),
        "the list saw no part"
    );
    assert!(!Plant { snaps: 0, ..good }.pass(), "no snapshot");
}
