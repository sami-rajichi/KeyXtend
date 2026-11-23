//! The table in two files: `services.toml` holds the table-wide values, the notes and the upload-form rows, and
//! `services-own.toml` only the rows of services with their own shape. Both read as one table, form rows first, under
//! one validator; a table-wide key in the second file is refused, and each file stays under the size that splits it.

use super::*;

/// The lines at which the owner's size rule splits a file.
const SPLIT_AT: usize = 400;

/// Each row's id, in order.
fn ids(table: &Services) -> Vec<&str> {
    table.rows().iter().map(|r| r.id.as_str()).collect()
}

#[test]
fn the_two_files_read_as_one_table_the_form_rows_first() {
    let form = Services::parse(TEXT, "", LANGS).unwrap();
    assert!(
        form.rows()
            .iter()
            .all(|r| matches!(r.shape, CloudShape::Form(_))),
        "only form rows in the first file"
    );
    let all = ids(shipped());
    let (first, rest) = all.split_at(form.rows().len());
    assert_eq!(first, ids(&form), "the form rows first");
    assert_eq!(rest, SHAPED, "then the rows of their own shape");
}

#[test]
fn a_table_wide_key_in_the_second_file_is_refused() {
    for key in [
        head_line("light_hours"),
        head_line("room_kb"),
        "[reply]\nformats = []",
    ] {
        let own = format!("{key}\n{OWN}");
        let got = Services::parse(TEXT, &own, LANGS);
        assert!(matches!(got, Err(ServicesError::Own(_))), "{key}: {got:?}");
    }
}

#[test]
fn the_second_files_rows_meet_the_same_rules() {
    let header = row_edit(OWN, "elevenlabs", &[("header", "header = \"x y\"")]);
    let got = Services::parse(TEXT, &header, LANGS);
    let why = Some(ServiceError::Header);
    assert_eq!(got.err().and_then(row_why), why, "a row's rule");
    let twice = Services::parse(TEXT, &format!("{OWN}{OWN}"), LANGS);
    assert!(matches!(twice, Err(ServicesError::Id(_))), "an id twice");
}

#[test]
fn each_file_stays_under_the_size_at_which_a_file_is_split() {
    for (name, text) in [("services.toml", TEXT), ("services-own.toml", OWN)] {
        assert!(text.lines().count() < SPLIT_AT, "{name}");
    }
}

/// The rule a row broke, if the table was refused for one.
fn row_why(error: ServicesError) -> Option<ServiceError> {
    match error {
        ServicesError::Row { why, .. } => Some(why),
        _ => None,
    }
}
