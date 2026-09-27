//! `.cmd` launchers the owner opens to trust or untrust the test certificate.

use std::path::{Path, PathBuf};

use super::config::DevSetup;
use super::{DevError, admin, paths, ps, script};

/// Line ending of `.cmd` files.
const CMD_EOL: &str = "\r\n";
/// First line: do not echo commands.
const CMD_HEADER: &str = "@echo off";
/// Second line: read the rest as UTF-8, so non-ASCII paths survive.
const CMD_UTF8: &str = "chcp 65001>nul";
/// Characters that make cmd split or reinterpret an argument unless it is quoted.
const CMD_SPECIAL: &str = "&|<>^()";
/// The quote character, which cannot appear inside a quoted argument.
const QUOTE: char = '"';
/// A trailing backslash would escape the closing quote.
const ESCAPE: char = '\\';

/// What a launcher asks the admin script to do.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Launch {
    /// Trust the exported certificate with this thumbprint.
    Trust(String),
    /// Stop trusting the certificates with these thumbprints and the configured subject.
    Untrust(Vec<String>),
}

/// The text of a `.cmd` launcher that runs the admin script with `shell`.
pub(crate) fn text(setup: &DevSetup, shell: &Path, launch: &Launch) -> String {
    let c = &setup.config;
    let mut params = match launch {
        Launch::Trust(thumbprint) => vec![
            (script::ACTION, script::TRUST.to_string()),
            (script::CERT_FILE, paths::text(&setup.out(&c.cert_file))),
            (script::THUMBPRINT, thumbprint.clone()),
        ],
        Launch::Untrust(thumbprints) => vec![
            (script::ACTION, script::UNTRUST.to_string()),
            (script::SUBJECT, c.cert_subject.clone()),
            (script::THUMBPRINT, thumbprints.join(script::LIST_SEP)),
        ],
    };
    params.extend(admin::codes(c));
    let mut args = vec![paths::text(shell)];
    args.extend(ps::script_args(&setup.script(&c.admin_script), &params));
    args.push(script::NOTIFY.to_string());
    let line: Vec<String> = args.iter().map(|a| cmd_arg(a)).collect();
    [CMD_HEADER, CMD_UTF8, &line.join(" ")].join(CMD_EOL) + CMD_EOL
}

/// Escapes one argument for a `.cmd` file: doubles `%` and quotes it when needed.
fn cmd_arg(arg: &str) -> String {
    let escaped = arg.replace('%', "%%");
    let special = escaped
        .chars()
        .any(|c| c.is_whitespace() || CMD_SPECIAL.contains(c));
    if escaped.is_empty() || special {
        format!("{QUOTE}{escaped}{QUOTE}")
    } else {
        escaped
    }
}

/// True if `arg` survives quoting unchanged: no `"` inside and no `\` at the end.
pub(crate) fn is_quotable(arg: &str) -> bool {
    !arg.contains(QUOTE) && !arg.ends_with(ESCAPE)
}

/// Writes a launcher named `name` into the output folder and returns its path.
pub(crate) fn write(
    setup: &DevSetup,
    shell: &Path,
    name: &str,
    launch: &Launch,
) -> Result<PathBuf, DevError> {
    let path = setup.out(name);
    std::fs::write(&path, text(setup, shell, launch))?;
    Ok(path)
}

#[cfg(test)]
mod tests;
