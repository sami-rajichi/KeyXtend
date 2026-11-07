//! What the panel tests share: a small notebook, an open panel of 5 rows a page, and the view after every step, as the
//! module draws it after each key or click.

use std::sync::Arc;

use kx_module_api::Redacted;
use kx_module_api::ports::{EntryKind, EntryRow, FillAct, FillClick, FillPage, PanelKey, Sheet};
use kx_platform_fake::FakeRandom;

use super::{Desk, Do, Panel, PanelConfig};
use crate::book::Notebook;
use crate::entry::{Draft, Stamp};
use crate::fixture::{LIMITS, draft, field, own};
use crate::view::build;

/// The plan's page of 5 rows, the default limits and Make one's default length.
pub(crate) const CONFIG: PanelConfig = PanelConfig {
    rows: 5,
    limits: LIMITS,
    make_chars: 20,
};
/// The time the tests save at.
pub(crate) const NOW: Stamp = Stamp::from_unix_ms(1_762_000_000_000);

/// A notebook, an open panel and the last view.
pub(crate) struct Rig {
    pub book: Notebook,
    pub random: FakeRandom,
    pub panel: Panel,
    pub page: Arc<FillPage>,
}

impl Rig {
    /// An open panel with `cfg` over `drafts`, saved in order under the default limits, as before a setting was lowered.
    pub fn with(cfg: PanelConfig, drafts: Vec<Draft>) -> Self {
        let (mut book, random) = (Notebook::default(), FakeRandom::new(7));
        for d in drafts {
            book.add(d, NOW, &random, &LIMITS).unwrap();
        }
        let mut panel = Panel::new(cfg);
        panel.open();
        let page = build(&mut panel, &book).0;
        Self {
            book,
            random,
            panel,
            page,
        }
    }

    /// An open panel of 5 rows over `drafts`.
    pub fn new(drafts: Vec<Draft>) -> Self {
        Self::with(CONFIG, drafts)
    }

    /// An open panel over the shared sample.
    pub fn sample() -> Self {
        Self::new(sample())
    }

    /// Draws the view again, as the module does after each change.
    pub fn draw(&mut self) -> &FillPage {
        self.page = build(&mut self.panel, &self.book).0;
        &self.page
    }

    /// Hands in `key`, then draws.
    pub fn key(&mut self, key: &PanelKey) -> Do {
        let done = self.panel.key(key, &self.book);
        self.draw();
        done
    }

    /// Types `text` as letters from our keyboard.
    pub fn text(&mut self, text: &str) -> Do {
        self.key(&PanelKey::Text(Redacted::new(text.to_owned())))
    }

    /// Clicks `click` on the last view, then draws.
    pub fn click(&mut self, click: FillClick) -> Do {
        self.act(self.page.round, click)
    }

    /// Clicks `click` on the view before the last one, then draws.
    pub fn stale(&mut self, click: FillClick) -> Do {
        self.act(self.page.round.wrapping_sub(1), click)
    }

    /// Hands in `click` named for `round`, then draws.
    fn act(&mut self, round: u32, click: FillClick) -> Do {
        let mut desk = Desk {
            book: &mut self.book,
            random: &self.random,
            now: NOW,
        };
        let done = self.panel.act(FillAct { round, click }, &mut desk);
        self.draw();
        done
    }

    /// The id of the entry named `name`.
    pub fn id(&self, name: &str) -> u64 {
        let found = self
            .book
            .entries()
            .iter()
            .find(|e| e.name().expose() == name);
        found.map(crate::entry::Entry::id).unwrap()
    }

    /// The list's rows, or none on another page.
    pub fn rows(&self) -> &[EntryRow] {
        match &self.page.sheet {
            Sheet::List(rows) => rows,
            _ => &[],
        }
    }

    /// The names the list shows, in order.
    pub fn names(&self) -> Vec<String> {
        self.rows()
            .iter()
            .map(|r| r.name.expose().clone())
            .collect()
    }
}

/// One entry of each kind: a login, a person, a Wi-Fi, a card, a note and an Other with an own hidden label.
pub(crate) fn sample() -> Vec<Draft> {
    vec![
        draft(
            "Bank",
            EntryKind::Login,
            vec![
                field("user", "alex@example.com", false),
                field("password", "hunter2", true),
                field("website", "bank.example", false),
            ],
        ),
        draft(
            "Alex Martin",
            EntryKind::Person,
            vec![
                field("full-name", "Alex Martin", false),
                field("email", "alex@example.com", false),
                field("phone", "+33 6 12", false),
            ],
        ),
        draft(
            "Home",
            EntryKind::Wifi,
            vec![
                field("network", "HomeNet", false),
                field("password", "wifi-pass", true),
            ],
        ),
        draft(
            "Library",
            EntryKind::Card,
            vec![field("number", "1234", true)],
        ),
        draft("Gate", EntryKind::Note, vec![field("text", "4321", false)]),
        draft("Locker", EntryKind::Other, vec![own("Code", "999", true)]),
    ]
}
