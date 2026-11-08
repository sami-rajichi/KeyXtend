//! The app on Windows starts in order: pinned folders, command line, hook, the data folder's probe, one copy, hub and
//! kernel, log, modules, the hidden copies' questions, kernel thread, then hardened Qt on the lowest-power graphics
//! adapter. It stops as Quit stops the modules and ends the UI, then `drain::stop` hides the tray icon and joins the
//! kernel's thread, and `drain::release` frees the session before it waits for the deletes still running.

use keyxtend::args::{self, Input};
use keyxtend::boot::{Boot, BootError, Services};
use keyxtend::copy::{self, Reach, Start};
use keyxtend::hidden::{Asked, Deletes, Probed, Run};
use keyxtend::relay::Relay;
use keyxtend::{UI_ACCESS, clip, drain, log, quiet, vault, words};
use kx_crypto::OsRandom;
use kx_kernel::KernelSettings;
use kx_module_api::ports::PointerHook;
use kx_platform::Platform;
use kx_platform::ui::UiError;
use kx_platform_windows::{WindowsPlatform, dll, gpu, release_now, uiaccess};
use kx_settings::Files;
use kx_ui::{Hub, UiPorts};
use std::env;
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

/// The app's version, for the first log line.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the app to its end and says how it ended.
pub fn main() -> ExitCode {
    if let Err(e) = UI_ACCESS.then(dll::pin).transpose() {
        return fail(&e);
    }
    let (command, input) = match args::parse(env::args_os().skip(1)) {
        Ok(parsed) => parsed,
        Err(e) => return fail(&e),
    };
    quiet::install(release_now);
    let platform = match WindowsPlatform::new() {
        Ok(platform) => Arc::new(platform),
        Err(e) => return fail(&e),
    };
    let data = platform.dirs().data_dir();
    let probed = Run::probe(&*platform.hidden(), data.path());
    let files = Files::new(data.path());
    let reach = match Reach::from_file(probed.run.peek(&files).as_ref()) {
        Ok(reach) => reach,
        Err(e) => return fail(&e),
    };
    let mut claim = match copy::start(&*platform.instance(), command, reach, thread::sleep) {
        Start::Run(claim) => claim,
        Start::Done => return ExitCode::SUCCESS,
        Start::Unanswered(e) => return fail(&format!("another copy runs but did not answer: {e}")),
    };
    let relay = Arc::new(Relay::default());
    let heard = Arc::clone(&relay);
    let listened = claim.listen(Box::new(move |c| heard.hear(c)));
    let (code, deletes) = run(&platform, &files, &relay, listened, input, &probed);
    drain::release(claim, &deletes);
    tracing::info!("KeyXtend ended");
    code
}

/// Boots the kernel and the hub over `platform` for the `probed` run, the mouse engine on `input`, logging into
/// `files` unless hidden, then runs Qt until the UI ends; it gives the deletes still running.
fn run(
    platform: &Arc<WindowsPlatform>,
    files: &Files,
    relay: &Relay,
    listened: Result<(), UiError>,
    input: Input,
    probed: &Probed,
) -> (ExitCode, Deletes) {
    let mut boot = match boot(platform, input, &probed.run) {
        Ok(boot) => boot,
        Err(e) => {
            start_log(probed, None, files);
            return (fail(&e), Deletes::default());
        }
    };
    let _tray = TrayGuard(Arc::clone(&boot.hub));
    tell_booted(&boot, probed, files, listened);
    #[cfg(feature = "gate-input")]
    warn_gate(input);
    relay.attach(&boot.hub);
    kx_ui::install(Arc::clone(&boot.hub));
    if let Err(e) = boot.start() {
        tracing::error!(error = e.kind(), "the modules did not start");
    }
    let asked = probed.run.ask(platform.hidden(), &boot.kernel, &boot.hub);
    let deletes = asked.as_ref().map(Asked::deletes).unwrap_or_default();
    let (kernel, hub) = (boot.kernel, boot.hub);
    let kernel_thread = match drain::spawn(kernel, Arc::clone(&hub), boot.requests, asked) {
        Ok(handle) => handle,
        Err(e) => return (fail(&e), deletes),
    };
    let pinned = UI_ACCESS.then(|| platform.dirs().exe_dir.as_path());
    let code = crate::qt::run(pinned, gpu::low_power_adapter, &hub.settings(), || {
        let _ = drain::stop(&hub, kernel_thread);
    });
    (code, deletes)
}

