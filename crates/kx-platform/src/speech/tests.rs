//! The speech host over scripted voice workers on a clock that moves only when a test moves it: notes are told as the
//! port says, and tests wait on channels, never sleep.

use super::*;
use crate::worker::fake::voice::{done, piece, shut};
use crate::worker::fake::{Call, Hands, Script, VoiceAct};
use crate::worker::{Launch, SpawnError, Spawner};
use kx_ipc::{
    MAX_SOUND_BYTES, SoundPiece, Stage, VOICE_JOB, VoiceAsk, VoiceFault, VoiceNote, Wiped,
};
use kx_module_api::ports::{Memory, SpeechModel, SpeechNote};
use kx_module_api::{Redacted, Zeroizing};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

mod jobs;
mod life;
mod once;
mod props;
mod spoken;

/// How long a test waits for a note; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// The real cores the test hardware has.
const CORES: usize = 8;
/// A test model's memory on the processor, in MB.
const MODEL_MB: u32 = 700;
/// The two test model files.
const FILE_A: &str = r"D:\Models\a.gguf";
const FILE_B: &str = r"D:\Models\b.gguf";
/// The runner the test models name.
const RUNNER: &str = "transcribe";
/// The language code the test models take.
const LANG: &str = "fr";
/// The shipped detector and the English and German clips in the tests.
const DETECTOR: &str = r"C:\App\silero.gguf";
const CLIP: &str = r"C:\App\check-en.wav";
const CLIP_DE: &str = r"C:\App\check-de.wav";
/// The PC's memory as the test system reads it.
const SYSTEM: Memory = Memory {
    total_mb: 16_384,
    free_mb: 9_626,
};
/// A card the engine lists, its system name, and the driver the system gives it.
const CARD: &str = "Vulkan0";
const CARD_NAME: &str = "Card A";
const DRIVER: &str = "32.0.15.6094";
/// A sound piece's test bytes, whole samples.
const SOUND: [u8; 4] = [1, 2, 3, 4];
/// A ten-minute recording, the longest a test dictation keeps, and its bytes as hand-over sound.
const TEN_MIN_S: u16 = 600;
const TEN_MIN_BYTES: usize = TEN_MIN_S as usize * kx_ipc::SOUND_HZ * kx_ipc::SAMPLE_BYTES;

/// Test limits, each inside its range; the real ones come from `[voice]`.
fn limits() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

/// A test model in `file`.
fn model(file: &str) -> SpeechModel {
    SpeechModel {
        file: Redacted::new(PathBuf::from(file)),
        lang: LANG.to_owned(),
        runner: RUNNER.to_owned(),
        cpu_mb: MODEL_MB,
    }
}

/// A dictation on `file`'s model, or listening only without one.
fn ask(file: Option<&str>) -> SpeechAsk {
    SpeechAsk {
        job: limits(),
        model: file.map(model),
        card: None,
        threads: 0,
        piece_s: 30,
        pad_ms: 300,
        level_hz: 30,
        record_max_s: TEN_MIN_S,
        silence_stop_s: None,
    }
}

/// The test system: `CORES` real cores, `SYSTEM`'s memory, and `DRIVER` for the card named `CARD_NAME`.
struct Box8;

impl Hardware for Box8 {
    fn cores(&self) -> usize {
        CORES
    }

    fn memory(&self) -> Option<Memory> {
        Some(SYSTEM)
    }

    fn driver(&self, name: &str) -> Option<String> {
        (name == CARD_NAME).then(|| DRIVER.to_owned())
    }
}

/// A system that tells no memory and no driver.
struct Quiet;

impl Hardware for Quiet {
    fn cores(&self) -> usize {
        CORES
    }

    fn memory(&self) -> Option<Memory> {
        None
    }

    fn driver(&self, _: &str) -> Option<String> {
        None
    }
}

/// The shipped files in the tests, the English clip staged before the German one.
fn assets() -> Assets {
    let clips = vec![
        ("en".to_owned(), PathBuf::from(CLIP)),
        ("de".to_owned(), PathBuf::from(CLIP_DE)),
    ];
    Assets {
        detector: PathBuf::from(DETECTOR),
        clips,
    }
}

/// Starts each worker from the next script in turn, the last one for every start past them.
#[derive(Clone)]
struct Turns(Arc<(Vec<Script<VoiceFrames>>, AtomicUsize)>);

