//! One running copy per user session: a session-local named mutex, and a command window only the running copy owns.
//! The command table and the object names are pure and tested everywhere; the OS calls are in `win`.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "The names and the claim rule serve only the Windows calls in win."
    )
)]

use kx_platform::ui::Command;

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinInstance;

/// Each command, its name and its code in the command message; both copies read this one table.
const COMMANDS: [(Command, &str, usize); 2] =
    [(Command::Show, "show", 1), (Command::Quit, "quit", 2)];

/// The session namespace: a named object in it is seen only inside this user session.
const SESSION: &str = r"Local\";
/// Ends the name of the mutex the running copy holds.
const MUTEX_TAIL: &str = ".Running";
/// Ends the class name of the running copy's command window.
const CLASS_TAIL: &str = ".Commands";

/// `ERROR_ALREADY_EXISTS`: the mutex was there before, so another copy holds it.
const ERROR_ALREADY_EXISTS: u32 = 183;
/// `ERROR_ACCESS_DENIED`: a copy at another level (such as elevated) holds the mutex.
const ERROR_ACCESS_DENIED: u32 = 5;

/// The command named `name`, such as `show`; anything else is refused.
#[must_use]
pub fn parse(name: &str) -> Option<Command> {
    COMMANDS.iter().find(|c| c.1 == name).map(|c| c.0)
}

/// The name of `command`.
#[must_use]
pub fn name(command: Command) -> &'static str {
    COMMANDS.iter().find(|c| c.0 == command).map_or("", |c| c.1)
}

/// The code `command` travels as.
fn code(command: Command) -> usize {
    COMMANDS.iter().find(|c| c.0 == command).map_or(0, |c| c.2)
}

/// The command that travels as `code`; anything else is refused.
fn from_code(code: usize) -> Option<Command> {
    COMMANDS.iter().find(|c| c.2 == code).map(|c| c.0)
}

/// The session-local mutex name of app `app`.
fn mutex_name(app: &str) -> String {
    format!("{SESSION}{app}{MUTEX_TAIL}")
}

/// The class name of app `app`'s command window.
fn class_name(app: &str) -> String {
    format!("{app}{CLASS_TAIL}")
}

/// What the session's mutex says about this copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Hold {
    /// This copy made the mutex, so it is the running one.
    Ours,
    /// Another copy holds the mutex.
    Taken,
    /// Windows could not check, so this copy runs as the only one.
    Unknown,
}

/// What making the mutex means: `made` says whether Windows gave a handle, `error` is its last error.
fn hold_of(made: bool, error: u32) -> Hold {
    match (made, error) {
        (true, ERROR_ALREADY_EXISTS) | (false, ERROR_ACCESS_DENIED) => Hold::Taken,
        (true, _) => Hold::Ours,
        (false, _) => Hold::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_command_reads_back_by_name_and_by_code() {
        for command in [Command::Show, Command::Quit] {
            assert_eq!(parse(name(command)), Some(command));
            assert_eq!(from_code(code(command)), Some(command));
        }
        assert_eq!(parse("show"), Some(Command::Show));
        assert_eq!(parse("quit"), Some(Command::Quit));
    }

    #[test]
    fn anything_else_is_refused() {
        for text in ["", "Show", " show", "show ", "exit", "quit\0", "ﺷ"] {
            assert_eq!(parse(text), None, "{text:?}");
        }
        for code in [0, 3, usize::MAX] {
            assert_eq!(from_code(code), None, "{code}");
        }
    }

    #[test]
    fn no_command_has_code_zero_and_codes_differ() {
        assert_ne!(code(Command::Show), 0);
        assert_ne!(code(Command::Show), code(Command::Quit));
    }

    #[test]
    fn the_mutex_is_session_local_and_both_names_follow_the_app() {
        let mutex = mutex_name("KeyXtend");
        assert!(mutex.starts_with(SESSION), "{mutex}");
        assert!(mutex.contains("KeyXtend") && class_name("KeyXtend").contains("KeyXtend"));
        assert_ne!(mutex_name("A"), mutex_name("B"));
        assert_ne!(class_name("A"), class_name("B"));
    }

    #[test]
    fn a_new_mutex_is_ours_and_one_made_before_is_taken() {
        assert_eq!(hold_of(true, 0), Hold::Ours);
        assert_eq!(hold_of(true, ERROR_ALREADY_EXISTS), Hold::Taken);
        assert_eq!(
            hold_of(false, ERROR_ACCESS_DENIED),
            Hold::Taken,
            "a copy at another level holds it"
        );
    }

    #[test]
    fn when_windows_cannot_check_the_copy_runs_alone() {
        assert_eq!(hold_of(false, 0), Hold::Unknown);
        assert_eq!(hold_of(false, ERROR_ALREADY_EXISTS), Hold::Unknown);
        assert_eq!(
            hold_of(true, ERROR_ACCESS_DENIED),
            Hold::Ours,
            "a stale error"
        );
    }

    proptest::proptest! {
        #[test]
        fn any_text_or_code_reads_without_panic(text in r"\PC{0,12}", code: usize) {
            let _ = (parse(&text), from_code(code));
        }
    }

    #[cfg(windows)]
    #[test]
    fn the_error_table_matches_windows() {
        use windows::Win32::Foundation::{
            ERROR_ACCESS_DENIED as DENIED, ERROR_ALREADY_EXISTS as EXISTS,
        };
        assert_eq!(ERROR_ALREADY_EXISTS, EXISTS.0);
        assert_eq!(ERROR_ACCESS_DENIED, DENIED.0);
    }
}
