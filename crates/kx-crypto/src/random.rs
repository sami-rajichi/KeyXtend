//! The system's random bytes behind the `Random` port, for Quick-fill's ids and made passwords (ADR-0038).

use kx_module_api::ports::{Random, RandomError};

/// The OS's random source, through `getrandom`, on every system the app runs on.
#[derive(Copy, Clone, Debug, Default)]
pub struct OsRandom;

impl Random for OsRandom {
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError> {
        getrandom::fill(buf).map_err(|_| RandomError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes one fill takes: a data key's length.
    const LEN: usize = 32;

    #[test]
    fn two_fills_differ() {
        let (mut a, mut b) = ([0u8; LEN], [0u8; LEN]);
        assert_eq!(OsRandom.fill(&mut a), Ok(()));
        assert_eq!(OsRandom.fill(&mut b), Ok(()));
        assert_ne!(a, b, "two draws of 32 bytes never match");
    }

    #[test]
    fn an_empty_buffer_is_fine() {
        let source: &dyn Random = &OsRandom;
        assert_eq!(source.fill(&mut []), Ok(()));
    }
}
