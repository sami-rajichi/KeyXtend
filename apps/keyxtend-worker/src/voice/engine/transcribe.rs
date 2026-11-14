//! The transcribe.cpp adapter (ADR-0043, ADR-0044). Its parts load once, from the worker's own folder only, before any
//! device list or model; its log is off, as a line may name a path. Each model runs with the language code the ask
//! gives, timestamps off, and a cancel token its run's `Stop` sets.

use crate::voice::library::{Failed, Found, Library, Loaded, Place, Stop};
use kx_ipc::{DeviceKind, SOUND_HZ};
use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use transcribe_cpp::{
    Backend, CancelToken, Device, DeviceType, Error, Model, ModelOptions, RunOptions, Session,
    SessionOptions, TimestampKind, Transcript,
};
use zeroize::{Zeroize, Zeroizing};

/// Milliseconds in a second.
const MS_PER_S: i64 = 1000;
/// The engine's variable prefixes: such a variable can load an outside library, write to disk or narrow the devices.
const ENGINE_VARS: [&str; 2] = ["GGML_", "TRANSCRIBE_"];

/// Whether the parts loaded, decided once for the process.
static STARTED: OnceLock<bool> = OnceLock::new();

/// Loads the engine's parts once from the folder that holds this program, never from a path a variable or an ask
/// names; false when no processor part loaded, or when a variable could steer the engine, as the host removes them.
pub fn start() -> bool {
    *STARTED.get_or_init(|| {
        transcribe_cpp::disable_logging();
        let clean = clean(env::vars_os().map(|(name, _)| name));
        clean && own_folder().is_some_and(|dir| transcribe_cpp::init_backends(dir).is_ok())
    })
}

/// True when none of `names` is a variable the engine reads; names are matched in any case, as Windows does.
fn clean(names: impl IntoIterator<Item = OsString>) -> bool {
    names.into_iter().all(|name| {
        let name = name.to_string_lossy().to_ascii_uppercase();
        !ENGINE_VARS.iter().any(|prefix| name.starts_with(prefix))
    })
}

/// The folder that holds this program.
fn own_folder() -> Option<PathBuf> {
    env::current_exe().ok()?.parent().map(Path::to_path_buf)
}

/// The library: its state is the process's, so it carries none.
pub struct Transcribe;

/// A model with its one session and the token that aborts its run.
pub struct Spoken {
    session: Session,
    token: CancelToken,
    langs: Vec<String>,
    most: Option<usize>,
}

impl Library for Transcribe {
    const RUNNER: &'static str = "transcribe";
    type Model = Spoken;

    fn start(&self) -> bool {
        start()
    }

    fn devices(&self) -> Vec<Found> {
        transcribe_cpp::devices().iter().filter_map(found).collect()
    }

    fn langs(&self, path: &Path) -> Result<Vec<String>, Failed> {
        let model = Model::load_with(path, &on_cpu()).map_err(|e| load_failed(&e))?;
        let caps = model.capabilities().map_err(|_| Failed::BadFile)?;
        Ok(caps.languages)
    }

    fn load(&self, path: &Path, place: &Place, threads: usize) -> Result<Spoken, Failed> {
        let options = match place {
            Place::Cpu => on_cpu(),
            Place::Card(id) => on_card(id)?,
        };
        let model = Model::load_with(path, &options).map_err(|e| load_failed(&e))?;
        let langs = model.capabilities().map_err(|_| Failed::BadFile)?.languages;
        let n_threads = i32::try_from(threads).unwrap_or(i32::MAX);
        let options = SessionOptions {
            n_threads,
            ..SessionOptions::default()
        };
        let mut session = model.session_with(&options).map_err(|e| load_failed(&e))?;
        let token = CancelToken::new();
        session.set_cancel_token(&token);
        let most = most(&session, &model);
        Ok(Spoken {
            session,
            token,
            langs,
            most,
        })
    }
}

impl Loaded for Spoken {
    fn langs(&self) -> &[String] {
        &self.langs
    }

    fn most(&self) -> Option<usize> {
        self.most
    }

    fn run(&mut self, pcm: &[f32], lang: &str, stop: &Stop) -> Result<Zeroizing<String>, Failed> {
        self.token.reset();
        let token = self.token.clone();
        stop.arm(Box::new(move || token.cancel()));
        let options = RunOptions {
            language: Some(lang.to_owned()),
            timestamps: TimestampKind::None,
            ..RunOptions::default()
        };
        let got = self.session.run(pcm, &options);
        stop.disarm();
        match got {
            Ok(mut said) => Ok(kept(&mut said)),
            Err(why) => ran_failed(why),
        }
    }
}

