//! Tidy violation types, and `cargo xtask tidy`'s rule pipeline and report format.

pub(crate) mod deps;
pub(crate) mod files;
#[cfg(test)]
pub(crate) mod testkit;

use crate::text::plural;
use crate::workspace::Workspace;

/// D1: a module depends on a workspace member outside `module_allowed`.
pub(crate) const D1: &str = "D1";
/// D2: a package other than the app or a tool depends on a platform adapter or a module.
pub(crate) const D2: &str = "D2";
/// D3: a `banned_network` crate is reachable from the app.
pub(crate) const D3: &str = "D3";
/// D4: a `banned_media` crate is reachable from the app.
pub(crate) const D4: &str = "D4";
/// D5: a shipped crate depends on a test tool or a `test_only` crate.
pub(crate) const D5: &str = "D5";
/// F1: a source file is over `warn_file_lines` or `max_file_lines`.
pub(crate) const F1: &str = "F1";
/// F2: a crate root is missing `unsafe_attr` (or `tool_unsafe_attr`, for a tool).
pub(crate) const F2: &str = "F2";

/// How serious a tidy [`Violation`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    /// Fails the tidy check.
    Error,
    /// Reported but does not fail the check.
    Warning,
}

/// One rule violation found while walking a [`crate::workspace::Workspace`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Violation {
    /// The rule that found it, e.g. `D1`.
    pub rule: &'static str,
    /// Where it was found: a package name, or a banned crate name.
    pub place: String,
    /// A one-line description; for D3/D4 this is the dependency chain.
    pub message: String,
    /// How serious it is.
    pub severity: Severity,
}

/// The word [`render`] prints before an [`Severity::Error`] violation.
const ERROR: &str = "error";
/// The word [`render`] prints before a [`Severity::Warning`] violation.
const WARNING: &str = "warning";
/// The label [`summary`] prints its line under.
const SUMMARY_LABEL: &str = "tidy";

/// Runs every tidy rule against `ws`, in a fixed order: D1-D5, then F1-F2.
pub(crate) fn run(ws: &Workspace) -> Vec<Violation> {
    let mut violations = deps::d1_module_isolation(ws);
    violations.extend(deps::d2_app_only_platform_and_module(ws));
    violations.extend(deps::d3_no_network_in_app(ws));
    violations.extend(deps::d4_no_media_in_app(ws));
    violations.extend(deps::testonly::d5_test_only_deps(ws));
    violations.extend(files::f1_file_length(ws));
    violations.extend(files::f2_unsafe_attr(ws));
    violations
}

/// Renders one violation as `error[D1] <place>: <message>` or `warning[F1] ...`.
pub(crate) fn render(v: &Violation) -> String {
    let word = match v.severity {
        Severity::Error => ERROR,
        Severity::Warning => WARNING,
    };
    format!("{word}[{}] {}: {}", v.rule, v.place, v.message)
}

/// True when none of `violations` is an [`Severity::Error`]; warnings never fail the run.
pub(crate) fn passed(violations: &[Violation]) -> bool {
    !violations.iter().any(|v| v.severity == Severity::Error)
}

/// One summary line, e.g. `tidy: 0 errors, 1 warning`.
pub(crate) fn summary(violations: &[Violation]) -> String {
    let errors = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warnings = violations.len() - errors;
    format!(
        "{SUMMARY_LABEL}: {errors} {}, {warnings} {}",
        plural(errors, ERROR),
        plural(warnings, WARNING)
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::test_support::unique_temp_dir;
    use crate::tidy::testkit::{WorkspaceBuilder, dep};
    use crate::workspace::DependencyKind;

    fn error(rule: &'static str, place: &str, message: &str) -> Violation {
        Violation {
            rule,
            place: place.to_string(),
            message: message.to_string(),
            severity: Severity::Error,
        }
    }

    fn warning(rule: &'static str, place: &str, message: &str) -> Violation {
        Violation {
            rule,
            place: place.to_string(),
            message: message.to_string(),
            severity: Severity::Warning,
        }
    }

    #[test]
    fn passed_is_true_for_no_violations() {
        assert!(passed(&[]));
    }

    #[test]
    fn passed_is_true_for_warnings_only() {
        assert!(passed(&[warning(F1, "a", "m")]));
    }

    #[test]
    fn passed_is_false_with_any_error() {
        assert!(!passed(&[warning(F1, "a", "m"), error(D1, "b", "m")]));
    }

    #[test]
    fn render_formats_an_error() {
        assert_eq!(
            render(&error(D1, "kx-mod-a", "boom")),
            "error[D1] kx-mod-a: boom"
        );
    }

    #[test]
    fn render_formats_a_warning() {
        assert_eq!(
            render(&warning(F1, "xtask/big.rs", "301 lines")),
            "warning[F1] xtask/big.rs: 301 lines"
        );
    }

    #[test]
    fn summary_counts_and_pluralizes() {
        assert_eq!(summary(&[]), "tidy: 0 errors, 0 warnings");
        assert_eq!(
            summary(&[error(D1, "a", "m"), warning(F1, "b", "m")]),
            "tidy: 1 error, 1 warning"
        );
    }

    #[test]
    fn run_on_a_missing_root_returns_exactly_the_d1_violation() {
        let ws = WorkspaceBuilder::new()
            .member(
                "kx-mod-a",
                "crates",
                vec![dep("kx-kernel", DependencyKind::Normal)],
            )
            .member("kx-kernel", "crates", vec![])
            .build();

        let violations = run(&ws);

        assert_eq!(
            violations,
            vec![error(
                D1,
                "kx-mod-a",
                "kx-mod-a depends on workspace member kx-kernel, which is not in module_allowed"
            )]
        );
    }

    #[test]
    fn run_on_a_module_depending_on_a_module_reports_only_d1() {
        let ws = WorkspaceBuilder::new()
            .member(
                "kx-mod-a",
                "crates",
                vec![dep("kx-mod-b", DependencyKind::Normal)],
            )
            .member("kx-mod-b", "crates", vec![])
            .build();

        let violations = run(&ws);

        assert_eq!(violations.len(), 1, "{violations:?}");
        assert_eq!(violations[0].rule, D1);
    }

    #[test]
    fn run_includes_d5_for_a_shipped_crate_that_depends_on_a_tool() {
        let ws = WorkspaceBuilder::new()
            .member(
                "kx-lib",
                "crates",
                vec![dep("kx-tool", DependencyKind::Normal)],
            )
            .member("kx-tool", "tools", vec![])
            .build();

        let violations = run(&ws);

        assert_eq!(violations.len(), 1, "{violations:?}");
        assert_eq!(violations[0].rule, D5);
        assert_eq!(violations[0].place, "kx-lib");
    }

    /// Plan edge case 7: a workspace with only `xtask`, and no scan dirs on disk, passes.
    #[test]
    fn run_on_an_xtask_only_workspace_with_no_scan_dirs_passes() {
        let root = unique_temp_dir("xtask-only");
        fs::create_dir_all(&root).unwrap();
        let ws = WorkspaceBuilder::new()
            .root(root.clone())
            .member("xtask", "", vec![])
            .build();

        let violations = run(&ws);

        fs::remove_dir_all(&root).ok();

        assert_eq!(violations, vec![]);
    }
}
