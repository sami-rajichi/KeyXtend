//! `cargo xtask dev-cert`: the self-signed certificate that signs uiAccess test builds.

use super::config::{self, DevSetup, DevToolsConfig};
use super::launcher::{self, Launch};
use super::{DevError, paths, ps, script};

/// Length of a SHA-1 certificate thumbprint in hex digits.
const THUMBPRINT_LEN: usize = 40;

/// Certificate-store steps, so the flow is testable without a real store.
pub(crate) trait CertStore {
    /// The thumbprint of a test certificate valid for long enough, if one exists.
    fn find(&mut self) -> Result<Option<String>, DevError>;
    /// Creates a test certificate and returns its thumbprint.
    fn create(&mut self) -> Result<String, DevError>;
    /// Exports the certificate's public part to the configured file.
    fn export(&mut self, thumbprint: &str) -> Result<(), DevError>;
    /// The thumbprints of every test certificate, valid or not.
    fn list(&mut self) -> Result<Vec<String>, DevError>;
    /// Deletes every test certificate and its key.
    fn remove(&mut self) -> Result<(), DevError>;
}

/// Finds or creates the test certificate, exports it, and returns its thumbprint.
pub(crate) fn ensure(store: &mut impl CertStore) -> Result<String, DevError> {
    let thumbprint = match store.find()? {
        Some(existing) => existing,
        None => store.create()?,
    };
    store.export(&thumbprint)?;
    Ok(thumbprint)
}

/// Lists the certificates, lets `write` record them, then deletes them; none means no action.
pub(crate) fn retire(
    store: &mut impl CertStore,
    write: impl FnOnce(&[String]) -> Result<(), DevError>,
) -> Result<Vec<String>, DevError> {
    let list = store.list()?;
    if !list.is_empty() {
        write(&list)?;
        store.remove()?;
    }
    Ok(list)
}

/// The last non-blank line of `text`, trimmed.
fn last_line(text: &str) -> &str {
    text.lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or_default()
}

/// Parses one thumbprint: exactly 40 hex digits, returned in upper case.
fn thumbprint(line: &str) -> Result<String, DevError> {
    if line.len() == THUMBPRINT_LEN && line.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(line.to_ascii_uppercase())
    } else {
        Err(DevError::BadThumbprint(line.to_string()))
    }
}

/// Parses a thumbprint from the last non-blank line of script output.
pub(crate) fn parse_thumbprint(stdout: &str) -> Result<String, DevError> {
    thumbprint(last_line(stdout))
}

/// Like [`parse_thumbprint`], but blank output means no certificate was found.
pub(crate) fn parse_found(stdout: &str) -> Result<Option<String>, DevError> {
    if last_line(stdout).is_empty() {
        Ok(None)
    } else {
        parse_thumbprint(stdout).map(Some)
    }
}

/// Parses one thumbprint per non-blank line.
pub(crate) fn parse_list(stdout: &str) -> Result<Vec<String>, DevError> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(thumbprint)
        .collect()
}

/// Parameters for finding a certificate that stays valid long enough.
pub(crate) fn find_params(config: &DevToolsConfig) -> Vec<(&'static str, String)> {
    vec![(script::MIN_DAYS, config.cert_min_days.to_string())]
}

/// Parameters for creating a certificate.
pub(crate) fn create_params(config: &DevToolsConfig) -> Vec<(&'static str, String)> {
    vec![
        (script::DAYS, config.cert_days.to_string()),
        (script::DIGEST, config.digest.clone()),
    ]
}

/// All parameters of one `dev-cert.ps1` action: action, subject, store, then `extra`.
pub(crate) fn action_params(
    config: &DevToolsConfig,
    action: &str,
    extra: Vec<(&'static str, String)>,
) -> Vec<(&'static str, String)> {
    let mut params = vec![
        (script::ACTION, action.to_string()),
        (script::SUBJECT, config.cert_subject.clone()),
        (script::STORE, config.user_store.clone()),
    ];
    params.extend(extra);
    params
}

/// The certificate store behind `dev-cert.ps1`.
pub(crate) struct ScriptStore<'a> {
    /// Settings and workspace root.
    pub setup: &'a DevSetup,
}

impl ScriptStore<'_> {
    /// Runs one `dev-cert.ps1` action; it never shows a prompt, so no exit code maps to cancelled.
    fn action(&self, action: &str, extra: Vec<(&'static str, String)>) -> Result<String, DevError> {
        let c = &self.setup.config;
        let params = action_params(c, action, extra);
        ps::run_script(self.setup, &c.cert_script, &params, None)
    }
}

impl CertStore for ScriptStore<'_> {
    fn find(&mut self) -> Result<Option<String>, DevError> {
        parse_found(&self.action(script::FIND, find_params(&self.setup.config))?)
    }

    fn create(&mut self) -> Result<String, DevError> {
        parse_thumbprint(&self.action(script::CREATE, create_params(&self.setup.config))?)
    }

    fn export(&mut self, thumbprint: &str) -> Result<(), DevError> {
        let file = self.setup.out(&self.setup.config.cert_file);
        let extra = vec![
            (script::THUMBPRINT, thumbprint.to_string()),
            (script::CERT_FILE, paths::text(&file)),
        ];
        self.action(script::EXPORT, extra).map(|_| ())
    }

    fn list(&mut self) -> Result<Vec<String>, DevError> {
        parse_list(&self.action(script::LIST, Vec::new())?)
    }

    fn remove(&mut self) -> Result<(), DevError> {
        self.action(script::REMOVE, Vec::new()).map(|_| ())
    }
}

/// Runs `cargo xtask dev-cert`, or `dev-cert --remove` when `remove` is set.
pub(crate) fn run(remove: bool) -> Result<(), DevError> {
    let setup = config::from_cargo()?;
    std::fs::create_dir_all(setup.out(""))?;
    if remove {
        remove_cert(&setup)
    } else {
        add_cert(&setup)
    }
}

/// Finds or creates the certificate, then writes the trust launcher.
fn add_cert(setup: &DevSetup) -> Result<(), DevError> {
    let shell = setup.powershell(|key| std::env::var(key).ok())?;
    let thumbprint = ensure(&mut ScriptStore { setup })?;
    let launch = Launch::Trust(thumbprint.clone());
    let path = launcher::write(setup, &shell, &setup.config.trust_launcher, &launch)?;
    println!("Test certificate {thumbprint} is ready.");
    println!("To trust it, open {} and click Yes.", paths::text(&path));
    Ok(())
}

/// Writes the untrust launcher for every test certificate, then deletes them from the user store.
fn remove_cert(setup: &DevSetup) -> Result<(), DevError> {
    let shell = setup.powershell(|key| std::env::var(key).ok())?;
    let name = &setup.config.untrust_launcher;
    let mut path = None;
    let removed = retire(&mut ScriptStore { setup }, |list| {
        path = Some(launcher::write(
            setup,
            &shell,
            name,
            &Launch::Untrust(list.to_vec()),
        )?);
        Ok(())
    })?;
    match path {
        None => println!("No test certificate in your store."),
        Some(path) => {
            println!(
                "Removed {} test certificate(s) and their keys.",
                removed.len()
            );
            println!(
                "To stop Windows trusting them, open {} and click Yes.",
                paths::text(&path)
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
