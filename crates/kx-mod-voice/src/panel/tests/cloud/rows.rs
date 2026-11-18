//! A Cloud page row carries its card's warning fact id, so the row's mark names that warning, not a fixed one.

use kx_module_api::ports::ServiceName;

use super::{cloud_sheet, on_cloud};
use crate::services::tests::ROUTER;

#[test]
fn a_row_carries_its_cards_warning_fact_id_and_none_without_one() {
    let mut rig = on_cloud(&format!("{ROUTER}warn = {{ id = \"keeps-logs\" }}\n"));
    let sheet = cloud_sheet(&mut rig);
    let router = ServiceName::Named("OpenRouter".into());
    assert_eq!(sheet.rows.len(), 3, "Groq, Custom and OpenRouter");
    for row in &sheet.rows {
        let want = (row.name == router).then(|| "keeps-logs".to_owned());
        assert_eq!(row.warn, want, "{:?}", row.name);
    }
}
