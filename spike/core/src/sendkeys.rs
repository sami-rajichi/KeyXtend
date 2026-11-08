//! Spike helper over the product injector (`kx-platform-windows`) until its phase moves it: paced Unicode typing
//! for voice (P11).

use std::time::Duration;

use kx_module_api::Redacted;
use kx_module_api::ports::{InputInjector, TextRoute};
use kx_platform_windows::WinInjector;

/// Types `text` as Unicode characters in one batch, whatever the target's layout.
fn text(text: &str) -> Result<(), String> {
    let text = Redacted::new(text.to_owned());
    WinInjector
        .text(&text, TextRoute::Unicode)
        .map_err(|e| e.to_string())
}

/// Types `text` one character at a time, `gap_ms` apart, for apps that drop a fast batch; 0 types one batch.
/// It blocks for about `gap_ms` per character, so call it off the UI thread.
pub fn text_paced(text: &str, gap_ms: u64) -> Result<(), String> {
    if gap_ms == 0 {
        return self::text(text);
    }
    for part in chars(text) {
        self::text(&part)?;
        std::thread::sleep(Duration::from_millis(gap_ms));
    }
    Ok(())
}

/// Each character of `text` on its own, so a surrogate pair is never split.
fn chars(text: &str) -> Vec<String> {
    text.chars().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paced_typing_keeps_each_character_whole() {
        assert_eq!(chars("aب😀"), ["a", "ب", "😀"]);
        assert!(chars("").is_empty());
    }
}
