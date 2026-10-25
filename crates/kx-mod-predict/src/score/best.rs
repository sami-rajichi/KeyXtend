//! The best guesses so far: at most `limit` distinct texts, each with its best score, so a reading's thousands of
//! completions never need sorting.

use std::fmt;

/// The best distinct texts offered so far.
pub(super) struct Best {
    /// The most texts kept.
    limit: usize,
    /// Each kept text with its best score.
    items: Vec<(String, f64)>,
}

impl fmt::Debug for Best {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Best")
            .field("limit", &self.limit)
            .field("kept", &self.items.len())
            .finish()
    }
}

impl Best {
    /// An empty list keeping at most `limit` texts.
    pub(super) fn new(limit: usize) -> Self {
        Self {
            limit,
            items: Vec::with_capacity(limit),
        }
    }

    /// The most texts kept.
    pub(super) fn limit(&self) -> usize {
        self.limit
    }

    /// True when a text scoring `score` could enter or raise its own score.
    pub(super) fn wants(&self, score: f64) -> bool {
        self.limit > 0 && (self.items.len() < self.limit || score > self.min().1)
    }

    /// Keeps `text` at `score` when it beats the worst kept text; a text kept already takes its best score.
    pub(super) fn offer(&mut self, text: String, score: f64) {
        if let Some(item) = self.items.iter_mut().find(|item| item.0 == text) {
            item.1 = item.1.max(score);
            return;
        }
        if !self.wants(score) {
            return;
        }
        if self.items.len() == self.limit {
            let (at, _) = self.min();
            self.items.swap_remove(at);
        }
        self.items.push((text, score));
    }

    /// The kept texts, best first; equal scores go in text order.
    pub(super) fn into_texts(mut self) -> Vec<String> {
        self.items
            .sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        self.items.into_iter().map(|item| item.0).collect()
    }

    /// The place and score of the worst kept text; a score no text loses to when none is kept.
    fn min(&self) -> (usize, f64) {
        let worst = self
            .items
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.1.total_cmp(&b.1.1));
        worst.map_or((0, f64::NEG_INFINITY), |(at, item)| (at, item.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_shows_counts_never_the_guesses() {
        let mut best = Best::new(2);
        best.offer("secret".to_owned(), 1.0);
        let shown = format!("{best:?}");
        assert!(
            !shown.contains("secret") && shown.contains("kept: 1"),
            "{shown}"
        );
    }
}