/// Opens the `probed` run's log from `boot`'s settings, then logs a clipboard folder that could not be made, the
/// version, and a second copy that cannot reach this one, as `listened` says.
fn tell_booted(boot: &Boot, probed: &Probed, files: &Files, listened: Result<(), UiError>) {
    start_log(probed, boot.kernel.kernel_settings(), files);
    if let Some(kind) = boot.no_clip_dir {
        tracing::warn!(error = %kind, "the clipboard folder could not be made, so its history may not save");
    }
    let ui_access = uiaccess::active();
    tracing::info!(version = VERSION, ui_access, "KeyXtend started");
    if let Err(e) = listened {
        tracing::warn!(error = %e, "a second copy cannot reach this one");
    }
}

/// The hub and the kernel over `platform` for `run`, with every module added and the settings loaded; the learned
/// words, the clipboard history and Quick-fill's notebook each keep their own sealed folder and key, or memory alone in
/// a hidden run.
fn boot(platform: &Arc<WindowsPlatform>, input: Input, run: &Run) -> Result<Boot, BootError> {
    let dirs = platform.dirs();
    let sealed = dirs.data_dir().path().join(words::SEALED_DIR);
    let guard = || platform.key_guard();
    let shelf = run.shelf(clip::folder(dirs), guard, platform.disk_room());
    let vault = run.store(vault::folder(dirs), guard);
    let services = Services {
        injector: platform.injector(),
        layouts: platform.layouts(),
        pointer: pointer(platform, input),
        scroll: platform.scroll_target(),
        shortcuts: platform.shortcuts(),
        power: platform.power_dialog(),
        field: platform.field_watch(),
        packs: platform.word_packs(words::pack_dirs(&dirs.exe_dir)),
        sealed: run.store(sealed, guard),
        clipboard: platform.clipboard(),
        writer: platform.clipboard_writer(),
        shelf,
        pictures: platform.pictures(),
        calendar: platform.calendar(),
        vault,
        random: Arc::new(OsRandom),
        boxes: platform.boxes(),
        places: platform.places(),
        selection: platform.selection_watch(),
        link: platform.web_link(),
    };
    Boot::load(platform.clone(), services, ports(platform), run)
}

/// The mouse hook on `input`: the hand's, or in the gate-only build the gate tool's.
fn pointer(platform: &WindowsPlatform, input: Input) -> Arc<dyn PointerHook> {
    match input {
        Input::Hand => platform.pointer_hook(),
        #[cfg(feature = "gate-input")]
        Input::Sim => platform.sim_pointer_hook(),
    }
}

/// Warns once that the engine takes the gate tool's input; only the gate-only build can. Its `flag` field keeps the
/// flag's text whole in a release exe, which is what the stage check looks for.
#[cfg(feature = "gate-input")]
fn warn_gate(input: Input) {
    if input == Input::Sim {
        let flag = args::SIM_INPUT;
        tracing::warn!(
            flag,
            "simulated mouse input: a gate build, never for daily use"
        );
    }
}

/// Every UI port from the Windows adapter, the tray icon too.
fn ports(platform: &WindowsPlatform) -> UiPorts {
    UiPorts {
        guard: platform.guard(),
        screens: platform.screens(),
        look: platform.look(),
        backdrop: platform.backdrop(),
        tray: platform.tray(),
        memory: platform.memory(),
        speaker: platform.speaker(),
        reveal: platform.reveal(),
    }
}

/// Opens the `probed` run's log, in `files` unless hidden, from the kernel's `loaded` settings or its defaults, then
/// logs how the probe went.
fn start_log(probed: &Probed, loaded: Option<KernelSettings>, files: &Files) {
    match probed.run.log(loaded, files) {
        Some(opened) => {
            let _ = log::install(opened);
        }
        None => eprintln!("no log this run: the kernel's settings did not read"),
    }
    probed.tell();
}

/// Removes the tray icon when this thread unwinds or returns early; the hub itself never drops.
struct TrayGuard(Arc<Hub>);

impl Drop for TrayGuard {
    fn drop(&mut self) {
        self.0.tray_hide();
    }
}

/// Says why the app stops, on standard error and in the log once it is open, and gives the failure code.
fn fail(why: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("KeyXtend stopped: {why}");
    tracing::error!(error = %why, "KeyXtend stopped");
    ExitCode::FAILURE
}
