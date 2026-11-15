//! The voice hosts' Windows side (ADR-0042, ADR-0044): the processor's real cores and each card's driver version for
//! the speech host, the disk for the models host, and the session watch's adapter lives in `session`.

mod cards;
mod cores;
mod disk;

pub use disk::WinDisk;
#[cfg(test)]
pub(crate) use disk::make_guarded;

use kx_platform::speech::Hardware;

/// What Windows tells the speech host: the real cores, counted once, and each card's driver as DXGI reads it.
#[derive(Clone, Copy, Debug)]
pub struct WinHardware {
    cores: usize,
}

impl WinHardware {
    /// Counts the real cores now.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cores: cores::real(),
        }
    }
}

impl Default for WinHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl Hardware for WinHardware {
    fn cores(&self) -> usize {
        self.cores
    }

    fn driver(&self, name: &str) -> Option<String> {
        cards::driver(name)
    }
}
