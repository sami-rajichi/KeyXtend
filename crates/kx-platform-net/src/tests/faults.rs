//! Each HTTP error becomes the fetch or cloud fault the host shows.

use crate::NetError;
use kx_ipc::{FetchFault, NetFault};

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
        (NetError::Header, FetchFault::Internal),
        (NetError::Stopped, FetchFault::Internal),
        (NetError::Internal, FetchFault::Internal),
    ];
    for (e, fault) in cases {
        assert_eq!(FetchFault::from(e), fault, "{e:?}");
    }
}

#[test]
fn each_error_maps_to_its_cloud_fault() {
    let cases = [
        (NetError::NotHttps, NetFault::Range),
        (NetError::Header, NetFault::Range),
        (NetError::Unlisted, NetFault::Strange),
        (NetError::Hops, NetFault::Strange),
        (NetError::Status(302), NetFault::Strange),
        (NetError::Range, NetFault::Strange),
        (NetError::Server, NetFault::Strange),
        (NetError::Stall, NetFault::Timeout),
        (NetError::Net, NetFault::Network),
        (NetError::Internal, NetFault::Gone),
        (NetError::Stopped, NetFault::Gone),
    ];
    for (e, fault) in cases {
        assert_eq!(NetFault::from(e), fault, "{e:?}");
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
