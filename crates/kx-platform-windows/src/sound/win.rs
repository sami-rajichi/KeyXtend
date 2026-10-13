//! The `XAudio2` side of the speaker: the engine, its mastering voice on the default device and the source voices, each
//! undone on drop, on a thread in COM's multithreaded apartment.

use std::ops::RangeInclusive;
use std::sync::{Arc, OnceLock};

use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig};
use windows::Win32::Foundation::{E_INVALIDARG, E_POINTER};
use windows::Win32::Media::Audio::XAudio2::{
    IXAudio2, IXAudio2MasteringVoice, IXAudio2SourceVoice, IXAudio2VoiceCallback, XAUDIO2_BUFFER,
    XAUDIO2_COMMIT_NOW, XAUDIO2_DEFAULT_CHANNELS, XAUDIO2_DEFAULT_FREQ_RATIO,
    XAUDIO2_DEFAULT_SAMPLERATE, XAUDIO2_E_DEVICE_INVALIDATED, XAUDIO2_END_OF_STREAM,
    XAUDIO2_MAX_SAMPLE_RATE, XAUDIO2_MIN_SAMPLE_RATE, XAUDIO2_USE_DEFAULT_PROCESSOR,
    XAUDIO2_VOICE_NOSAMPLESPLAYED, XAUDIO2_VOICE_STATE,
};
use windows::Win32::Media::Audio::{
    AUDCLNT_E_DEVICE_INVALIDATED, AUDIO_STREAM_CATEGORY, AudioCategory_SoundEffects,
    WAVE_FORMAT_PCM, WAVEFORMATEX,
};
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows::core::{Error, HRESULT, PCSTR, PCWSTR, s, w};

use super::{Actor, Fault, Os, Warn};
use crate::QpcClock;
use crate::com::Com;

/// `XAudio2` 2.9's library, loaded from System32 only at the first play, as the SDK's own `xaudio2.h` does.
const LIBRARY: PCWSTR = w!("xaudio2_9.dll");
/// Its engine factory.
const CREATE: PCSTR = s!("XAudio2Create");
/// The kind of stream Windows files our sounds under.
const CATEGORY: AUDIO_STREAM_CATEGORY = AudioCategory_SoundEffects;
/// The errors that mean the device was lost or changed.
const LOST: [HRESULT; 2] = [XAUDIO2_E_DEVICE_INVALIDATED, AUDCLNT_E_DEVICE_INVALIDATED];
/// Bits in a byte, for the sample size in bits.
const BYTE_BITS: u16 = 8;
/// The format tag of plain PCM.
#[allow(clippy::cast_possible_truncation, reason = "The PCM tag is 1.")]
const PCM: u16 = WAVE_FORMAT_PCM as u16;

/// `XAudio2Create` as `xaudio2.h` declares it: the engine out, the flags and the processor.
type Create = unsafe extern "system" fn(*mut Option<IXAudio2>, u32, u32) -> HRESULT;

/// The Windows speaker: `XAudio2` on a thread of its own, made at the first play.
#[derive(Debug)]
pub struct WinSpeaker(Actor);

impl WinSpeaker {
    /// Starts the speaker thread, timed by the platform's `clock`; no audio call runs before the first play.
    #[must_use]
    pub fn new(clock: QpcClock) -> Self {
        Self(Actor::spawn(WinOs::new, Arc::new(clock)))
    }
}

impl Speaker for WinSpeaker {
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig) {
        self.0.prepare(clips, config);
    }

    fn play(&self, id: ClipId, gain: f32) {
        self.0.play(id, gain);
    }
}

/// `XAudio2` on the speaker thread, which keeps COM on until it drops.
struct WinOs {
    _com: Option<Com>,
}

impl WinOs {
    /// Starts COM on this thread; without it the engine fails and warns at the first play.
    fn new() -> Self {
        Self {
            _com: Com::start().ok(),
        }
    }
}

/// The engine; dropping it releases `XAudio2`.
struct Engine(IXAudio2);

/// The mastering voice; dropping it destroys the voice.
struct Master(IXAudio2MasteringVoice);

/// A source voice; dropping it destroys the voice.
struct Voice(IXAudio2SourceVoice);

impl Drop for Master {
    #[allow(unsafe_code, reason = "Destroys a voice we made.")]
    fn drop(&mut self) {
        // SAFETY: the rig drops its engine only after this, and its source voices, which send here, before it.
        unsafe { self.0.DestroyVoice() };
    }
}

