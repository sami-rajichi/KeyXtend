//! The voice gate build only: the gate worker started with its clip after the voice job's words, so it plays the clip in
//! place of the microphone (G22). No shipped build holds it, and the stage refuses a program holding its flag.

mod told;

use std::path::Path;
use std::sync::Arc;

use kx_ipc::SOUND_FLAG;
use kx_module_api::Clock;
use kx_module_api::ports::Speech;
use kx_platform::speech::{Assets, WorkerSpeech};
use kx_platform::worker::{Launch, Piped, SpawnError, Spawner, VoiceFrames};
use kx_platform_windows::{WinChild, WinHardware, WinSpawner};
use told::Told;

/// The worker's spawner, with the sound flag and the clip after each launch's own words.
pub struct Played {
    worker: WinSpawner,
    extra: [String; 2],
}

impl Played {
    /// The worker `worker` starts, playing `clip`; `None` when the clip's path is not UTF-8.
    #[must_use]
    pub fn new(worker: WinSpawner, clip: &Path) -> Option<Self> {
        let clip = clip.to_str()?.to_owned();
        let extra = [SOUND_FLAG.to_owned(), clip];
        Some(Self { worker, extra })
    }

    /// `launch` with the sound flag and the clip after its own words.
    #[must_use]
    pub fn launch(&self, launch: &Launch) -> Launch {
        let mut out = launch.clone();
        out.args.extend_from_slice(&self.extra);
        out
    }
}

impl Spawner<VoiceFrames> for Played {
    type Worker = Piped<WinChild, VoiceFrames>;

    fn spawn(&self, launch: &Launch) -> Result<Self::Worker, SpawnError> {
        let (input, output, child) = self.worker.start(&self.launch(launch))?;
        Piped::new(input, output, child)
    }
}

/// Speech by the gate worker beside the program in `exe_dir`, on `clock`, with `assets`, playing `clip` and logging the
/// device of each dictation it writes; `None` when the clip's path is not UTF-8.
#[must_use]
pub fn speech(
    exe_dir: &Path,
    clock: Arc<dyn Clock>,
    assets: Assets,
    clip: &Path,
) -> Option<Arc<dyn Speech>> {
    let played = Played::new(WinSpawner::beside(exe_dir), clip)?;
    let hardware = Arc::new(WinHardware::new());
    let speech = WorkerSpeech::new(played, clock, assets, hardware);
    Some(Arc::new(Told(speech)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worker's words for the voice job, as the speech host gives them.
    fn voice() -> Launch {
        Launch {
            args: vec![kx_ipc::VOICE_JOB.to_owned()],
            memory_mb: 512,
        }
    }

    #[test]
    fn each_launch_gets_the_sound_flag_and_the_clip_after_its_own_words() {
        let clip = Path::new(r"D:\clips\check-fr.wav");
        let played = Played::new(WinSpawner::at(r"D:\app\w.exe".into()), clip).unwrap();
        let got = played.launch(&voice());
        let want = [kx_ipc::VOICE_JOB, SOUND_FLAG, r"D:\clips\check-fr.wav"];
        assert_eq!(got.args, want);
        assert_eq!(got.memory_mb, voice().memory_mb, "the cap stays");
    }

    #[test]
    fn a_clip_whose_path_is_not_utf8_is_refused() {
        use std::os::windows::ffi::OsStringExt;
        let odd = std::path::PathBuf::from(std::ffi::OsString::from_wide(&[0xD800]));
        assert!(Played::new(WinSpawner::at(r"D:\app\w.exe".into()), &odd).is_none());
    }

    #[test]
    fn a_utf8_clip_always_gives_played_speech_and_starts_no_worker_yet() {
        let clock = Arc::new(kx_platform_fake::FakeClock::new(
            kx_module_api::Mono::default(),
        ));
        let (dir, clip) = (Path::new(r"D:\app"), Path::new(r"D:\clips\check-ar.wav"));
        let speech = speech(dir, clock, Assets::beside(dir), clip);
        assert!(speech.is_some(), "never the microphone in its place");
    }
}
