//! D6: no package in the resolve turns on a banned feature, such as toml's `unbounded` nesting.

use super::super::{D6, Severity, Violation};
use crate::workspace::{Package, Workspace};

/// D6: one error for each banned feature the resolve turns on, in any package of that name.
pub(crate) fn d6_banned_features(ws: &Workspace) -> Vec<Violation> {
    let mut violations = Vec::new();
    for package in &ws.packages {
        let Some(banned) = ws.tidy.banned_features.get(&package.name) else {
            continue;
        };
        for feature in banned.iter().filter(|f| package.features.contains(f)) {
            violations.push(d6_violation(package, feature));
        }
    }
    violations
}

/// Builds one D6 violation for `package` with `feature` turned on.
fn d6_violation(package: &Package, feature: &str) -> Violation {
    Violation {
        rule: D6,
        place: package.name.clone(),
        message: format!("{} has banned feature {feature} turned on", package.name),
        severity: Severity::Error,
    }
}

#[cfg(test)]
mod tests;
