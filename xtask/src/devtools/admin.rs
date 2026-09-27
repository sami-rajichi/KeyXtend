//! Parameters and runs for `dev-admin.ps1`, which asks Windows for admin rights when needed.

use std::path::Path;

use super::config::{DevSetup, DevToolsConfig};
use super::{DevError, paths, ps, script};

/// The exit codes every admin run needs.
pub(crate) fn codes(config: &DevToolsConfig) -> Vec<(&'static str, String)> {
    vec![
        (script::CANCEL_CODE, config.cancel_code.to_string()),
        (script::FAIL_CODE, config.fail_code.to_string()),
    ]
}

/// Parameters to install `source` into `target`, or to remove `target` when `source` is `None`.
pub(crate) fn build_params(
    config: &DevToolsConfig,
    source: Option<&Path>,
    target: &Path,
) -> Vec<(&'static str, String)> {
    let action = if source.is_some() {
        script::INSTALL
    } else {
        script::UNINSTALL
    };
    let mut params = vec![
        (script::ACTION, action.to_string()),
        (script::TARGET, paths::text(target)),
        (script::INSTALL_DIR, config.install_dir.clone()),
        (script::SECURE_ENV, config.install_env().to_string()),
    ];
    params.extend(codes(config));
    if let Some(source) = source {
        params.push((script::SOURCE, paths::text(source)));
    }
    params
}

/// Runs an install or uninstall; the admin window's error comes back on stderr.
pub(crate) fn run(
    setup: &DevSetup,
    mut params: Vec<(&'static str, String)>,
) -> Result<(), DevError> {
    let error_file = setup.out(&setup.config.admin_error_file);
    params.push((script::ERROR_FILE, paths::text(&error_file)));
    let cancel = Some(setup.config.cancel_code);
    ps::run_script(setup, &setup.config.admin_script, &params, cancel).map(|_| ())
}

#[cfg(test)]
mod tests;
