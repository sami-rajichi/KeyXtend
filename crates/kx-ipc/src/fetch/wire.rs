//! The fetch frames' fields and their caps, on the shared field machinery in `crate::wire`.

use crate::limits::{
    MAX_HOST_BYTES, MAX_HOSTS, MAX_MODEL_BYTES, MAX_PATH_BYTES, MAX_URL_BYTES, MAX_URLS, PART_EXT,
    SHA256_BYTES,
};
use crate::wire::{EIGHT, ONE, Spec, TWO};
use std::ops::RangeInclusive;

/// The count of addresses, tried in order.
pub(crate) const URLS: Spec = Spec::new(ONE, 1, MAX_URLS);
/// One address: visible ASCII starting with `HTTPS`.
pub(crate) const URL: Spec = Spec::new(TWO, 1, MAX_URL_BYTES);
/// The count of hosts a redirect may name.
pub(crate) const HOSTS: Spec = Spec::new(ONE, 1, MAX_HOSTS);
/// One host: lower-case letters, digits, dots and hyphens.
pub(crate) const HOST: Spec = Spec::new(ONE, 1, MAX_HOST_BYTES);
/// The final path, short enough that its `.part` name fits `MAX_PATH_BYTES`.
pub(crate) const PATH: Spec = Spec::new(TWO, 1, MAX_PATH_BYTES - PART_EXT.len());
/// A model file's size.
pub(crate) const SIZE: RangeInclusive<u64> = 1..=MAX_MODEL_BYTES;

/// A download's fields after its path: the size, the fingerprint and the stall time.
const TAIL: usize = EIGHT + SHA256_BYTES + TWO;
/// A download: its addresses, its hosts, its path, then its tail.
pub(crate) const GET_LEAST: usize =
    URLS.width + URL.least() + HOSTS.width + HOST.least() + PATH.least() + TAIL;
pub(crate) const GET_MOST: usize =
    URLS.width + URLS.max * URL.most() + HOSTS.width + HOSTS.max * HOST.most() + PATH.most() + TAIL;
/// Progress: the bytes so far, then the total.
pub(crate) const PROGRESS_LEN: usize = 2 * EIGHT;

// A download ask fits the header's `u32` length.
const _: () = assert!(GET_MOST <= u32::MAX as usize);
