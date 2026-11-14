//! `boxed --mb <n> <program> [args…]`: starts a program under the worker's own limits (restricted token, Medium
//! integrity, no uiAccess, its job's memory cap) and reports its exit code and what it printed. P11 uses it to prove
//! that the microphone and a graphics card work inside the worker's box before the voice job is built.
#![cfg(windows)]

use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use kx_platform::worker::{Child, Launch};
use kx_platform_windows::WinSpawner;
use serde_json::{Value, json};

/// The command's word.
pub const COMMAND: &str = "boxed";
/// The flag that sets the memory cap, in MB.
const CAP_FLAG: &str = "--mb";

/// One boxed run, as the command line asks it.
#[derive(Debug)]
struct Boxed {
    /// The program, an absolute path.
    exe: PathBuf,
    /// The job's memory cap, in MB.
    memory_mb: u32,
    /// The program's own arguments, in order.
    args: Vec<String>,
}

/// The usage line.
fn usage() -> String {
    format!("usage: kx-gates {COMMAND} {CAP_FLAG} <MB> <absolute program path> [arguments…]")
}

/// Reads `--mb <n> <program> [args…]`; the cap is required, above zero, and the program's path absolute.
fn parse(rest: &[String]) -> Result<Boxed, String> {
    let [flag, mb, exe, args @ ..] = rest else {
        return Err(usage());
    };
    if flag != CAP_FLAG {
        return Err(usage());
    }
    let memory_mb = mb
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(usage)?;
    let exe = PathBuf::from(exe);
    if !exe.is_absolute() {
        return Err(usage());
    }
    Ok(Boxed {
        exe,
        memory_mb,
        args: args.to_vec(),
    })
}

/// Runs the program in the box, waits for it to end, and reports its code and output.
pub fn run(rest: &[String]) -> Result<Value, String> {
    let b = parse(rest)?;
    let name = b.exe.file_name().map(|n| n.to_string_lossy().into_owned());
    let launch = Launch {
        args: b.args,
        memory_mb: b.memory_mb,
    };
    let (input, mut output, mut child) = WinSpawner::at(b.exe)
        .start(&launch)
        .map_err(|e| e.to_string())?;
    // The program reads nothing from us; closing its stdin lets it end.
    drop(input);
    let mut bytes = Vec::new();
    output.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    // Its stdout closed, so it is ending; the wait is the longest `code` allows.
    let code = child.code(Duration::MAX);
    Ok(json!({
        "gate": COMMAND.to_uppercase(),
        "program": name,
        "memory_mb": b.memory_mb,
        "code": code,
        "pass": code == Some(0),
        "output": String::from_utf8_lossy(&bytes),
    }))
}

#[cfg(test)]
mod tests;
