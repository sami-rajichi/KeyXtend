//! `cargo xtask check-uiaccess <exe>`: checks the three conditions Windows needs for uiAccess.

use std::path::Path;

use super::config::{self, DevSetup, DevToolsConfig};
use super::{DevError, paths, ps, sdk};

/// Report name of the secure-folder check.
const FOLDER_CHECK: &str = "in a secure folder";
/// Report name of the signature check.
const SIGNATURE_CHECK: &str = "trusted signature";
/// Report name of the manifest check.
const MANIFEST_CHECK: &str = "uiAccess in the manifest";

/// `signtool` interface: verify with the default Authenticode policy, embedded or catalog signature.
const VERIFY: [&str; 3] = ["verify", "/pa", "/a"];
/// `mt.exe` flag that hides its banner.
const MT_NOLOGO: &str = "-nologo";
/// `mt.exe` flag prefix naming the file whose manifest resource is read.
const MT_INPUT: &str = "-inputresource:";
/// Resource id of an executable's manifest.
const MT_MANIFEST_ID: &str = ";#1";
/// `mt.exe` flag prefix naming the output file.
const MT_OUT: &str = "-out:";
/// XML comment delimiters.
const COMMENT: (&str, &str) = ("<!--", "-->");
/// XML character that opens an element.
const TAG_OPEN: char = '<';
/// XML separator between a namespace prefix and a name.
const PREFIX_SEP: char = ':';
/// Characters besides letters and digits that an XML prefix may hold.
const PREFIX_EXTRA: &str = "_-.";
/// Report words for a passed and a failed check.
const VERDICT: (&str, &str) = ("PASS", "FAIL");

/// One checked condition and what was found.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Check {
    /// What was checked.
    pub name: &'static str,
    /// Whether it passed.
    pub passed: bool,
    /// What was found, for the report.
    pub detail: String,
}

/// The value of `attr` inside the first `<tag …>` element of `xml`, ignoring comments.
pub(crate) fn manifest_value(xml: &str, tag: &str, attr: &str) -> Option<String> {
    if attr.is_empty() || tag.is_empty() {
        return None;
    }
    let xml = strip_comments(xml);
    attr_value(element(&xml, tag)?, attr)
}

/// `xml` with every `<!-- … -->` comment removed; an unclosed comment runs to the end.
fn strip_comments(xml: &str) -> String {
    let (open, close) = COMMENT;
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find(open) {
        out.push_str(&rest[..start]);
        rest = rest[start..]
            .find(close)
            .map_or("", |end| &rest[start + end + close.len()..]);
    }
    out.push_str(rest);
    out
}

/// The text of the first `<tag …>` or `<prefix:tag …>` element, up to its closing `>`.
fn element<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let mut from = 0;
    while let Some(pos) = xml[from..].find(tag) {
        let start = from + pos + tag.len();
        from = start;
        let next = xml[start..].chars().next();
        let ends = next.is_some_and(|c| c.is_whitespace() || c == '/' || c == '>');
        if ends && opens_element(&xml[..start - tag.len()]) {
            return xml[start..].find('>').map(|end| &xml[start..start + end]);
        }
    }
    None
}

/// True if `before` ends with `<` or with `<prefix:`, so the name after it starts an element.
fn opens_element(before: &str) -> bool {
    let Some(rest) = before.strip_suffix(PREFIX_SEP) else {
        return before.ends_with(TAG_OPEN);
    };
    let name = rest.trim_end_matches(|c: char| c.is_alphanumeric() || PREFIX_EXTRA.contains(c));
    name.len() < rest.len() && name.ends_with(TAG_OPEN)
}

/// The value of `attr` in an element's text, matching whole attribute names only.
fn attr_value(text: &str, attr: &str) -> Option<String> {
    let mut from = 0;
    while let Some(pos) = text[from..].find(attr) {
        let start = from + pos;
        from = start + attr.len();
        let at_boundary = text[..start]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace);
        if let (true, Some(value)) = (at_boundary, quoted_value(&text[from..])) {
            return Some(value);
        }
    }
    None
}

/// Reads `= "value"` or `= 'value'` at the start of `rest`, allowing spaces around `=`.
fn quoted_value(rest: &str) -> Option<String> {
    let rest = rest.trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next().filter(|q| *q == '"' || *q == '\'')?;
    let body = &rest[quote.len_utf8()..];
    body.find(quote).map(|end| body[..end].to_string())
}

/// Passes only if the manifest turns uiAccess on.
pub(crate) fn manifest_check(xml: &str, config: &DevToolsConfig) -> Check {
    let value = manifest_value(xml, &config.ui_access_tag, &config.ui_access_attr);
    Check {
        name: MANIFEST_CHECK,
        passed: value.as_deref() == Some(config.ui_access_on.as_str()),
        detail: format!("{}={}", config.ui_access_attr, value.unwrap_or_default()),
    }
}

