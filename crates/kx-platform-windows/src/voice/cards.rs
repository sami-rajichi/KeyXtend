//! Each card's driver version as DXGI reads it, found by the name the engine gives the card, as the engine reports
//! none and a driver update must run the card's check again (edge 26).

use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIAdapter1, IDXGIDevice, IDXGIFactory1,
};
use windows::core::Interface;

/// Bits in each of a driver version's four parts, high part first.
const PART_BITS: u32 = 16;
/// Parts in a driver version, such as 32.0.15.6094.
const PARTS: u32 = 4;
/// What joins a driver version's parts.
const DOT: &str = ".";

/// The driver version of the card named `name`; `None` when no card has that name or DXGI does not tell.
#[allow(
    unsafe_code,
    reason = "Walks DXGI's adapters and asks each for its driver."
)]
pub(super) fn driver(name: &str) -> Option<String> {
    // SAFETY: plain factory creation; the interface is released when dropped.
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.ok()?;
    // SAFETY: each call asks for adapter `i`, and DXGI ends the list with an error, which stops the walk.
    let adapters = (0..u32::MAX).map_while(|i| unsafe { factory.EnumAdapters1(i) }.ok());
    let card = adapters
        .into_iter()
        .find(|a| described(a).is_some_and(|d| same(&d, name)))?;
    // SAFETY: a plain query on a live adapter, with the interface id DXGI documents for the driver's version.
    let umd = unsafe { card.CheckInterfaceSupport(&IDXGIDevice::IID) }.ok()?;
    Some(version(umd))
}

/// The name DXGI gives adapter `a`.
#[allow(unsafe_code, reason = "Reads an adapter's description into a local.")]
fn described(a: &IDXGIAdapter1) -> Option<String> {
    // SAFETY: plain query; the description is returned by value.
    let d = unsafe { a.GetDesc1() }.ok()?;
    let end = d
        .Description
        .iter()
        .position(|u| *u == 0)
        .unwrap_or(d.Description.len());
    Some(String::from_utf16_lossy(&d.Description[..end]))
}

/// True when DXGI's name and the engine's name the same card, spaces at the ends and case aside.
pub(super) fn same(system: &str, engine: &str) -> bool {
    !engine.trim().is_empty() && system.trim().eq_ignore_ascii_case(engine.trim())
}

/// A driver version as Windows shows it, its four parts from DXGI's number, high part first.
pub(super) fn version(umd: i64) -> String {
    let bits = umd.cast_unsigned();
    let mask = (1_u64 << PART_BITS) - 1;
    let parts = (0..PARTS)
        .rev()
        .map(|at| ((bits >> (at * PART_BITS)) & mask).to_string());
    parts.collect::<Vec<_>>().join(DOT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_driver_version_reads_as_windows_shows_it() {
        let umd = [32_i64, 0, 15, 6094]
            .into_iter()
            .fold(0, |v, part| (v << PART_BITS) + part);
        assert_eq!(version(umd), "32.0.15.6094");
    }

    #[test]
    fn a_card_is_found_by_its_name_in_any_case_and_never_by_an_empty_one() {
        assert!(same("AMD Radeon RX 6600 ", "amd radeon rx 6600"));
        assert!(!same("AMD Radeon RX 6600", "AMD Radeon RX 6600 XT"));
        assert!(!same("", " "));
    }

    #[test]
    fn a_name_no_card_has_has_no_driver() {
        assert_eq!(driver("KeyXtend test card that is not there"), None);
    }
}
