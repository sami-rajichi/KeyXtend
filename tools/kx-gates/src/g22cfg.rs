//! G22's settings in `kx-gates-p11.toml`: the bench's (`[g22_bench]`, in `bench`), the product run's
//! (`[g22_product]`, in `prod`) with its cloud line (`[g22_cloud]`, in `cloud`), and the staged worker's edges
//! (`[g22_staged]`, in `staged`). The run and the edges read the bench's clips, folders and gate worker.
#![cfg(windows)]

mod bench;
mod cloud;
mod prod;
mod staged;

#[cfg(feature = "voice")]
pub use bench::fewest;
pub use bench::{Clip, G22Bench, is_part};
pub use cloud::{DAILY, G22Cloud};
pub use prod::G22Product;
pub use staged::G22Staged;

use std::path::{Component, Path, PathBuf};

/// A single folder name: not empty, not a dot name, no drive or separator; the gates delete in it.
pub fn plain_name(n: &str) -> bool {
    let mut parts = Path::new(n).components();
    matches!(
        (parts.next(), parts.next()),
        (Some(Component::Normal(_)), None)
    ) && !n.contains(':')
}

/// Clip `file` in the voice folder `voice`; refused unless it is a plain name, so no sound outside the folder is read.
#[cfg_attr(
    not(feature = "voice"),
    allow(dead_code, reason = "Only the voice gates call it.")
)]
pub fn clip_in(voice: &Path, file: &str) -> Result<PathBuf, String> {
    if plain_name(file) {
        Ok(voice.join(file))
    } else {
        Err(format!("{file:?} is no plain name in the voice folder"))
    }
}

#[cfg(test)]
mod tests;
