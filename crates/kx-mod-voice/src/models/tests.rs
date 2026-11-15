//! The models table: every row valid, the table's refusals, the orders and defaults, picks, the fit and other files.

use super::*;

mod fit;
mod orders;
mod refuse;
mod rows;
mod tested;

/// The compiled-in table, which must load.
fn shipped() -> &'static Table {
    table().unwrap()
}

/// The row of `id` in the compiled-in table.
fn row(id: &str) -> &'static Row {
    shipped().get(id).unwrap()
}

/// The folder the tests put models in.
const FOLDER: &str = r"D:\KeyXtend\models";