impl Drop for Voice {
    #[allow(unsafe_code, reason = "Destroys a voice we made.")]
    fn drop(&mut self) {
        // SAFETY: the rig drops its engine only after this; nothing uses the voice afterwards.
        unsafe { self.0.DestroyVoice() };
    }
}

impl Os for WinOs {
    type Engine = Engine;
    type Master = Master;
    type Voice = Voice;

    const RATES: RangeInclusive<u32> = XAUDIO2_MIN_SAMPLE_RATE..=XAUDIO2_MAX_SAMPLE_RATE;

    #[allow(unsafe_code, reason = "Calls XAudio2's factory.")]
    fn engine(&self) -> Result<Engine, Fault> {
        let create = factory().map_err(Fault::Failed)?;
        let mut engine = None;
        // SAFETY: `create` has the signature `xaudio2.h` declares; `engine` is a local it fills, with no flags.
        unsafe { create(&raw mut engine, 0, XAUDIO2_USE_DEFAULT_PROCESSOR) }
            .ok()
            .map_err(|e| fault(&e))?;
        engine.map(Engine).ok_or(Fault::Failed(E_POINTER.0))
    }

    #[allow(unsafe_code, reason = "Makes the mastering voice.")]
    fn master(&self, engine: &Engine) -> Result<Master, Fault> {
        let mut voice = None;
        let (channels, rate) = (XAUDIO2_DEFAULT_CHANNELS, XAUDIO2_DEFAULT_SAMPLERATE);
        // SAFETY: `voice` is a local the call fills; no device id picks the default device, and there is no effect.
        unsafe {
            engine.0.CreateMasteringVoice(
                &raw mut voice,
                channels,
                rate,
                0,
                PCWSTR::null(),
                None,
                CATEGORY,
            )
        }
        .map_err(|e| fault(&e))?;
        voice.map(Master).ok_or(Fault::Failed(E_POINTER.0))
    }

    #[allow(unsafe_code, reason = "Makes a source voice.")]
    fn voice(&self, engine: &Engine, rate: u32) -> Result<Voice, Fault> {
        let (format, mut voice) = (pcm(rate), None);
        let ratio = XAUDIO2_DEFAULT_FREQ_RATIO;
        let callback = None::<&IXAudio2VoiceCallback>;
        // SAFETY: `format` and `voice` are locals that outlive the call; there is no callback, send list or effect.
        unsafe {
            engine.0.CreateSourceVoice(
                &raw mut voice,
                &raw const format,
                0,
                ratio,
                callback,
                None,
                None,
            )
        }
        .map_err(|e| fault(&e))?;
        voice.map(Voice).ok_or(Fault::Failed(E_POINTER.0))
    }

    #[allow(unsafe_code, reason = "Starts the engine.")]
    fn start(&self, engine: &Engine) -> Result<(), Fault> {
        // SAFETY: the engine lives as long as its handle.
        unsafe { engine.0.StartEngine() }.map_err(|e| fault(&e))
    }

    #[allow(unsafe_code, reason = "Stops the engine.")]
    fn stop(&self, engine: &Engine) {
        // SAFETY: the engine lives as long as its handle.
        unsafe { engine.0.StopEngine() };
    }

    fn play(&self, voice: &Voice, samples: &'static [u8], gain: f32) -> Result<(), Fault> {
        let bytes = u32::try_from(samples.len()).map_err(|_| Fault::Failed(E_INVALIDARG.0))?;
        let buffer = XAUDIO2_BUFFER {
            Flags: XAUDIO2_END_OF_STREAM,
            AudioBytes: bytes,
            pAudioData: samples.as_ptr(),
            ..Default::default()
        };
        submit(&voice.0, &buffer, gain).map_err(|e| fault(&e))
    }

    #[allow(unsafe_code, reason = "Reads a voice's queue.")]
    fn busy(&self, voice: &Voice) -> bool {
        let (voice, mut state) = (&voice.0, XAUDIO2_VOICE_STATE::default());
        // SAFETY: the voice lives as long as its handle, and `state` is a local the call fills, with no sample count.
        unsafe { voice.GetState(&raw mut state, XAUDIO2_VOICE_NOSAMPLESPLAYED) };
        state.BuffersQueued > 0
    }

