//! The Voice panel as QML reads it (design "The gear and the Voice panel"): a header with Back, the page's name and line
//! and Close, a line above the list, the note band, the page's rows, then a footer line and its buttons, tick then act.
//! The home list lives here; the other pages build their own part.

use super::acts::CONFIRM;
use super::parts::{Button, Hint, Lang, Tone, Top, word};
use super::{device, lists, options, sheets};
use crate::I18n;
use crate::i18n::clean;
use crate::i18n::keys::vars::{DATE, DEVICE, LANG};
use crate::i18n::keys::{
    VOICE_AGAIN_HINT, VOICE_AUTO_CPU, VOICE_AUTO_ON, VOICE_CHANGED, VOICE_CPU, VOICE_DEFAULTS,
    VOICE_FOR_LANG, VOICE_LANG, VOICE_LAST_RUN, VOICE_NEVER, VOICE_NO, VOICE_NO_MODEL, VOICE_TEXTS,
    VOICE_TITLE, VOICE_YES, WINDOW_VOICE_PANEL, voice_tab_id,
};
use kx_module_api::ports::{
    DeviceLabel, DeviceName, PanelButton, VoiceHome, VoicePanel, VoiceSheet, VoiceTab,
};
use serde::Serialize;

/// The home list's page id.
pub(super) const HOME: &str = "home";
/// The folder list's page id.
pub(super) const FOLDERS: &str = "folders";

/// The panel, as JSON.
#[derive(Serialize)]
pub(super) struct Panel {
    page: &'static str,
    window: String,
    title: String,
    sub: String,
    lang: Lang,
    back: Option<Button>,
    close: Button,
    top: Option<Top>,
    note: Option<Hint>,
    extra: Option<Button>,
    line: Option<String>,
    foot: Vec<Button>,
    pages: Pages,
    ticked: u16,
    sheet: Sheet,
}

/// Where the shown list is: the page from 0, of `count`; the pager words its own number and arrows.
#[derive(Serialize)]
struct Pages {
    at: u16,
    count: u16,
}

/// A page's body, by what it is.
#[derive(Serialize)]
#[serde(tag = "is", rename_all = "lowercase")]
pub(super) enum Sheet {
    Home {
        rows: Vec<HomeRow>,
    },
    Models {
        rows: Vec<sheets::ModelView>,
        untested: Option<String>,
        slow: Option<String>,
        room: Option<sheets::Room>,
    },
    Device {
        rows: Vec<device::DeviceView>,
    },
    Ready {
        intro: String,
        text: Option<String>,
        line: String,
        recording: Option<String>,
    },
    Running {
        rows: Vec<lists::RunView>,
    },
    Results {
        head: Vec<String>,
        rows: Vec<lists::ResultView>,
    },
    History {
        rows: Vec<lists::HistoryView>,
        empty: Option<String>,
        // Pin reads Unpin, as every ticked text is pinned.
        unpin: bool,
    },
    Options {
        rows: Vec<options::OptionView>,
    },
    Folders {
        rows: Vec<String>,
    },
}

/// A home row: the page it opens, its name and its present value.
#[derive(Serialize)]
pub(super) struct HomeRow {
    tab: &'static str,
    label: String,
    value: String,
}

/// What a page brings to the panel's frame.
pub(super) struct Page {
    pub(super) id: &'static str,
    pub(super) title: String,
    pub(super) sub: String,
    pub(super) top: Option<Top>,
    pub(super) extra: Option<Button>,
    pub(super) line: Option<String>,
    pub(super) foot: Vec<Button>,
    pub(super) sheet: Sheet,
}

impl Page {
    /// Page `id` named `title` over `sub`, showing `sheet`, with nothing else yet.
    pub(super) const fn new(id: &'static str, title: String, sub: String, sheet: Sheet) -> Self {
        Self {
            id,
            title,
            sub,
            top: None,
            extra: None,
            line: None,
            foot: Vec::new(),
            sheet,
        }
    }
}

/// The words and facts a page needs.
pub(super) struct Cx<'a> {
    pub(super) t: &'a I18n,
    pub(super) panel: &'a VoicePanel,
    /// Compare's set text in the language in front, if it has one.
    pub(super) read: Option<&'a str>,
}

