//! The speech job over a scripted server and a still clock: the upload form, its status table, the reply reader, the
//! WAV wrap, the tries and the wait after a 429, a run's sound and texts, and the loop over pipes. None touches the
//! network or sleeps; replies are fixtures under `replies/`, and the key is a made-up one.

mod bench;
mod form;
mod job;
mod pipe;
mod props;
mod reply;
mod send;
mod table;
mod wav;

use super::Kit;
use super::clock::Clock;
use kx_ipc::{NetCall, NetForm, NetHeader, NetLimits, NetRun, NetShape, Wiped};
use kx_platform_net::Head;
use kx_platform_net::fake::{Act, FakeHttp};
use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use zeroize::Zeroizing;

/// A made-up key, never a real one.
const KEY: &[u8] = b"gsk_kx-test-key-not-real";
/// The key as text, to look for in what the job writes.
const KEY_TEXT: &str = "gsk_kx-test-key-not-real";
/// Groq's transcription address, as its row names it.
const URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// Groq's key test address.
const MODELS: &str = "https://api.groq.com/openai/v1/models";
/// The model and the language the tests send.
const MODEL: &str = "whisper-large-v3-turbo";
const LANG: &str = "fr";
/// The call's wait for one answer.
const WAIT_S: u16 = 60;
/// The status of a reply that holds what was asked.
const OK: u16 = 200;
/// The status of too many requests.
const BUSY: u16 = 429;

/// A clock that stands still at its `now` and keeps each wait it is asked for.
#[derive(Default)]
struct Still {
    now: AtomicU64,
    waits: Mutex<Vec<u64>>,
}

impl Clock for Still {
    fn now_ms(&self) -> u64 {
        self.now.load(Ordering::SeqCst)
    }

    fn wait(&self, ms: u64) {
        lock(&self.waits).push(ms);
    }
}

impl Still {
    /// Every wait so far, smallest first, as threads may ask in any order.
    fn waits(&self) -> Vec<u64> {
        let mut waits = lock(&self.waits).clone();
        waits.sort_unstable();
        waits
    }
}

/// `m`'s guard, even after a panic elsewhere.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Groq's form: `language` as one value, with no fixed field.
fn form() -> NetForm<'static> {
    NetForm {
        lang: Cow::Borrowed("language"),
        list: false,
        fixed: Vec::new(),
    }
}

/// A call to `url` in `shape`, the key as a bearer.
fn call_to(url: &'static str, shape: NetShape<'static>) -> NetCall<'static> {
    NetCall {
        shape,
        url: Cow::Borrowed(url),
        header: NetHeader::Bearer,
        key: Wiped::Kept(Zeroizing::new(KEY.to_vec())),
        wait_s: WAIT_S,
    }
}

/// Groq's transcription call.
fn call() -> NetCall<'static> {
    call_to(URL, NetShape::Form(form()))
}

/// A run of pieces of `sizes` bytes, `tries` per piece, at most `at_once` at once and `per_min` a minute.
fn run_of(sizes: &[u32], tries: u8, at_once: u8, per_min: u16) -> NetRun<'static> {
    NetRun {
        model: Some(Cow::Borrowed(MODEL)),
        lang: Cow::Borrowed(LANG),
        limits: NetLimits {
            max_mb: 25,
            max_s: 600,
            per_min,
            at_once,
        },
        tries,
        sizes: sizes.to_vec(),
    }
}

/// `n` bytes of sound, each `fill`.
fn sound(n: usize, fill: u8) -> Vec<u8> {
    vec![fill; n]
}

/// A reply of `status` holding the fixture `body`.
fn reply(status: u16, body: &[u8]) -> Act {
    Act::with(status, body)
}

/// A reply holding `text` as the form's JSON.
fn said(text: &str) -> Act {
    let body = serde_json::json!({ "text": text }).to_string();
    Act::with(OK, body.as_bytes())
}

/// A head of `status` with the headers `extra`.
fn head(status: u16, extra: &[(&str, &str)]) -> Head {
    let extra = extra
        .iter()
        .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()));
    Head {
        status,
        extra: extra.collect(),
        ..Head::default()
    }
}

/// A 429 answer with the headers `extra` and the fixture's rate message.
fn busy(extra: &[(&str, &str)]) -> Act {
    busy_with(extra, RATE)
}

/// A 429 answer with the headers `extra` and `body`.
fn busy_with(extra: &[(&str, &str)], body: &[u8]) -> Act {
    Act::Answer(
        head(BUSY, extra),
        vec![kx_platform_net::fake::Step::Bytes(body.to_vec())],
    )
}

/// A kit over `http` and `clock`.
fn kit(http: &Arc<FakeHttp>, clock: &Arc<Still>) -> Kit {
    Kit {
        http: http.clone(),
        clock: clock.clone(),
    }
}

/// The fixture replies, read at build time.
const TEXT: &[u8] = include_bytes!("tests/replies/text.json");
const ARABIC: &[u8] = include_bytes!("tests/replies/arabic.json");
const BLANK: &[u8] = include_bytes!("tests/replies/blank.json");
const NO_TEXT: &[u8] = include_bytes!("tests/replies/no-text.json");
const KEY_BAD: &[u8] = include_bytes!("tests/replies/key.json");
const RATE: &[u8] = include_bytes!("tests/replies/rate.json");
const RATE_HOUR: &[u8] = include_bytes!("tests/replies/rate-hour.json");
const CREDIT: &[u8] = include_bytes!("tests/replies/credit.json");
const LANGUAGE: &[u8] = include_bytes!("tests/replies/language.json");
const BAD: &[u8] = include_bytes!("tests/replies/bad.json");
const TOO_BIG: &[u8] = include_bytes!("tests/replies/too-big.json");
const DOWN: &[u8] = include_bytes!("tests/replies/down.json");
const LIST: &[u8] = include_bytes!("tests/replies/models.json");
const HOTSPOT: &[u8] = include_bytes!("tests/replies/hotspot.html");
