//! Qt on the main thread: hardened first, kept on the lowest-power graphics adapter and drawing every window on this
//! thread, then the app object, the shared graphics device with the root QML over the installed hub, and the event
//! loop, run again each time a lost device's windows are made again.

use cxx_qt_lib::QGuiApplication;
use kx_ui::{After, Gpu, Shell, harden_qt, pin_adapter, pin_drawing};
use std::path::Path;
use std::process::ExitCode;

/// Loads the root QML and runs Qt's loop until the UI ends; `end` runs after the loop, while Qt's objects still live.
/// With `pinned`, Qt loads plugins and QML modules only from that folder; it draws on the adapter `gpu` picks.
pub fn run(pinned: Option<&Path>, gpu: Gpu, end: impl FnOnce()) -> ExitCode {
    let cleared = harden_qt(pinned);
    if cleared > 0 {
        tracing::info!(cleared, "Qt variables were cleared before Qt started");
    }
    pin((gpu.pick)());
    if !pin_drawing() {
        tracing::warn!("Qt's drawing variables could not be set, so each window draws as Qt picks");
    }
    let mut app = QGuiApplication::new();
    let mut shell = Shell::open(pinned, gpu);
    let exec = || app.as_mut().map(QGuiApplication::exec);
    let code = shell.as_mut().and_then(|s| looped(exec, s));
    end();
    drop(shell);
    drop(app);
    if code == Some(0) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// What runs after each end of Qt's loop; the app's is the shell.
trait Rebuild {
    /// Makes a lost device's windows again, or says the app ends.
    fn rebuilt(&mut self) -> After;
}

impl Rebuild for Shell {
    fn rebuilt(&mut self) -> After {
        Shell::rebuilt(self)
    }
}

/// Runs Qt's loop with `exec` until it ends with no lost device to rebuild; its exit code, or `None` with no app object
/// or when a lost device's windows could not be made again.
fn looped(mut exec: impl FnMut() -> Option<i32>, shell: &mut impl Rebuild) -> Option<i32> {
    loop {
        let code = exec()?;
        match shell.rebuilt() {
            After::Again => {}
            After::End => return Some(code),
            After::Failed => return None,
        }
    }
}

/// Has Qt draw on `adapter`, its place in DXGI's adapter list; with none, Qt picks its own.
fn pin(adapter: Option<u32>) {
    let pinned = adapter.map(|index| (index, pin_adapter(index)));
    match pinned {
        Some((index, true)) => {
            tracing::info!(
                adapter = index,
                "Qt draws on the lowest-power graphics adapter"
            );
        }
        Some((index, false)) => {
            tracing::warn!(adapter = index, "the graphics adapter could not be pinned");
        }
        None => tracing::info!("no graphics adapter was found to pin, so Qt picks one"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shell whose ends of the loop answer `afters` in turn, then `End`.
    struct Fake(Vec<After>);

    impl Rebuild for Fake {
        fn rebuilt(&mut self) -> After {
            if self.0.is_empty() {
                After::End
            } else {
                self.0.remove(0)
            }
        }
    }

    /// A loop that ends with `code` and counts its runs in `runs`.
    fn exec(code: i32, runs: &mut u32) -> impl FnMut() -> Option<i32> + '_ {
        move || {
            *runs += 1;
            Some(code)
        }
    }

    #[test]
    fn the_loop_runs_again_after_each_rebuild_then_gives_its_code() {
        let mut runs = 0;
        let code = looped(
            exec(0, &mut runs),
            &mut Fake(vec![After::Again, After::Again]),
        );
        assert_eq!((code, runs), (Some(0), 3));
    }

    #[test]
    fn windows_that_cannot_be_made_again_end_the_app_with_a_failure() {
        let mut runs = 0;
        let code = looped(
            exec(0, &mut runs),
            &mut Fake(vec![After::Again, After::Failed]),
        );
        assert_eq!((code, runs), (None, 2));
    }

    #[test]
    fn no_app_object_runs_no_loop() {
        let code = looped(|| None, &mut Fake(vec![After::Again]));
        assert_eq!(code, None);
    }
}