    fn warn(&self, warn: Warn, code: Option<i32>) {
        let why = match warn {
            Warn::Open => "no sound: the audio engine could not start",
            Warn::Rebuild => "no sound: the audio engine could not be rebuilt",
            Warn::Play => "a sound was dropped: the audio device refused it",
            Warn::Bytes => "a sound was skipped: its data is empty or not whole samples",
            Warn::Rate => "a sound was skipped: the audio engine takes no such sample rate",
        };
        if let Some(code) = code {
            tracing::warn!(code, "{why}");
        } else {
            tracing::warn!("{why}");
        }
    }
}

/// Plays `buffer` at `gain` on `voice`, which has nothing queued; starting a voice that runs already does nothing.
#[allow(unsafe_code, reason = "Plays static samples on a source voice.")]
fn submit(
    voice: &IXAudio2SourceVoice,
    buffer: &XAUDIO2_BUFFER,
    gain: f32,
) -> windows::core::Result<()> {
    // SAFETY: the voice lives as long as its handle; the buffer points at static samples, which outlive any play, and
    // XAudio2 copies the struct itself during the call.
    unsafe {
        voice.SetVolume(gain, XAUDIO2_COMMIT_NOW)?;
        voice.SubmitSourceBuffer(buffer, None)?;
        voice.Start(0, XAUDIO2_COMMIT_NOW)
    }
}

/// `XAudio2`'s factory, loaded once; the system's error code when the library or its export is missing.
fn factory() -> Result<Create, i32> {
    static FACTORY: OnceLock<Result<Create, i32>> = OnceLock::new();
    *FACTORY.get_or_init(load)
}

/// Loads `XAudio2` 2.9 from System32 and finds its factory; the library stays loaded for the process.
#[allow(
    unsafe_code,
    reason = "Loads a system library and takes its export's declared signature."
)]
fn load() -> Result<Create, i32> {
    // SAFETY: a constant name searched in System32 only; the library is never freed, so its code outlives every call.
    let library = unsafe { LoadLibraryExW(LIBRARY, None, LOAD_LIBRARY_SEARCH_SYSTEM32) }
        .map_err(|e| e.code().0)?;
    // SAFETY: a constant export name in the library just loaded.
    let export =
        unsafe { GetProcAddress(library, CREATE) }.ok_or_else(|| Error::from_thread().code().0)?;
    // SAFETY: `xaudio2.h` declares `XAudio2Create` with exactly the `Create` signature.
    Ok(unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, Create>(export) })
}

/// `e` as a fault: a lost or changed device, or another failure, with its code.
fn fault(e: &Error) -> Fault {
    let code = e.code();
    if LOST.contains(&code) {
        Fault::Lost(code.0)
    } else {
        Fault::Failed(code.0)
    }
}

/// The clips' format, mono PCM16, at `rate`.
fn pcm(rate: u32) -> WAVEFORMATEX {
    let block = Clip::CHANNELS * Clip::SAMPLE_BYTES;
    WAVEFORMATEX {
        wFormatTag: PCM,
        nChannels: Clip::CHANNELS,
        nSamplesPerSec: rate,
        nAvgBytesPerSec: rate.saturating_mul(u32::from(block)),
        nBlockAlign: block,
        wBitsPerSample: Clip::SAMPLE_BYTES * BYTE_BITS,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Foundation::E_FAIL;

    #[test]
    fn a_lost_or_changed_device_is_a_loss_and_any_other_error_a_failure() {
        for code in LOST {
            assert_eq!(fault(&code.into()), Fault::Lost(code.0));
        }
        assert_eq!(fault(&E_FAIL.into()), Fault::Failed(E_FAIL.0));
    }

    #[test]
    fn the_voices_take_mono_pcm16_at_the_clip_rate_and_every_rate_the_port_names() {
        let f = pcm(48_000);
        let (tag, channels, rate) = (f.wFormatTag, f.nChannels, f.nSamplesPerSec);
        let (per_second, block, bits) = (f.nAvgBytesPerSec, f.nBlockAlign, f.wBitsPerSample);
        assert_eq!((tag, channels, rate), (1, 1, 48_000));
        assert_eq!((per_second, block, bits), (96_000, 2, 16));
        let (low, high) = (Clip::RATES.start(), Clip::RATES.end());
        assert!(WinOs::RATES.contains(low) && WinOs::RATES.contains(high));
    }
}
