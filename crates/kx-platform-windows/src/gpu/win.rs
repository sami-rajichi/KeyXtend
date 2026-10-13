//! DXGI's adapter lists: in its own order, which Qt counts, and by power with the lowest first.

use super::{Card, luid, pick};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_GPU_PREFERENCE_MINIMUM_POWER,
    IDXGIAdapter1, IDXGIFactory6,
};

/// The place in DXGI's adapter list of the lowest-power hardware adapter, as Qt's `QT_D3D_ADAPTER_INDEX` takes it;
/// none when DXGI cannot say or there is none, so Qt keeps its own choice.
#[must_use]
pub fn low_power_adapter() -> Option<u32> {
    let (listed, by_power) = lists()?;
    pick(&listed, &by_power)
}

/// The adapters in DXGI's order and by power, lowest first; none when DXGI has no factory that ranks by power.
#[allow(
    unsafe_code,
    reason = "Creates a DXGI factory and walks its two adapter lists."
)]
pub(super) fn lists() -> Option<(Vec<Card>, Vec<Card>)> {
    // SAFETY: plain factory creation; the interface is released when dropped.
    let f: IDXGIFactory6 = unsafe { CreateDXGIFactory1() }.ok()?;
    // SAFETY: each call asks for adapter `i`, and DXGI ends a list with an error, which stops the walk.
    let listed = walk(|i| unsafe { f.EnumAdapters1(i) });
    // SAFETY: as above, in power order.
    let by_power =
        walk(|i| unsafe { f.EnumAdapterByGpuPreference(i, DXGI_GPU_PREFERENCE_MINIMUM_POWER) });
    Some((listed, by_power))
}

/// Every adapter `at` gives from 0, as cards, until DXGI fails to give or describe one, so each keeps its place.
fn walk(at: impl Fn(u32) -> windows::core::Result<IDXGIAdapter1>) -> Vec<Card> {
    (0..u32::MAX)
        .map_while(|i| at(i).ok().and_then(|a| card(&a)))
        .collect()
}

/// Adapter `a` as a card; none when DXGI cannot describe it.
#[allow(unsafe_code, reason = "Reads an adapter's description into a local.")]
fn card(a: &IDXGIAdapter1) -> Option<Card> {
    // SAFETY: plain query; the description is returned by value.
    let d = unsafe { a.GetDesc1() }.ok()?;
    let software = d.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0.cast_unsigned() != 0;
    let id = d.AdapterLuid;
    Some(Card {
        luid: luid(id.LowPart, id.HighPart),
        software,
    })
}
