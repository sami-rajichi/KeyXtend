//! The Voice panel's pages the Qt Quick tests draw, as the module would hand them: the home list, Models with each
//! mark, a download and the folder line, Compare's three steps and History.

use kx_module_api::ports::{
    CompareMic, CompareRow, CompareRun, CompareSheet, Date, DeviceLabel, DeviceName, FetchState,
    FolderLine, HistoryRow, HistorySheet, ModelRow, ModelSide, ModelState, ModelsSheet, RunStep,
    VoiceHome, VoiceSheet, VoiceTab, When,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

/// Bytes in a megabyte, and the models drive's free room.
const MB: u64 = 1 << 20;
const FREE: u64 = 212_000 * MB;
/// The day Compare last ran, and the day History's texts were dictated.
const RAN: (u16, u8, u8) = (2026, 11, 12);
const TODAY: (u16, u8, u8) = (2026, 11, 15);

/// A day in the fixtures.
fn day((y, m, d): (u16, u8, u8)) -> Date {
    Date::new(y, m, d).expect("a date")
}

/// `text`, redacted.
fn secret(text: &str) -> Redacted<String> {
    Redacted::new(text.to_owned())
}

/// A spoken `text`, redacted and wiped on drop.
fn spoken(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// The home list: Parakeet ready, Automatic on the processor, run on the 12th, 12 texts, two options changed.
pub fn home() -> VoiceSheet {
    VoiceSheet::Home(VoiceHome {
        model: Some(secret("Parakeet TDT 0.6b v3")),
        device: DeviceLabel::Auto(DeviceName::Cpu),
        compared: Some(day(RAN)),
        history: 12,
        changed: 2,
        cloud: None,
        tabs: VoiceTab::ALL[..5].to_vec(),
    })
}

/// Model `name` of `mb` megabytes in `state`, not the default, unused and unticked.
fn model(name: &str, mb: u64, state: ModelState) -> ModelRow {
    ModelRow {
        name: secret(name),
        bytes: mb * MB,
        default: false,
        state,
        used: false,
        tick: false,
        fetch: None,
        cloud: None,
    }
}

/// The models folder on D:.
fn folder() -> FolderLine {
    FolderLine {
        path: secret(r"D:\KeyXtend\models"),
        free: FREE,
    }
}

/// The first page: the default in use, a ready model ticked unless `used_ticked`, one to download, one downloading
/// and one too big.
fn first_page(used_ticked: bool) -> Vec<ModelRow> {
    let mut used = model("Parakeet TDT 0.6b v3", 740, ModelState::Ready);
    (used.default, used.used, used.tick) = (true, true, used_ticked);
    let mut ready = model("Whisper Large v3 Turbo", 620, ModelState::Ready);
    ready.tick = !used_ticked;
    let mut fetching = model("Whisper Large v3", 1160, ModelState::Missing);
    fetching.fetch = Some(FetchState {
        done: 534 * MB,
        total: 1160 * MB,
        paused: false,
    });
    let unfit = ModelState::NotHere {
        need_mb: 3277,
        have_mb: 4096,
        free_mb: 2867,
    };
    let missing = model("Cohere Transcribe", 2410, ModelState::Missing);
    vec![
        used,
        ready,
        missing,
        fetching,
        model("Qwen3-ASR 1.7B", 2190, unfit),
    ]
}

/// The second page: an other file and a paused download.
fn second_page() -> Vec<ModelRow> {
    let mut paused = model("Canary 1b v2", 1140, ModelState::Missing);
    paused.fetch = Some(FetchState {
        done: 228 * MB,
        total: 1140 * MB,
        paused: true,
    });
    vec![model("ggml-medium.bin", 1530, ModelState::Other), paused]
}

/// Models' page `page`, the model in use ticked while `used_ticked`, under the folder line.
pub fn models(page: u16, used_ticked: bool) -> VoiceSheet {
    let rows = if page == 0 {
        first_page(used_ticked)
    } else {
        second_page()
    };
    VoiceSheet::Models(ModelsSheet {
        rows,
        best: Some("Parakeet TDT 0.6b v3".to_owned()),
        untested: false,
        slow: false,
        folder: Some(folder()),
        offer: None,
        side: ModelSide::Local,
        band: None,
    })
}

/// A language whose default is Not for this PC: its line says local models will be very slow (owner, 2026-11-15).
pub fn slow() -> VoiceSheet {
    let unfit = ModelState::NotHere {
        need_mb: 5120,
        have_mb: 4096,
        free_mb: 2867,
    };
    let mut default = model("Cohere Transcribe Arabic", 2410, unfit);
    default.default = true;
    let ready = model("Whisper Large v3 Turbo", 620, ModelState::Ready);
    VoiceSheet::Models(ModelsSheet {
        rows: vec![default, ready],
        best: Some("Cohere Transcribe Arabic".to_owned()),
        untested: true,
        slow: true,
        folder: Some(folder()),
        offer: None,
        side: ModelSide::Local,
        band: None,
    })
}

/// The first download asks for the models folder.
pub fn offer() -> VoiceSheet {
    let VoiceSheet::Models(mut m) = models(0, false) else {
        unreachable!("models() gives the Models page");
    };
    (m.folder, m.offer) = (None, Some(folder()));
    VoiceSheet::Models(m)
}

/// Compare's set text, ready on six models.
pub fn ready() -> VoiceSheet {
    VoiceSheet::Compare(CompareSheet::Ready {
        models: 6,
        device: DeviceLabel::Auto(DeviceName::Cpu),
        mic: CompareMic::Closed,
        cloud: Vec::new(),
        send: Vec::new(),
    })
}

/// Compare running its third model of six: one done, one failed, one writing and two waiting.
pub fn running() -> VoiceSheet {
    let run = |name: &str, step| CompareRun {
        name: secret(name),
        step,
        why: None,
    };
    VoiceSheet::Compare(CompareSheet::Running {
        at: 3,
        of: 6,
        name: secret("Canary 1b v2"),
        rows: vec![
            run("Whisper Small", RunStep::Done { wait_ms: 2100 }),
            run("Whisper Large v3 Turbo", RunStep::Failed),
            run("Canary 1b v2", RunStep::Writing),
            run("Qwen3-ASR 0.6B", RunStep::Waiting),
            run("Parakeet TDT 0.6b v3", RunStep::Waiting),
        ],
    })
}

/// Compare's results on the processor, the second row ticked; with no set text, each model's text and no count.
pub fn results(free: bool) -> VoiceSheet {
    let row = |name: &str, wrong: u16, wait_ms, mb, tick| CompareRow {
        name: secret(name),
        wrong: (!free).then_some(wrong),
        words: 28,
        wait_ms,
        mb,
        text: free.then(|| spoken("The evening train leaves at seven.")),
        tick,
        cloud: None,
    };
    VoiceSheet::Compare(CompareSheet::Results {
        day: day(RAN),
        device: DeviceLabel::Auto(DeviceName::Cpu),
        rows: vec![
            row("Whisper Small", 0, 2100, 512, false),
            row("Whisper Large v3 Turbo", 0, 4000, 512, true),
            row("Canary 1b v2", 1, 1800, 1229, false),
        ],
    })
}

/// Text `id` said in `lang` at `hour`, by `model`.
fn said(id: u64, text: &str, lang: &str, hour: u8, model: &str) -> HistoryRow {
    let ar = lang == "ar";
    HistoryRow {
        id,
        text: spoken(text),
        rtl: ar,
        arabic: ar,
        when: When {
            days_ago: 0,
            date: day(TODAY),
            hour,
            minute: 32,
        },
        lang: lang.to_owned(),
        model: secret(model),
        pinned: false,
        tick: None,
        cloud: None,
    }
}

/// History's first page of 12 texts: an English, a French and an Arabic text, the first two ticked in that order and
/// the French one pinned; with the Delete-all question while `confirm`, or empty while `empty`.
pub fn history(confirm: bool, empty: bool) -> VoiceSheet {
    let mut en = said(
        12,
        "Thanks for the update. I will send the signed form on Thursday.",
        "en",
        14,
        "Parakeet TDT 0.6b v3",
    );
    let mut fr = said(
        11,
        "Bonjour Maya, la réunion est déplacée à jeudi à dix heures.",
        "fr",
        11,
        "Parakeet TDT 0.6b v3",
    );
    let ar = said(
        10,
        "شكرًا جزيلًا، سأرسل الملف مساء اليوم بعد الاجتماع.",
        "ar",
        9,
        "Cohere Transcribe Arabic",
    );
    (en.tick, fr.tick, fr.pinned) = (Some(2), Some(1), true);
    let (rows, total) = if empty {
        (vec![], 0)
    } else {
        (vec![en, fr, ar], 12)
    };
    VoiceSheet::History(HistorySheet {
        rows,
        total,
        unpinned: total.saturating_sub(1),
        unpin: false,
        confirm,
    })
}
