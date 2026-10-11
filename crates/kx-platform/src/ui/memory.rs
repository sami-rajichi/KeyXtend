//! The memory port: after a look change, the heaps' free pages go back to the OS (ADR-0023).

use super::UiError;

/// Gives the free pages the heaps keep committed back to the OS; systems whose heaps do that alone keep the default,
/// which does nothing.
pub trait Memory: Send + Sync {
    /// Asks the OS to take back the heaps' free pages; pages in use stay, so it never trims the working set
    /// (ADR-0021).
    ///
    /// # Errors
    /// `Refused` when the OS says no; the free pages then stay committed, which costs memory but nothing else.
    fn give_back(&self) -> Result<(), UiError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A system whose heaps give free pages back alone.
    struct Alone;

    impl Memory for Alone {}

    #[test]
    fn a_system_that_keeps_the_default_gives_nothing_back_and_succeeds() {
        assert_eq!(Alone.give_back(), Ok(()));
    }
}
