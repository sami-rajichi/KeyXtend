//! The random port: its open gate, its error's fixed text, and a source used as a trait object.

use super::*;

/// A source that fills every byte with one value, or fails.
struct Stuck(Option<u8>);

impl Random for Stuck {
    fn fill(&self, buf: &mut [u8]) -> Result<(), RandomError> {
        let byte = self.0.ok_or(RandomError)?;
        buf.fill(byte);
        Ok(())
    }
}

#[test]
fn randomness_needs_no_capability() {
    assert_eq!(RandomKey::ID.as_str(), "random");
    assert_eq!(RandomKey::CAPABILITY, None);
}

#[test]
fn a_source_fills_the_whole_buffer_through_the_trait() {
    let source: &dyn Random = &Stuck(Some(7));
    let mut buf = [0u8; 5];
    assert_eq!(source.fill(&mut buf), Ok(()));
    assert_eq!(buf, [7; 5]);
}

#[test]
fn a_failed_fill_says_so_in_fixed_words() {
    let mut buf = [0u8; 3];
    assert_eq!(Stuck(None).fill(&mut buf), Err(RandomError));
    assert_eq!(RandomError.to_string(), "the system gave no random bytes");
}
