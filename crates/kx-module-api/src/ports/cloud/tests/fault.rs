//! A fault's kind: one plain name per variant, letters only, so a log line may hold it and never a value it carries.

use std::collections::BTreeSet;
use std::time::Duration;

use super::*;

/// Every fault, one of each variant; the match makes a new variant fail to build until it is listed here.
fn every() -> Vec<CloudFault> {
    let renews_in = Some(Duration::from_secs(3600));
    let all = vec![
        CloudFault::Range("pieces"),
        CloudFault::Key,
        CloudFault::Credit,
        CloudFault::Account,
        CloudFault::Allowance { renews_in },
        CloudFault::Rate,
        CloudFault::TooBig,
        CloudFault::Language,
        CloudFault::Region,
        CloudFault::Admin,
        CloudFault::Empty,
        CloudFault::Strange,
        CloudFault::Network,
        CloudFault::Timeout,
        CloudFault::Gone,
    ];
    for fault in &all {
        match fault {
            CloudFault::Range(_)
            | CloudFault::Key
            | CloudFault::Credit
            | CloudFault::Account
            | CloudFault::Allowance { .. }
            | CloudFault::Rate
            | CloudFault::TooBig
            | CloudFault::Language
            | CloudFault::Region
            | CloudFault::Admin
            | CloudFault::Empty
            | CloudFault::Strange
            | CloudFault::Network
            | CloudFault::Timeout
            | CloudFault::Gone => {}
        }
    }
    all
}

#[test]
fn each_fault_has_its_own_kind_of_letters_only_and_never_a_value_it_holds() {
    let all = every();
    let kinds: BTreeSet<&str> = all.iter().map(CloudFault::kind).collect();
    assert_eq!(kinds.len(), all.len(), "one kind per variant");
    for fault in &all {
        let kind = fault.kind();
        assert!(
            !kind.is_empty() && kind.bytes().all(|b| b.is_ascii_alphabetic()),
            "{kind}"
        );
        assert!(
            format!("{fault:?}").starts_with(kind),
            "{kind} names its variant"
        );
    }
    for value in ["pieces", "3600"] {
        assert!(all.iter().all(|f| !f.kind().contains(value)), "{value}");
    }
}
