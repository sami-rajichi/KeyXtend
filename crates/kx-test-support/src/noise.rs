//! Picture noise no deflate can shrink, the same each run: `SplitMix64`'s hash of each pixel's place, shared by the
//! worker's encode tests and the live encode measurement.

/// `SplitMix64`'s step, added to the place before it is mixed.
const STEP: u64 = 0x9E37_79B9_7F4A_7C15;
/// `SplitMix64`'s two mixing factors and the shifts before each.
const MIX: [(u32, u64); 2] = [(30, 0xBF58_476D_1CE4_E5B9), (27, 0x94D0_49BB_1331_11EB)];
/// `SplitMix64`'s last shift, after the mixing.
const LAST_SHIFT: u32 = 31;

/// The blue, green and red bytes of the pixel `across` and `down`.
#[must_use]
pub fn noise(across: u32, down: u32) -> [u8; 3] {
    let place = u64::from(down) << u32::BITS | u64::from(across);
    let mut mix = place.wrapping_add(STEP);
    for (shift, factor) in MIX {
        mix = (mix ^ (mix >> shift)).wrapping_mul(factor);
    }
    let [blue, green, red, ..] = (mix ^ (mix >> LAST_SHIFT)).to_le_bytes();
    [blue, green, red]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SplitMix64`'s published first output from seed 0, `0xE220A8397B1DCDAF`: its three low bytes, lowest first.
    const FIRST: [u8; 3] = [0xAF, 0xCD, 0x1D];

    #[test]
    fn noise_is_the_same_each_run_and_differs_from_place_to_place() {
        assert_eq!(noise(0, 0), FIRST, "the place 0 is seed 0");
        let row: Vec<[u8; 3]> = (0..64).map(|x| noise(x, 0)).collect();
        let column: Vec<[u8; 3]> = (0..64).map(|y| noise(0, y)).collect();
        assert_ne!(row, column);
        let mut seen = row.clone();
        seen.dedup();
        assert_eq!(seen.len(), row.len(), "no two neighbours alike");
    }
}
