//! The HTTPS client the network jobs ask with: `WinHTTP` on Windows, and on other systems none until P15 brings an
//! adapter, so every request there fails as `Net`.

use kx_platform_net::Http;
#[cfg(any(test, not(windows)))]
use kx_platform_net::{Call, Hop, NetError, Reply};

/// The client the shipped jobs ask with: `WinHTTP` on Windows.
#[cfg(windows)]
pub fn live() -> Box<dyn Http> {
    Box::new(kx_platform_net::WinHttp::new())
}

/// The client elsewhere: every request fails as `Net` until P15 brings an adapter.
#[cfg(not(windows))]
pub fn live() -> Box<dyn Http> {
    Box::new(Offline)
}

/// A client with no network, for systems without an adapter yet; tests check it everywhere.
#[cfg(any(test, not(windows)))]
pub struct Offline;

#[cfg(any(test, not(windows)))]
impl Http for Offline {
    fn send(&self, _: &Hop<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }

    fn deliver(&self, _: &Call<'_>) -> Result<Reply, NetError> {
        Err(NetError::Net)
    }
}