/// Passes if `exe` is inside one of the secure folders named by the environment.
pub(crate) fn check_folder(
    config: &DevToolsConfig,
    exe: &Path,
    env: impl Fn(&str) -> Option<String>,
) -> Check {
    let roots = config.secure_roots(env);
    let listed: Vec<String> = roots.iter().map(|r| paths::text(r)).collect();
    Check {
        name: FOLDER_CHECK,
        passed: roots.iter().any(|root| paths::is_under(exe, root)),
        detail: listed.join(", "),
    }
}

/// `signtool` arguments that verify the signature of `exe`.
pub(crate) fn verify_args(exe: &Path) -> Vec<String> {
    let mut args: Vec<String> = VERIFY.iter().map(ToString::to_string).collect();
    args.push(paths::text(exe));
    args
}

/// `mt.exe` arguments that write the manifest of `exe` to `out`.
pub(crate) fn mt_args(exe: &Path, out: &Path) -> Vec<String> {
    vec![
        MT_NOLOGO.to_string(),
        format!("{MT_INPUT}{}{MT_MANIFEST_ID}", paths::text(exe)),
        format!("{MT_OUT}{}", paths::text(out)),
    ]
}

/// One report line: `PASS` or `FAIL`, the condition, and what was found.
pub(crate) fn report_line(check: &Check) -> String {
    let verdict = if check.passed { VERDICT.0 } else { VERDICT.1 };
    if check.detail.is_empty() {
        format!("{verdict}  {}", check.name)
    } else {
        format!("{verdict}  {}: {}", check.name, check.detail)
    }
}

/// Turns a step result into a check: a failed step fails the check, other errors stop the run.
fn step_check(name: &'static str, result: Result<String, DevError>) -> Result<Check, DevError> {
    match result {
        Ok(_) => Ok(Check {
            name,
            passed: true,
            detail: String::new(),
        }),
        Err(DevError::Failed { detail, .. }) => Ok(Check {
            name,
            passed: false,
            detail,
        }),
        Err(other) => Err(other),
    }
}

/// Checks that the signature chains to a trusted root.
fn check_signature(setup: &DevSetup, exe: &Path) -> Result<Check, DevError> {
    let tool = sdk::find_tool(&setup.config, &setup.config.signtool)?;
    let result = ps::run(
        &setup.config.signtool,
        &paths::text(&tool),
        &verify_args(exe),
    );
    step_check(SIGNATURE_CHECK, result)
}

/// Extracts the embedded manifest with `mt.exe` and checks it; the temp file is always deleted.
fn check_manifest(setup: &DevSetup, exe: &Path) -> Result<Check, DevError> {
    let c = &setup.config;
    let tool = sdk::find_tool(c, &c.mt)?;
    let out = setup.out(&c.manifest_file);
    remove_if_present(&out)?;
    let ran = ps::run(&c.mt, &paths::text(&tool), &mt_args(exe, &out));
    let step = step_check(MANIFEST_CHECK, ran);
    if !matches!(step, Ok(Check { passed: true, .. })) {
        remove_if_present(&out)?;
        return step;
    }
    Ok(manifest_check(&read_and_remove(&out)?, c))
}

/// Reads `path` as text, replacing bad bytes, and deletes it even when the read fails.
fn read_and_remove(path: &Path) -> Result<String, DevError> {
    let read = std::fs::read(path);
    remove_if_present(path)?;
    Ok(String::from_utf8_lossy(&read?).into_owned())
}

/// Deletes `path` if it exists.
fn remove_if_present(path: &Path) -> Result<(), DevError> {
    match std::fs::remove_file(path) {
        Err(err) if err.kind() != std::io::ErrorKind::NotFound => Err(err.into()),
        _ => Ok(()),
    }
}

/// Runs `cargo xtask check-uiaccess <exe>`; returns true if all three checks pass.
pub(crate) fn run(exe: &str) -> Result<bool, DevError> {
    let setup = config::from_cargo()?;
    let exe = std::path::absolute(exe)?;
    if !exe.is_file() {
        return Err(DevError::NotAFile(paths::text(&exe)));
    }
    std::fs::create_dir_all(setup.out(""))?;
    let checks = [
        check_folder(&setup.config, &exe, |key| std::env::var(key).ok()),
        check_signature(&setup, &exe)?,
        check_manifest(&setup, &exe)?,
    ];
    for check in &checks {
        println!("{}", report_line(check));
    }
    Ok(checks.iter().all(|c| c.passed))
}

#[cfg(test)]
mod tests;
