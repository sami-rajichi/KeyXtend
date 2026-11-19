//! The speech-net frames built from the port's asks, the key and the sound moved into wiped buffers with no copy where
//! they fit one frame, and the job's faults and losses as the port's.

use super::SHAPE;
use crate::worker::Lost;
use kx_ipc::{
    MAX_SOUND_BYTES, NetAsk, NetCall, NetFault, NetField, NetForm, NetHeader, NetLimits, NetRun,
    NetShape, Wiped,
};
use kx_module_api::ports::{CloudFault, CloudLimits, CloudShape, FormFields, HttpsUrl, KeyHeader};
use kx_module_api::{Redacted, Secret, SecretBytes, Zeroizing};
use std::borrow::Cow;
use std::iter;
use std::time::Duration;

/// The value `Range` names for a piece no frame can size; `CloudAsk::check` refuses it first.
const PIECES: &str = "pieces";

/// The call to `url` in `shape`, its key under `header` moved into a wiped buffer.
pub(super) fn call(
    shape: CloudShape,
    url: &HttpsUrl,
    header: KeyHeader,
    key: Secret,
    wait_s: u16,
) -> NetCall<'static> {
    let mut key = key.into_inner();
    let key = Zeroizing::new(std::mem::take(&mut *key).into_bytes());
    NetCall {
        shape: shape_of(shape),
        url: Cow::Owned(url.expose().to_owned()),
        header: match header {
            KeyHeader::Bearer => NetHeader::Bearer,
            KeyHeader::Named(name) => NetHeader::Named(Cow::Owned(name)),
        },
        key: Wiped::Kept(key),
        wait_s,
    }
}

/// The frames' shape for the row's `shape`.
fn shape_of(shape: CloudShape) -> NetShape<'static> {
    match shape {
        CloudShape::Form(form) => NetShape::Form(form_of(form)),
        CloudShape::Eleven(form) => NetShape::Eleven(form_of(form)),
        CloudShape::Gemini => NetShape::Gemini,
    }
}

/// The frames' form for the row's `form`.
fn form_of(form: FormFields) -> NetForm<'static> {
    NetForm {
        lang: Cow::Owned(form.lang),
        list: form.list,
        fixed: (form.fixed.into_iter())
            .map(|f| NetField {
                name: Cow::Owned(f.name),
                value: Cow::Owned(f.value),
            })
            .collect(),
        paywall: form.paywall,
        region: form.region.map(Cow::Owned),
    }
}

/// A run of `model` in `lang` within `limits`, with `tries` and each piece's size.
///
/// # Errors
/// `Range` naming the pieces when a size passes the frames' width.
pub(super) fn run(
    model: Option<Redacted<String>>,
    lang: String,
    limits: CloudLimits,
    tries: u8,
    sizes: &[usize],
) -> Result<NetRun<'static>, CloudFault> {
    let sizes = sizes.iter().map(|&n| u32::try_from(n));
    let sizes = sizes.collect::<Result<_, _>>();
    let CloudLimits {
        max_mb,
        max_s,
        per_min,
        at_once,
    } = limits;
    Ok(NetRun {
        model: model.map(|m| Cow::Owned(m.into_inner())),
        lang: Cow::Owned(lang),
        limits: NetLimits {
            max_mb,
            max_s,
            per_min,
            at_once,
        },
        tries,
        sizes: sizes.map_err(|_| CloudFault::Range(PIECES))?,
    })
}

/// Each piece's sound as run `id`'s `Sound` parts, in order and at most `MAX_SOUND_BYTES` each: a piece that fits one
/// part moves into it, and a longer one is copied a part at a time, then wiped.
pub(super) fn parts(id: u32, pieces: Vec<SecretBytes>) -> impl Iterator<Item = NetAsk<'static>> {
    (0..)
        .zip(pieces)
        .flat_map(move |(piece, sound)| split(id, piece, sound.into_inner()))
}

/// Piece `piece`'s `sound` as parts of run `id`.
fn split(
    id: u32,
    piece: u16,
    sound: Zeroizing<Vec<u8>>,
) -> Box<dyn Iterator<Item = NetAsk<'static>>> {
    let part = move |bytes| NetAsk::Sound {
        id,
        piece,
        bytes: Wiped::Kept(bytes),
    };
    if sound.len() <= MAX_SOUND_BYTES {
        return Box::new(iter::once(part(sound)));
    }
    let count = sound.len().div_ceil(MAX_SOUND_BYTES);
    Box::new((0..count).map(move |k| {
        let start = k * MAX_SOUND_BYTES;
        let end = (start + MAX_SOUND_BYTES).min(sound.len());
        part(Zeroizing::new(
            sound.get(start..end).unwrap_or_default().to_vec(),
        ))
    }))
}

/// The port's fault for the job's; the job's own `Range` names the row's shape, which refused a value.
pub(super) fn fault(fault: NetFault) -> CloudFault {
    match fault {
        NetFault::Key => CloudFault::Key,
        NetFault::Credit => CloudFault::Credit,
        NetFault::Allowance { renews_s } => CloudFault::Allowance {
            renews_in: renews_s.map(|s| Duration::from_secs(s.into())),
        },
        NetFault::Rate => CloudFault::Rate,
        NetFault::TooBig => CloudFault::TooBig,
        NetFault::Language => CloudFault::Language,
        NetFault::Region => CloudFault::Region,
        NetFault::Empty => CloudFault::Empty,
        NetFault::Strange => CloudFault::Strange,
        NetFault::Network => CloudFault::Network,
        NetFault::Timeout => CloudFault::Timeout,
        NetFault::Range => CloudFault::Range(SHAPE),
        NetFault::Gone => CloudFault::Gone,
    }
}

/// The port's fault for a job that was lost: `Timeout` when it was late, else `Gone`.
pub(super) const fn lost(lost: Lost) -> CloudFault {
    match lost {
        Lost::Late => CloudFault::Timeout,
        Lost::Gone | Lost::Down { .. } => CloudFault::Gone,
    }
}
