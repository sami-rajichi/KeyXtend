//! The app's order of start and stop on Windows. Start: the command line and the quiet hook, the one running copy
//! (listening at once), the hub and the kernel, the log, the modules, the kernel's thread, then Qt. Stop: Quit reaches
//! the kernel's thread, which stops the modules and ends the UI; once Qt's loop returns, the tray icon goes and the
//! kernel's thread is joined.

use keyxtend::boot::{Boot, Services};
use keyxtend::copy::{self, Reach, Start};
use keyxtend::relay::Relay;
use keyxtend::{args, drain, log, quiet};
use kx_platform::Platform;
use kx_platform::ui::UiError;
use kx_platform_windows::{WindowsPlatform, uiaccess};
use kx_settings::Files;
use kx_ui::{Hub, UiPorts, UiSettings};
use std::env;
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

/// The app's version, for the first log line.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the app to its end and says how it ended.
pub fn main() -> ExitCode {
    let command = match args::parse(env::args_os().skip(1)) {
        Ok(command) => command,
        Err(e) => return fail(&e),
    };
    quiet::install();
    let platform = match WindowsPlatform::new() {
        Ok(platform) => Arc::new(platform),
        Err(e) => return fail(&e),
    };
    let reach = match UiSettings::from_defaults() {
        Ok(settings) => Reach::of(&settings),
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
    let code = run(&platform, &relay, listened);
    drop(claim);
    tracing::info!("KeyXtend ended");
    code
}

/// Boots the kernel and the hub over `platform`, then runs Qt until the UI ends.
fn run(platform: &Arc<WindowsPlatform>, relay: &Relay, listened: Result<(), UiError>) -> ExitCode {
    let services = Services {
        injector: platform.injector(),
        layouts: platform.layouts(),
    };
    let mut boot = match Boot::load(platform.clone(), services, ports(platform)) {
        Ok(boot) => boot,
        Err(e) => return fail(&e),
    };
    let _tray = TrayGuard(Arc::clone(&boot.hub));
    start_log(&boot, platform);
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
        heard: _heard,
    } = boot;
    let kernel_thread = match drain::spawn(kernel, Arc::clone(&hub), requests) {
        Ok(handle) => handle,
        Err(e) => {
            tracing::error!(error = %e, "the kernel's thread did not start");
            return ExitCode::FAILURE;
        }
    };
    crate::qt::run(|| {
        hub.quit();
        hub.tray_hide();
        if kernel_thread.join().is_err() {
            tracing::error!("the kernel's thread ended in a panic");
        }
    })
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

/// Opens the log from the kernel's loaded settings, before any module starts, and notes how the app runs.
fn start_log(boot: &Boot, platform: &WindowsPlatform) {
    let files = Files::new(platform.dirs().data_dir().path());
    let settings = boot.kernel.kernel_settings();
    match settings.as_ref().and_then(|s| log::open(s, &files)) {
        Some(opened) => {
            let _ = log::install(opened);
        }
        None => eprintln!("no log this run: the kernel's settings did not load"),
    }
    tracing::info!(
        version = VERSION,
        ui_access = uiaccess::active(),
        "KeyXtend started"
    );
}

/// Removes the tray icon when it drops, so a panic on this thread leaves no dead icon; the hub itself never drops.
struct TrayGuard(Arc<Hub>);

impl Drop for TrayGuard {
    fn drop(&mut self) {
        self.0.tray_hide();
    }
}

/// Says why the app stops on standard error, as no log may be open yet, and gives the failure code.
fn fail(why: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("KeyXtend stopped: {why}");
    tracing::error!(error = %why, "KeyXtend stopped");
    ExitCode::FAILURE
}
