//! The app's order of start and stop on Windows. Start: pinned library folders in release, the command line and the
//! quiet hook, the one running copy (listening at once), the hub and the kernel, the log, the modules, the kernel's
//! thread, then Qt, hardened first. Stop: Quit reaches the kernel's thread, which stops the modules and ends the UI;
//! once Qt's loop returns, `drain::stop` removes the tray icon and joins the kernel's thread.

use keyxtend::boot::{Boot, BootError, Services};
use keyxtend::copy::{self, Reach, Start};
use keyxtend::relay::Relay;
use keyxtend::{args, drain, log, quiet};
use kx_kernel::KernelSettings;
use kx_platform::Platform;
use kx_platform::ui::UiError;
use kx_platform_windows::{WindowsPlatform, dll, uiaccess};
use kx_settings::{Files, peek};
use kx_ui::{Hub, UiPorts};
use std::env;
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

/// The app's version, for the first log line.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Release builds load libraries only from the app's folder and System32, and Qt's plugins and QML modules only from
/// the app's folder; debug builds find the installed Qt through `PATH`.
const PINNED: bool = !cfg!(debug_assertions);

/// Runs the app to its end and says how it ended.
pub fn main() -> ExitCode {
    if let Err(e) = PINNED.then(dll::pin).transpose() {
        return fail(&e);
    }
    let command = match args::parse(env::args_os().skip(1)) {
        Ok(command) => command,
        Err(e) => return fail(&e),
    };
    quiet::install();
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
    let code = run(&platform, &files, &relay, listened);
    drop(claim);
    tracing::info!("KeyXtend ended");
    code
}

/// Boots the kernel and the hub over `platform`, logging into `files`, then runs Qt until the UI ends.
fn run(
    platform: &Arc<WindowsPlatform>,
    files: &Files,
    relay: &Relay,
    listened: Result<(), UiError>,
) -> ExitCode {
    let mut boot = match boot(platform) {
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
    if let Err(e) = listened {
        tracing::warn!(error = %e, "a second copy cannot reach this one");
    }
    relay.attach(&boot.hub);
    kx_ui::install(Arc::clone(&boot.hub));
    if let Err(e) = boot.start() {
        tracing::error!(error = %e, "the modules did not start");
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
    let pinned = PINNED.then(|| platform.dirs().exe_dir.as_path());
    crate::qt::run(pinned, || {
        let _ = drain::stop(&hub, kernel_thread);
    })
}

/// The hub and the kernel over `platform`, with every module added and the settings loaded.
fn boot(platform: &Arc<WindowsPlatform>) -> Result<Boot, BootError> {
    let services = Services {
        injector: platform.injector(),
        layouts: platform.layouts(),
    };
    Boot::load(platform.clone(), services, ports(platform))
}

/// Every UI port from the Windows adapter, the tray icon too.
fn ports(platform: &WindowsPlatform) -> UiPorts {
    UiPorts {
        guard: platform.guard(),
        screens: platform.screens(),
        look: platform.look(),
        backdrop: platform.backdrop(),
        tray: platform.tray(),
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
