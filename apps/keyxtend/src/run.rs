//! The app on Windows starts in order: pinned folders, command line, hook, one copy, hub and kernel, log, modules,
//! kernel thread, then hardened Qt on the lowest-power graphics adapter. It stops as Quit stops the modules and ends
//! the UI, then `drain::stop` hides the tray icon and joins the kernel's thread.

use keyxtend::args::{self, Input};
use keyxtend::boot::{Boot, BootError, Services};
use keyxtend::copy::{self, Reach, Start};
use keyxtend::relay::Relay;
use keyxtend::{UI_ACCESS, drain, log, quiet, words};
use kx_crypto::SealedFiles;
use kx_kernel::KernelSettings;
use kx_module_api::ports::PointerHook;
use kx_platform::Platform;
use kx_platform::ui::UiError;
use kx_platform_windows::{WindowsPlatform, dll, gpu, release_now, uiaccess};
use kx_settings::{Files, peek};
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
    let files = Files::new(platform.dirs().data_dir().path());
    let reach = match Reach::from_file(peek(&files).as_ref()) {
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
    let code = run(&platform, &files, &relay, listened, input);
    drop(claim);
    tracing::info!("KeyXtend ended");
    code
}

/// Boots the kernel and the hub over `platform`, the mouse engine on `input`, logging into `files`, then runs Qt until
/// the UI ends.
fn run(
    platform: &Arc<WindowsPlatform>,
    files: &Files,
    relay: &Relay,
    listened: Result<(), UiError>,
    input: Input,
) -> ExitCode {
    let mut boot = match boot(platform, input) {
        Ok(boot) => boot,
        Err(e) => {
            start_log(None, files);
            return fail(&e);
        }
    };
    let _tray = TrayGuard(Arc::clone(&boot.hub));
    start_log(boot.kernel.kernel_settings(), files);
    let ui_access = uiaccess::active();
    tracing::info!(version = VERSION, ui_access, "KeyXtend started");
    #[cfg(feature = "gate-input")]
    warn_gate(input);
    if let Err(e) = listened {
        tracing::warn!(error = %e, "a second copy cannot reach this one");
    }
    relay.attach(&boot.hub);
    kx_ui::install(Arc::clone(&boot.hub));
    if let Err(e) = boot.start() {
        tracing::error!(error = e.kind(), "the modules did not start");
    }
    let Boot {
        kernel,
        hub,
        requests,
        ..
    } = boot;
    let kernel_thread = match drain::spawn(kernel, Arc::clone(&hub), requests) {
        Ok(handle) => handle,
        Err(e) => return fail(&e),
    };
    let pinned = UI_ACCESS.then(|| platform.dirs().exe_dir.as_path());
    crate::qt::run(pinned, gpu::low_power_adapter, &hub.settings(), || {
        let _ = drain::stop(&hub, kernel_thread);
    })
}

/// The hub and the kernel over `platform`, with every module added and the settings loaded.
fn boot(platform: &Arc<WindowsPlatform>, input: Input) -> Result<Boot, BootError> {
    let dirs = platform.dirs();
    let sealed = dirs.data_dir().path().join(words::SEALED_DIR);
    let services = Services {
        injector: platform.injector(),
        layouts: platform.layouts(),
        pointer: pointer(platform, input),
        scroll: platform.scroll_target(),
        shortcuts: platform.shortcuts(),
        power: platform.power_dialog(),
        field: platform.field_watch(),
        packs: platform.word_packs(words::pack_dirs(&dirs.exe_dir)),
        sealed: Arc::new(SealedFiles::new(sealed, platform.key_guard())),
    };
    Boot::load(platform.clone(), services, ports(platform))
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
    }
}

/// Opens the log in `files` from the kernel's `loaded` settings, or from its defaults when none loaded.
fn start_log(loaded: Option<KernelSettings>, files: &Files) {
    match log::or_defaults(loaded).and_then(|s| log::open(&s, files)) {
        Some(opened) => {
            let _ = log::install(opened);
        }
        None => eprintln!("no log this run: the kernel's settings did not read"),
    }
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