/// Takes the text of `said` and wipes its other copies of the words.
fn kept(said: &mut Transcript) -> Zeroizing<String> {
    let text = Zeroizing::new(std::mem::take(&mut said.text));
    said.raw_text.zeroize();
    said.segments.iter_mut().for_each(|s| s.text.zeroize());
    said.words.iter_mut().for_each(|w| w.text.zeroize());
    said.tokens.iter_mut().for_each(|t| t.text.zeroize());
    text
}

/// A model on the processor alone.
fn on_cpu() -> ModelOptions {
    ModelOptions {
        backend: Backend::Cpu,
        ..ModelOptions::default()
    }
}

/// A model on the card the library names `id`; `Device` when it is gone.
fn on_card(id: &str) -> Result<ModelOptions, Failed> {
    let card = transcribe_cpp::devices().into_iter().find(|d| d.name == id);
    Ok(ModelOptions {
        device: Some(card.ok_or(Failed::Device)?),
        ..ModelOptions::default()
    })
}

/// A device as the engine's rules see it; host accelerators and unknown kinds are left out.
fn found(d: &Device) -> Option<Found> {
    let kind = match d.device_type {
        DeviceType::Cpu => DeviceKind::Cpu,
        DeviceType::Gpu => DeviceKind::Card,
        DeviceType::Igpu => DeviceKind::BuiltIn,
        DeviceType::Accel | DeviceType::Unknown => return None,
    };
    Some(Found {
        kind,
        id: d.name.clone(),
        name: d.description.clone(),
        total: d.memory_total,
        free: d.memory_free,
    })
}

/// The longest sound one run of `session` takes, in samples; `None` when the model sets no limit.
fn most(session: &Session, model: &Model) -> Option<usize> {
    let caps_ms = model.capabilities().map_or(0, |c| c.max_audio_ms);
    let ms = session
        .limits()
        .map_or(caps_ms, |l| l.effective_max_audio_ms);
    let samples = ms.checked_mul(i64::try_from(SOUND_HZ).ok()?)? / MS_PER_S;
    usize::try_from(samples).ok().filter(|&n| n > 0)
}

/// Why a load failed, without the library's words.
fn load_failed(why: &Error) -> Failed {
    match why {
        Error::ModelFileNotFound(_) | Error::Nul(_) => Failed::NoFile,
        Error::ModelLoad(_) | Error::InvalidArgument(_) | Error::UnsupportedRole(_) => {
            Failed::BadFile
        }
        Error::OutOfMemory(_) => Failed::Memory,
        Error::Backend(_) => Failed::Device,
        _ => Failed::Other,
    }
}

/// A run's end when it failed: a cut-short text is kept, sound too short is no text, and the rest fail.
fn ran_failed(why: Error) -> Result<Zeroizing<String>, Failed> {
    match why {
        Error::OutputTruncated { partial, .. } | Error::OutputRepetition { partial, .. } => {
            Ok(partial.map_or_else(Zeroizing::default, |mut t| kept(&mut t)))
        }
        Error::InputTooShort(_) => Ok(Zeroizing::new(String::new())),
        Error::Aborted { partial, .. } => {
            drop(partial.map(|mut t| kept(&mut t)));
            Err(Failed::Stopped)
        }
        Error::Unsupported(_) => Err(Failed::Language),
        Error::OutOfMemory(_) => Err(Failed::Memory),
        Error::Backend(_) => Err(Failed::Device),
        _ => Err(Failed::Other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transcribe_cpp::{Segment, Token, Word};

    fn names(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn any_engine_variable_in_any_case_stops_the_start() {
        assert!(clean(names(&["PATH", "SystemRoot", "VULKAN_SDK", "GGML"])));
        for set in [
            "GGML_BACKEND_PATH",
            "TRANSCRIBE_DUMP_DIR",
            "ggml_disable_vulkan",
        ] {
            assert!(!clean(names(&["PATH", set])), "{set}");
        }
    }

    #[test]
    fn a_kept_text_leaves_no_other_copy_of_the_words() {
        let mut said = Transcript {
            text: "words".into(),
            raw_text: "words".into(),
            segments: vec![Segment {
                text: "words".into(),
                ..Segment::default()
            }],
            words: vec![Word {
                text: "words".into(),
                ..Word::default()
            }],
            tokens: vec![Token {
                text: "words".into(),
                ..Token::default()
            }],
            ..Transcript::default()
        };
        assert_eq!(*kept(&mut said), "words");
        let rest = [&said.raw_text, &said.segments[0].text, &said.words[0].text];
        assert!(rest.iter().all(|t| t.is_empty()) && said.tokens[0].text.is_empty());
        let cut = Error::OutputTruncated {
            message: String::new(),
            partial: None,
        };
        assert_eq!(ran_failed(cut).map(|t| t.is_empty()), Ok(true));
    }
}
