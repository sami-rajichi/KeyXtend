use super::*;

/// This PC's adapters: the AMD iGPU drives the screen and is listed first, then the NVIDIA dGPU, then WARP.
const AMD: Card = Card {
    luid: 0x1_19A3,
    software: false,
};
const NVIDIA: Card = Card {
    luid: 0x1_3771,
    software: false,
};
const WARP: Card = Card {
    luid: 0x1_4000,
    software: true,
};

#[test]
fn the_lowest_power_hardware_adapter_is_picked_by_its_place_in_dxgis_list() {
    assert_eq!(pick(&[AMD, NVIDIA, WARP], &[AMD, NVIDIA, WARP]), Some(0));
    // A laptop whose dGPU drives the screen lists it first, yet the iGPU stays the low-power one.
    assert_eq!(pick(&[NVIDIA, AMD, WARP], &[AMD, NVIDIA, WARP]), Some(1));
}

#[test]
fn software_adapters_are_never_picked() {
    assert_eq!(pick(&[WARP, AMD], &[WARP, AMD]), Some(1), "WARP skipped");
    assert_eq!(pick(&[WARP], &[WARP]), None, "no hardware: Qt chooses");
    assert_eq!(pick(&[], &[]), None);
}

#[test]
fn an_adapter_dxgi_does_not_list_gives_none() {
    assert_eq!(pick(&[AMD], &[NVIDIA, AMD]), None, "lists that disagree");
}

#[test]
fn a_luids_parts_make_one_number() {
    assert_eq!(luid(0x1_19A3, 0), 0x1_19A3);
    assert_eq!(luid(1, 2), (2 << 32) | 1);
    assert_eq!(luid(0, -1), 0xFFFF_FFFF_0000_0000, "the high part's bits");
}

#[cfg(windows)]
#[test]
fn on_this_system_the_pick_is_a_hardware_adapter_or_none() {
    let Some((listed, by_power)) = win::lists() else {
        panic!("DXGI answers on Windows");
    };
    let picked = low_power_adapter();
    assert_eq!(picked, pick(&listed, &by_power));
    let hardware = listed.iter().any(|c| !c.software);
    match picked {
        Some(i) => {
            let card = usize::try_from(i).ok().and_then(|i| listed.get(i));
            assert!(card.is_some_and(|c| !c.software), "{listed:?}");
        }
        None => assert!(!hardware, "{listed:?}"),
    }
}