impl Cx<'_> {
    /// Panel button `b` with its own word, acting while the panel lists it.
    pub(super) fn button(&self, b: PanelButton) -> Button {
        Button::panel(self.t, b, &self.panel.acts)
    }

    /// Panel button `b` labelled with text `key`, acting while the panel lists it.
    pub(super) fn button_as(&self, b: PanelButton, key: &str) -> Button {
        Button::panel_as(b, self.t.text(key), &self.panel.acts)
    }

    /// Yes, in `tone`, and No for the shown question.
    pub(super) fn ask(&self, tone: Tone) -> Vec<Button> {
        let yes = Button::new(CONFIRM, "", 1, self.t.text(VOICE_YES)).tone(tone);
        vec![yes, Button::new(CONFIRM, "", 0, self.t.text(VOICE_NO))]
    }

    /// The language in front as the panel's sentences name it.
    pub(super) fn lang(&self) -> String {
        self.t.voice_lang(&self.panel.lang)
    }

    /// A page's line naming the language in front.
    pub(super) fn for_lang(&self) -> String {
        self.t.voice_with(VOICE_FOR_LANG, &[(LANG, self.lang())])
    }

    /// Tab `tab`'s name.
    pub(super) fn tab(&self, tab: VoiceTab) -> String {
        self.t.text(&word(voice_tab_id(tab)))
    }

    /// The device `d` names.
    pub(super) fn device(&self, d: &DeviceLabel) -> String {
        let t = self.t;
        match d {
            DeviceLabel::Auto(DeviceName::Cpu) => t.text(VOICE_AUTO_CPU),
            DeviceLabel::Auto(DeviceName::Card(name)) => {
                t.voice_with(VOICE_AUTO_ON, &[(DEVICE, clean(name))])
            }
            DeviceLabel::Cpu => t.text(VOICE_CPU),
            DeviceLabel::Card(name) => clean(name),
        }
    }
}

/// `panel` with every text in `t`; `read` is Compare's set text in the language in front, if it has one.
pub(super) fn of(panel: &VoicePanel, t: &I18n, read: Option<&str>) -> Panel {
    let cx = Cx { t, panel, read };
    let page = match &panel.sheet {
        VoiceSheet::Home(h) => home(&cx, h),
        VoiceSheet::Models(m) => sheets::models(&cx, m),
        VoiceSheet::Device(d) => device::page(&cx, d),
        VoiceSheet::Folders(f) => sheets::folders(&cx, f),
        VoiceSheet::Compare(c) => lists::compare(&cx, c),
        VoiceSheet::History(h) => lists::history(&cx, h),
        VoiceSheet::Options(rows) => options::page(&cx, rows),
    };
    // Back and Close always act: the module names only the buttons that may dim.
    let header = |b| Button::panel(t, b, &[b]);
    Panel {
        page: page.id,
        window: t.text(WINDOW_VOICE_PANEL),
        title: page.title,
        sub: page.sub,
        lang: Lang::of(t, &panel.lang),
        back: (page.id != HOME).then(|| header(PanelButton::Back)),
        close: header(PanelButton::Close),
        top: page.top,
        note: panel.note.as_ref().map(|h| Hint::of(t, h)),
        extra: page.extra,
        line: page.line,
        foot: page.foot,
        pages: Pages {
            at: panel.pages.at,
            count: panel.pages.count,
        },
        ticked: panel.ticked,
        sheet: page.sheet,
    }
}

/// The home list: each page with its present value, and Try again.
fn home(cx: &Cx<'_>, h: &VoiceHome) -> Page {
    let t = cx.t;
    let value = |tab| match tab {
        VoiceTab::Models => h
            .model
            .as_ref()
            .map_or_else(|| t.text(VOICE_NO_MODEL), |m| clean(m.expose())),
        VoiceTab::Device => cx.device(&h.device),
        VoiceTab::Compare => h.compared.map_or_else(
            || t.text(VOICE_NEVER),
            |d| t.voice_with(VOICE_LAST_RUN, &[(DATE, t.voice_date(d))]),
        ),
        VoiceTab::History => t.counted(VOICE_TEXTS, count(h.history)),
        VoiceTab::Options if h.changed == 0 => t.text(VOICE_DEFAULTS),
        VoiceTab::Options => t.counted(VOICE_CHANGED, usize::from(h.changed)),
    };
    let rows = VoiceTab::ALL.map(|tab| HomeRow {
        tab: voice_tab_id(tab),
        label: cx.tab(tab),
        value: value(tab),
    });
    let sub = t.voice_with(VOICE_LANG, &[(LANG, cx.lang())]);
    let sheet = Sheet::Home { rows: rows.into() };
    let mut page = Page::new(HOME, t.text(VOICE_TITLE), sub, sheet);
    page.line = Some(t.text(VOICE_AGAIN_HINT));
    page.foot = vec![cx.button(PanelButton::Again).tone(Tone::Primary)];
    page
}

/// `n` as a count.
pub(super) fn count(n: u32) -> usize {
    usize::try_from(n).unwrap_or(usize::MAX)
}
