//! The models table: every row valid, the table's refusals, the orders and defaults, picks, the fit and other files.

use super::*;
use kx_test_support::paths::native_path;
use std::path::PathBuf;

mod fit;
mod orders;
mod refuse;
mod rows;
mod short;
mod tested;

/// The compiled-in table, which must load.
fn shipped() -> &'static Table {
    table().unwrap()
}

/// The row of `id` in the compiled-in table.
fn row(id: &str) -> &'static Row {
    shipped().get(id).unwrap()
}

/// The folder the tests put models in, in this system's form.
fn folder() -> PathBuf {
    native_path(r"D:\KeyXtend\models")
}
