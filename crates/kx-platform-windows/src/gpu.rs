//! The graphics adapter the UI draws on: the lowest-power one Windows knows, as its place in DXGI's adapter list,
//! which is what Qt's Direct3D backend counts. Picking it is pure and tested everywhere; the OS calls are in `win`.

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::low_power_adapter;

/// An adapter as DXGI lists it: its locally unique id as one number, and whether it draws in software (WARP).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card {
    /// The adapter's LUID, high part above the low.
    pub luid: u64,
    /// A software adapter, which the UI never picks.
    pub software: bool,
}

/// A LUID's two parts as one number.
#[must_use]
pub fn luid(low: u32, high: i32) -> u64 {
    (u64::from(high.cast_unsigned()) << u32::BITS) | u64::from(low)
}

/// The place in `listed`, DXGI's adapter order that Qt counts, of the first hardware adapter in `by_power`, which
/// lists the lowest-power first; none when there is no hardware adapter or `listed` lacks it.
#[must_use]
pub fn pick(listed: &[Card], by_power: &[Card]) -> Option<u32> {
    let want = by_power.iter().find(|c| !c.software)?;
    let at = listed.iter().position(|c| c.luid == want.luid)?;
    u32::try_from(at).ok()
}

#[cfg(test)]
mod tests;
