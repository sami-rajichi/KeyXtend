//! The cloud speech port from outside the crate (ADR-0047): only `Network` reaches it, an ask is built from public
//! parts, an address is only ever HTTPS, and a platform's run tells its notes until it is dropped.

use kx_module_api::Capability::{Microphone, Network, Voice};
use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudLimits, CloudNote, CloudRun, CloudShape, CloudSpeech,
    CloudSpeechKey, CloudTest, FormFields, HttpsUrl, JobLimits, KeyHeader,
};
use kx_module_api::{
    Capability, Manifest, ModuleCx, ModuleId, Redacted, ServiceError, ServiceKey, Zeroizing,
};
use kx_test_support::host::FakeHost;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A voice module that declares the network beside its own powers.
static VOICE: Manifest = Manifest {
    id: ModuleId::new("voice"),
    version: env!("CARGO_PKG_VERSION"),
    requires: &[CloudSpeechKey::ID],
    provides: &[],
    capabilities: &[Voice, Microphone, Network],
    settings: None,
};

/// A platform that writes each piece's place as its text, and counts the runs dropped.
#[derive(Default)]
struct Stub {
    dropped: Arc<AtomicUsize>,
}

/// A run of `Stub`; dropping it is its cancel.
struct StubRun(Arc<AtomicUsize>);

impl Drop for StubRun {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

impl CloudRun for StubRun {}

impl CloudSpeech for Stub {
    fn send(
        &self,
        ask: CloudAsk,
        tell: Box<dyn Fn(CloudNote) + Send>,
    ) -> Result<Box<dyn CloudRun>, CloudFault> {
        ask.check()?;
        for index in 0..u16::try_from(ask.pieces.len()).unwrap_or(u16::MAX) {
            let text = Redacted::new(Zeroizing::new(index.to_string()));
            tell(CloudNote::Piece { index, text });
        }
        tell(CloudNote::Done);
        Ok(Box::new(StubRun(Arc::clone(&self.dropped))))
    }

    fn test(&self, ask: CloudTest) -> Result<(), CloudFault> {
        ask.check()?;
        Err(CloudFault::Key)
    }
}

/// A host granted `granted`, holding the port as the platform would.
fn host(granted: &[Capability], stub: Arc<Stub>) -> FakeHost {
    let mut host = FakeHost::new(&VOICE);
    host.granted.extend(granted);
    let api: Arc<dyn CloudSpeech> = stub;
    let gate = CloudSpeechKey::CAPABILITY;
    host.services
        .insert(CloudSpeechKey::ID, (gate, Box::new(api)));
    host
}

/// Two pieces for Groq, with a key the user typed.
#[allow(clippy::unwrap_used, reason = "a test helper; Groq's address is HTTPS")]
fn ask() -> CloudAsk {
    let job = JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    };
    let shape = CloudShape::Form(FormFields {
        lang: "language".into(),
        list: false,
        fixed: Vec::new(),
    });
    let piece = || Redacted::new(Zeroizing::new(vec![0; 4]));
    CloudAsk {
        job,
        shape,
        url: HttpsUrl::new("https://api.groq.com/openai/v1/audio/transcriptions").unwrap(),
        header: KeyHeader::Bearer,
        key: Redacted::new(Zeroizing::new("gsk_contract".into())),
        model: Some("whisper-large-v3".into()),
        lang: "ar".into(),
        limits: CloudLimits {
            max_mb: 25,
            max_s: 600,
            per_min: 20,
            at_once: 2,
        },
        wait_s: 60,
        tries: 2,
        pieces: vec![piece(), piece()],
    }
}

#[test]
fn only_the_network_reaches_cloud_speech() {
    let mut bare = host(&[Voice, Microphone], Arc::default());
    let refused = ServiceError::NotGranted(CloudSpeechKey::ID, Network);
    let got = ModuleCx::new(&mut bare).service::<CloudSpeechKey>();
    assert_eq!(got.err(), Some(refused));
    let mut granted = host(&[Network], Arc::default());
    assert!(
        ModuleCx::new(&mut granted)
            .service::<CloudSpeechKey>()
            .is_ok()
    );
}

#[test]
fn a_run_tells_each_pieces_text_in_order_until_it_is_dropped() {
    let stub = Arc::new(Stub::default());
    let mut host = host(&[Network], Arc::clone(&stub));
    let cloud = ModuleCx::new(&mut host)
        .service::<CloudSpeechKey>()
        .unwrap();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let ear = Arc::clone(&heard);
    let tell = Box::new(move |note| ear.lock().unwrap().push(note));
    let run = cloud.send(ask(), tell).unwrap();
    let texts: Vec<String> = heard
        .lock()
        .unwrap()
        .iter()
        .map(|n| format!("{n:?}"))
        .collect();
    assert_eq!(texts.len(), 3, "two pieces, then Done: {texts:?}");
    assert!(
        texts[0].contains("index: 0") && texts[2] == "Done",
        "{texts:?}"
    );
    assert_eq!(stub.dropped.load(Ordering::SeqCst), 0);
    drop(run);
    assert_eq!(stub.dropped.load(Ordering::SeqCst), 1, "dropping cancels");
}

#[test]
fn a_bad_ask_never_starts_and_an_http_address_is_never_built() {
    let mut host = host(&[Network], Arc::default());
    let cloud = ModuleCx::new(&mut host)
        .service::<CloudSpeechKey>()
        .unwrap();
    let empty = CloudAsk {
        pieces: Vec::new(),
        ..ask()
    };
    let refused = cloud.send(empty, Box::new(|_| {}));
    assert_eq!(refused.err(), Some(CloudFault::Range("pieces")));
    let plain = "http://api.groq.com/openai/v1/audio/transcriptions";
    assert_eq!(HttpsUrl::new(plain), None, "edge 13");
}
