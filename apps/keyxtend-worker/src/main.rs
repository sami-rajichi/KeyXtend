//! The worker program (ADR-0004): `keyxtend-worker media --max-pixels <n>` decodes untrusted images for the
//! keyboard at normal rights, one request at a time over stdin and stdout.
#![forbid(unsafe_code)]

mod args;
#[cfg(test)]
mod fixture;
mod media;
mod serve;

use args::Job;
use kx_ipc::{CLEAN_EXIT, REFUSED_EXIT};
use std::fmt::Display;
use std::io;
use std::process::ExitCode;

/// The program's name, before each line it writes on stderr.
const NAME: &str = env!("CARGO_BIN_NAME");

fn main() -> ExitCode {
    match args::parse(std::env::args_os().skip(1)) {
        Ok(Job::Media { max_pixels }) => media_job(max_pixels),
        Err(e) => fail(&e),
    }
}

/// Answers draws on stdin and stdout until goodbye or the end of input.
fn media_job(max_pixels: u64) -> ExitCode {
    let draw = |codec, bytes: &[u8], side| media::draw(codec, bytes, side, max_pixels);
    match serve::answer(&mut io::stdin().lock(), &mut io::stdout().lock(), draw) {
        Ok(()) => ExitCode::from(CLEAN_EXIT),
        Err(e) => fail(&e),
    }
}

/// Says why on one line of stderr, then fails.
fn fail(why: &dyn Display) -> ExitCode {
    eprintln!("{NAME}: {why}");
    ExitCode::from(REFUSED_EXIT)
}
