//! The gate's own PowerShell scripts for the terminals: one prints the test line and keeps each pasted line, one names
//! its own process and keeps the configured ping writing to a log; and how many Num Lock taps turn the lock as wanted.
#![cfg(windows)]

use std::path::Path;

use crate::p10cfg::G19Product;

/// Marks the script as UTF-8, so PowerShell reads it whole.
const BOM: char = '\u{FEFF}';

/// `s` made safe inside a single-quoted PowerShell string.
fn quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// `path` as a single-quoted PowerShell string.
fn path(p: &Path) -> String {
    quoted(&p.to_string_lossy())
}

/// The line that marks the script ready in `ready`.
fn mark(ready: &Path) -> String {
    format!("[IO.File]::WriteAllText({}, 'ready')", path(ready))
}

/// The script that prints `line`, marks itself ready in `ready`, then appends each line typed to `result`.
pub fn lines(line: &str, ready: &Path, result: &Path) -> String {
    format!(
        "{BOM}Write-Host {}\r\n{}\r\nwhile ($true) {{ $t = Read-Host; [IO.File]::AppendAllText({}, \"$t`n\", \
         [Text.UTF8Encoding]::new($false)) }}\r\n",
        quoted(line),
        mark(ready),
        path(result)
    )
}

/// The line that writes the script's own process id into `ready`, which marks it ready too.
fn own_id(ready: &Path) -> String {
    format!("[IO.File]::WriteAllText({}, \"$PID\")", path(ready))
}

/// The process id a script's ready file holds, `mark`; none when it holds none.
pub fn shell_of(mark: &str) -> Option<u32> {
    mark.trim().parse().ok().filter(|p| *p != 0)
}

/// The script that writes its own process id into `ready`, then runs `g`'s ping until closed, each reply shown and
/// kept in `log`.
pub fn ping(g: &G19Product, ready: &Path, log: &Path) -> String {
    let command = [g.ping_exe.as_str()]
        .into_iter()
        .chain(g.ping_args.iter().map(String::as_str))
        .chain([g.ping_host.as_str()])
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{BOM}{}\r\n{command} | ForEach-Object {{ $_; [IO.File]::AppendAllText({}, \"$_`n\") }}\r\n",
        own_id(ready),
        path(log)
    )
}

/// The Num Lock taps that turn it from `now` to `want`.
pub fn taps(now: bool, want: bool) -> usize {
    usize::from(now != want)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_script_prints_its_line_marks_ready_and_keeps_each_typed_line() {
        let s = lines(
            "It's here.",
            Path::new(r"D:\o\r.txt"),
            Path::new(r"D:\o\it's.txt"),
        );
        assert!(s.contains("Write-Host 'It''s here.'"), "{s}");
        assert!(
            s.contains(r"'D:\o\r.txt'") && s.contains(r"'D:\o\it''s.txt'"),
            "{s}"
        );
        assert!(s.contains("Read-Host") && s.contains("while"), "{s}");
        assert!(s.find("Write-Host") < s.find("Read-Host"), "printed first");
    }

    #[test]
    fn the_ping_script_marks_ready_before_it_pings_and_keeps_each_reply() {
        let g = crate::config::load().expect("loads").g19_product;
        let s = ping(&g, Path::new(r"D:\o\r.txt"), Path::new(r"D:\o\p.log"));
        let command = format!("{} {} {}", g.ping_exe, g.ping_args.join(" "), g.ping_host);
        assert!(s.contains(&command), "{s}");
        assert!(s.find(r"D:\o\r.txt") < s.find(&g.ping_exe), "{s}");
        assert!(
            s.contains(r#"WriteAllText('D:\o\r.txt', "$PID")"#),
            "its own id: {s}"
        );
        assert!(s.contains(r"'D:\o\p.log'"), "{s}");
    }

    #[test]
    fn the_ready_file_names_the_scripts_shell_or_nothing() {
        assert_eq!(shell_of("4242\r\n"), Some(4242));
        assert_eq!(shell_of("ready"), None, "the line script's mark");
        assert_eq!((shell_of(""), shell_of("0")), (None, None));
    }

    #[test]
    fn numlock_is_tapped_only_when_it_differs() {
        assert_eq!(taps(true, true), 0);
        assert_eq!(taps(false, false), 0);
        assert_eq!(taps(true, false), 1);
        assert_eq!(taps(false, true), 1);
    }
}
