//! The OS-neutral half of a field watch (ADR-0033): the run and its thread, each read's deadline, and the rules every
//! OS shares. An adapter only says what box has focus and whether it is ours (`Probe`), and reads its text (`Reader`).

mod run;

use std::time::Instant;

use kx_module_api::ports::{FieldChange, FieldError};

use run::Ask;
pub use run::{Job, Shared, Tell, spawn};

/// What an adapter tells about one element that has or took the focus.
pub trait Probe {
    /// One element, as the OS hands it over.
    type El;

    /// Whether `el` shows in our own process.
    fn ours(&self, el: &Self::El) -> bool;

    /// The kind and id of box `el`.
    fn change(&self, el: &Self::El) -> FieldChange;
}

/// What an adapter reads on the watch's thread, each read by its deadline `by`; it never logs what it reads.
pub trait Reader: Probe {
    /// The element with the keyboard focus, in any app.
    ///
    /// # Errors
    /// `TimedOut` or `Refused`, as the OS answers.
    fn focused(&self) -> Result<Self::El, FieldError>;

    /// At most `max_words` words before the caret of `el`; `None` when `el` is no text box or has no caret.
    ///
    /// # Errors
    /// `TimedOut` past `by`, `Refused` when the app refuses.
    fn before_cursor(
        &self,
        el: &Self::El,
        max_words: u8,
        by: Instant,
    ) -> Result<Option<String>, FieldError>;

    /// Whether the text before the caret of `el` ends with `text`; `None` when it cannot tell.
    ///
    /// # Errors
    /// `TimedOut` past `by`, `Refused` when the app refuses.
    fn ends_with(&self, el: &Self::El, text: &str, by: Instant)
    -> Result<Option<bool>, FieldError>;
}

/// Answers `job` from the focused element: one past its deadline is `TimedOut` unstarted, and focus in our own process
/// reads as `None`, as it keeps the last box.
pub fn serve<R: Reader>(reader: &R, job: Job) {
    match job.0 {
        Ask::Before {
            max_words,
            by,
            reply,
        } => {
            let read = on_focus(reader, by, |el| reader.before_cursor(el, max_words, by));
            let _ = reply.send(read);
        }
        Ask::Ends { text, by, reply } => {
            let read = on_focus(reader, by, |el| reader.ends_with(el, text.expose(), by));
            let _ = reply.send(read);
        }
    }
}

/// `read` on the focused element, unless `by` has passed; `None` when focus is in our own process.
fn on_focus<R: Reader, T>(
    reader: &R,
    by: Instant,
    read: impl FnOnce(&R::El) -> Result<Option<T>, FieldError>,
) -> Result<Option<T>, FieldError> {
    late(by)?;
    let el = reader.focused()?;
    if reader.ours(&el) {
        return Ok(None);
    }
    read(&el)
}

/// `TimedOut` once `by` has passed.
///
/// # Errors
/// `TimedOut` past `by`.
pub fn late(by: Instant) -> Result<(), FieldError> {
    if Instant::now() > by {
        Err(FieldError::TimedOut)
    } else {
        Ok(())
    }
}

/// `text` after its first word, which a cut at the read cap may have split; empty when it holds one word.
#[must_use]
pub fn after_first(text: &str) -> &str {
    text.find(char::is_whitespace).map_or("", |at| &text[at..])
}

#[cfg(test)]
mod tests;
