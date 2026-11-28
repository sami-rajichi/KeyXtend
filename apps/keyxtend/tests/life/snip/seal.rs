//! The recorder the app registers opens only its own seal's passes (ADR-0056): a probe under the snip module's id takes
//! it with the microphone's passes, whose own pass goes through, and another seal's pass is refused before the
//! platform hears it, at the start and at a switch, even by a platform recorder that would open it.

use crate::rig::Rig;
use crate::rig::snip::SCREEN;
use keyxtend::hidden::Run;
use kx_kernel::{Kernel, Policy};
use kx_mod_snip::SnipSettings;
use kx_module_api::ports::{
    MicPassKey, MicPasses, MicSeal, Part, RecordAsk, RecordError, Recorder, RecorderKey,
};
use kx_module_api::{Capability, Manifest, Module, ModuleCx, ModuleError, ServiceKey};
use kx_platform_fake::FakeRecorder;
use std::sync::{Arc, Mutex};

/// A probe under the snip module's id that takes the recorder and the microphone's passes.
static CATCH: Manifest = Manifest {
    id: kx_mod_snip::ID,
    version: "0.0.0",
    requires: &[RecorderKey::ID, MicPassKey::ID],
    provides: &[],
    capabilities: &[Capability::ScreenCapture, Capability::Microphone],
    settings: None,
};

/// The ports `Catch` took.
type Caught = Arc<Mutex<Option<(Arc<dyn Recorder>, Arc<dyn MicPasses>)>>>;

/// Takes the recorder and the passes at its start, as the snip module does.
struct Catch(Caught);

impl Module for Catch {
    fn manifest(&self) -> &'static Manifest {
        &CATCH
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let taken = (cx.service::<RecorderKey>()?, cx.service::<MicPassKey>()?);
        *self.0.lock().unwrap() = Some(taken);
        Ok(())
    }
}

/// A recording of the whole screen at the least values, hearing the microphone with `mic`'s pass.
fn ask(mic: Option<&dyn MicPasses>) -> RecordAsk {
    RecordAsk {
        part: Part::new(SCREEN).unwrap(),
        fps: RecordAsk::FPS[0],
        mic: mic.map(MicPasses::pass),
        sound: false,
        max_min: *RecordAsk::MAX_MIN.start(),
        min_free_mb: *RecordAsk::MIN_FREE_MB.start(),
        job_mb: *RecordAsk::JOB_MB.start(),
        waits: SnipSettings::from_defaults().unwrap().record_waits(),
    }
}

/// The app's kernel over `rig`'s ports with `platform` as the platform's recorder, and what the probe took.
fn caught(rig: &Rig, platform: &FakeRecorder) -> (Kernel, Arc<dyn Recorder>, Arc<dyn MicPasses>) {
    let policy = Policy::new().allow(CATCH.id, CATCH.capabilities);
    let mut kernel = Kernel::new(rig.platform(), policy);
    let mut ports = rig.snip.ports(&Run::Normal);
    ports.recorder = Arc::new(platform.clone());
    ports.provide(&mut kernel).unwrap();
    let caught = Caught::default();
    kernel.add(Box::new(Catch(Arc::clone(&caught)))).unwrap();
    kernel.load().unwrap();
    kernel.start().unwrap();
    let (recorder, passes) = caught.lock().unwrap().take().unwrap();
    (kernel, recorder, passes)
}

#[test]
fn the_recorder_the_app_registers_opens_only_its_own_seals_passes() {
    // The platform's recorder opens `other`'s passes, so each refusal below is the app's own check.
    let (rig, other) = (Rig::new(), MicSeal::new());
    let platform = FakeRecorder::new(other.clone(), &rig.snip.normal.folder);
    let (_kernel, recorder, passes) = caught(&rig, &platform);
    assert!(
        rig.snip.seal.opens(&passes.pass()),
        "the app's seal's passes"
    );
    let refused = recorder.start(ask(Some(&other)), Box::new(|_| {}));
    assert_eq!(refused.err(), Some(RecordError::BadPass));
    assert!(platform.starts().is_empty(), "the platform never heard it");
    let mut run = recorder.start(ask(None), Box::new(|_| {})).unwrap();
    assert_eq!(platform.starts().len(), 1, "a start with no microphone");
    assert_eq!(run.mic(Some(other.pass())), Err(RecordError::BadPass));
    let heard = platform.runs()[0].heard.len();
    assert_eq!(heard, 1, "nor did that switch reach it");
    run.cancel();
}

#[test]
fn a_pass_of_the_passes_the_app_registers_opens_the_recorder_it_registers() {
    // The platform's recorder holds the app's seal too, so only the registered check can refuse.
    let rig = Rig::new();
    let platform = FakeRecorder::new(rig.snip.seal.clone(), &rig.snip.normal.folder);
    let (_kernel, recorder, passes) = caught(&rig, &platform);
    let started = recorder.start(ask(Some(&*passes)), Box::new(|_| {}));
    let mut run = started.unwrap();
    assert!(
        platform.starts()[0].mic.is_some(),
        "heard with the microphone"
    );
    assert_eq!(run.mic(Some(passes.pass())), Ok(()));
    run.cancel();
}
