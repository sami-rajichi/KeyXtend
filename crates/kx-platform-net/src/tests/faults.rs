//! Each HTTP error becomes the fetch fault the host shows.

use crate::NetError;
use kx_ipc::FetchFault;

#[test]
fn each_error_maps_to_its_fetch_fault() {
    let cases = [
        (NetError::NotHttps, FetchFault::NotHttps),
        (NetError::Unlisted, FetchFault::Redirect),
        (NetError::Hops, FetchFault::Server),
        (NetError::Status(404), FetchFault::Server),
        (NetError::Range, FetchFault::Server),
        (NetError::Server, FetchFault::Server),
        (NetError::Stall, FetchFault::Stall),
        (NetError::Net, FetchFault::Net),
        (NetError::Internal, FetchFault::Internal),
    ];
    for (e, fault) in cases {
        assert_eq!(FetchFault::from(e), fault, "{e:?}");
    }
}

#[test]
fn an_error_names_no_address_or_server_text() {
    assert_eq!(
        NetError::Status(503).to_string(),
        "the server answered status 503"
    );
    assert!(!NetError::Unlisted.to_string().contains("http"));
}
