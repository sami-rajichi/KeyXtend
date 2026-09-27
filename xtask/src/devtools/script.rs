//! The interface of the dev-tools PowerShell scripts: parameter and action names.

/// Parameter: which action to run.
pub(crate) const ACTION: &str = "Action";
/// Parameter: certificate subject.
pub(crate) const SUBJECT: &str = "Subject";
/// Parameter: days the new certificate stays valid.
pub(crate) const DAYS: &str = "Days";
/// Parameter: signature digest algorithm.
pub(crate) const DIGEST: &str = "Digest";
/// Parameter: certificate thumbprint.
pub(crate) const THUMBPRINT: &str = "Thumbprint";
/// Parameter: exported certificate file.
pub(crate) const CERT_FILE: &str = "CertFile";
/// Parameter: folder to install from.
pub(crate) const SOURCE: &str = "Source";
/// Parameter: folder to install into or remove.
pub(crate) const TARGET: &str = "Target";
/// Parameter: install folder name under Program Files.
pub(crate) const INSTALL_DIR: &str = "InstallDir";
/// Parameter: exit code for a cancelled Windows prompt.
pub(crate) const CANCEL_CODE: &str = "CancelCode";
/// Parameter: exit code for any other failure.
pub(crate) const FAIL_CODE: &str = "FailCode";
/// Parameter: env var naming the secure folder for installs.
pub(crate) const SECURE_ENV: &str = "SecureEnv";
/// Parameter: file where the admin window leaves its error message.
pub(crate) const ERROR_FILE: &str = "ErrorFile";
/// Parameter: current-user certificate store name.
pub(crate) const STORE: &str = "Store";
/// Parameter: days a reused certificate must still be valid.
pub(crate) const MIN_DAYS: &str = "MinDays";
/// Separator of a thumbprint list; `dev-admin.ps1` names the same one as `$ListSep`.
pub(crate) const LIST_SEP: &str = ",";
/// Switch: show the result in a message box.
pub(crate) const NOTIFY: &str = "-Notify";
/// Switch: check the parameters, change nothing and exit; the script tests use it.
#[cfg(test)]
pub(crate) const VALIDATE_ONLY: &str = "-ValidateOnly";

/// `dev-cert.ps1` action: print the thumbprint of a valid certificate, if any.
pub(crate) const FIND: &str = "Find";
/// `dev-cert.ps1` action: print the thumbprints of every certificate with the subject.
pub(crate) const LIST: &str = "List";
/// `dev-cert.ps1` action: create a certificate and print its thumbprint.
pub(crate) const CREATE: &str = "Create";
/// `dev-cert.ps1` action: export the public certificate to a file.
pub(crate) const EXPORT: &str = "Export";
/// `dev-cert.ps1` action: delete the certificate and its key.
pub(crate) const REMOVE: &str = "Remove";
/// `dev-admin.ps1` action: trust the exported certificate.
pub(crate) const TRUST: &str = "Trust";
/// `dev-admin.ps1` action: stop trusting the certificate.
pub(crate) const UNTRUST: &str = "Untrust";
/// `dev-admin.ps1` action: copy a build into Program Files.
pub(crate) const INSTALL: &str = "Install";
/// `dev-admin.ps1` action: delete an installed build.
pub(crate) const UNINSTALL: &str = "Uninstall";