impl Spawner<VoiceFrames> for Turns {
    type Worker = <Script<VoiceFrames> as Spawner<VoiceFrames>>::Worker;

    fn spawn(&self, launch: &Launch) -> Result<Self::Worker, SpawnError> {
        let (scripts, next) = &*self.0;
        let at = next.fetch_add(1, Ordering::SeqCst).min(scripts.len() - 1);
        scripts[at].spawn(launch)
    }
}

/// A speech host over `scripts` in turn, on the clock `clock`.
fn host(scripts: &[Script<VoiceFrames>], clock: &Hands) -> WorkerSpeech<Turns> {
    let turns = Turns(Arc::new((scripts.to_vec(), AtomicUsize::new(0))));
    WorkerSpeech::new(turns, Arc::new(clock.clone()), assets(), Arc::new(Box8))
}

/// One script, the clock and a speech host over that script alone.
fn solo() -> (Script<VoiceFrames>, Hands, WorkerSpeech<Turns>) {
    let (script, clock) = (Script::default(), Hands::default());
    let speech = host(std::slice::from_ref(&script), &clock);
    (script, clock, speech)
}

/// A run's callback and what it hears.
fn ear() -> (Tell, Receiver<SpeechNote>) {
    let (said, heard) = mpsc::channel();
    (Box::new(move |note| drop(said.send(note))), heard)
}

/// The next `n` notes heard.
fn next(heard: &Receiver<SpeechNote>, n: usize) -> Vec<SpeechNote> {
    (0..n)
        .map(|_| heard.recv_timeout(LONG).expect("told in time"))
        .collect()
}

/// The voice asks a script's workers took, in order.
fn asks(script: &Script<VoiceFrames>) -> Vec<VoiceAsk<'static>> {
    let voice = |call| match call {
        Call::Voice(ask) => Some(ask),
        _ => None,
    };
    script.calls().into_iter().filter_map(voice).collect()
}

/// The memory a job of `ask` is capped at.
fn cap(ask: &SpeechAsk) -> u32 {
    Key::of(ask).launch(0).memory_mb
}

/// The voice job capped at `memory_mb`.
fn job(memory_mb: u32) -> Launch {
    Launch {
        args: vec![VOICE_JOB.to_owned()],
        memory_mb,
    }
}

/// A piece of a held recording, the first of two.
fn first_sound(id: u32) -> VoiceNote {
    sound(id, 0, false, SOUND.to_vec())
}

/// The last of two pieces.
fn last_sound(id: u32) -> VoiceNote {
    sound(id, 1, true, SOUND.to_vec())
}

/// A sound note of run `id`.
fn sound(id: u32, index: u16, last: bool, bytes: Vec<u8>) -> VoiceNote {
    let bytes = Wiped::Kept(Zeroizing::new(bytes));
    let piece = SoundPiece { index, last, bytes };
    VoiceNote::Sound { id, piece }
}

/// The microphone closed for `id` after Stop, every piece told.
fn finished() -> VoiceAct {
    VoiceAct::Say(vec![shut, piece, done])
}

/// The job failed `id` with no recording.
fn no_sound(id: u32) -> VoiceNote {
    let fault = VoiceFault::NoSound;
    VoiceNote::Fault { id, fault }
}

/// The stage `stage` of run `id`.
fn stage(id: u32, stage: Stage) -> VoiceNote {
    VoiceNote::Stage { id, stage }
}

/// A full-size piece of a long recording, re-indexed by the host when handed in.
fn big_piece(id: u32) -> VoiceNote {
    sound(id, 0, false, vec![0; MAX_SOUND_BYTES])
}

/// The last piece of a ten-minute recording.
fn big_last(id: u32) -> VoiceNote {
    sound(id, 0, true, vec![0; TEN_MIN_BYTES % MAX_SOUND_BYTES])
}

/// Hears each later call of `script`'s workers, as a job's goodbye comes on its own thread.
fn calls(script: &Script<VoiceFrames>) -> Receiver<Call> {
    let (to, seen) = mpsc::channel();
    script.tell(to);
    seen
}

/// Waits for a goodbye among the calls `seen` hears.
fn said_bye(seen: &Receiver<Call>) -> bool {
    std::iter::from_fn(|| seen.recv_timeout(LONG).ok()).any(|call| call == Call::Bye)
}
