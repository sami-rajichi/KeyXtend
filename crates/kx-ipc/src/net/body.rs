//! The bodies the net asks send: a call (its shape, address, key header, key and wait), a run (its model, language,
//! limits, tries and piece sizes) and a part of a piece's sound. Each field's length is checked first, then its content
//! and range.

use super::code::{HeaderCode, ShapeCode};
use super::wire::{FIXED, KEY, SIZES, SOUND, TAG, URL, VALUE};
use super::{NetCall, NetField, NetForm, NetHeader, NetLimits, NetOwn, NetRun, NetShape};
use crate::code::Code;
use crate::error::IpcError;
use crate::limits::{
    MAX_RECORD_BYTES, MAX_SPEECH_PIECES, NET_AT_ONCE, NET_MAX_MB, NET_MAX_S, NET_PER_MIN,
    NET_TRIES, NET_WAIT_S, SAMPLE_BYTES,
};
use crate::rules::{self, ok_if, within};
use crate::voice::body::sound_index;
use crate::wiped::Wiped;
use crate::wire::{Payload, Sink, Spec};
use std::borrow::Cow;
use std::io::Read;

/// A content rule a text must pass.
type Rule = fn(&str) -> Result<(), IpcError>;

/// Sends a text of `spec` that passes `rule`.
fn put_text(s: &mut dyn Sink, spec: Spec, text: &str, rule: Rule) -> Result<(), IpcError> {
    s.field(spec, text.as_bytes())?;
    rule(text)
}

/// Reads a text of `spec` that passes `rule`.
fn read_text<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
    spec: Spec,
    rule: Rule,
) -> Result<Cow<'static, str>, IpcError> {
    let text = p.text(spec)?;
    rule(&text)?;
    Ok(Cow::Owned(text))
}

/// Sends a shape: its code, then an upload form's language name, list flag, fixed fields, paywall flag and region
/// code, `ElevenLabs`' language name and fixed fields, or `AssemblyAI`'s model header name.
fn put_shape(s: &mut dyn Sink, shape: &NetShape<'_>) -> Result<(), IpcError> {
    s.byte(shape.code().code())?;
    match shape {
        NetShape::Form(form) => put_form(s, form),
        NetShape::Eleven(own) => {
            put_text(s, TAG, &own.lang, rules::token)?;
            put_fixed(s, &own.fixed)
        }
        NetShape::Gemini => Ok(()),
        NetShape::Assembly { model_header } => put_text(s, TAG, model_header, rules::token),
    }
}

/// Sends an upload form's language name, list flag, fixed fields, paywall flag and region code.
fn put_form(s: &mut dyn Sink, form: &NetForm<'_>) -> Result<(), IpcError> {
    put_text(s, TAG, &form.lang, rules::token)?;
    s.byte(u8::from(form.list))?;
    put_fixed(s, &form.fixed)?;
    s.byte(u8::from(form.paywall))?;
    s.byte(u8::from(form.region.is_some()))?;
    match &form.region {
        Some(code) => put_text(s, TAG, code, rules::token),
        None => Ok(()),
    }
}

/// Sends the fixed fields' count, then each name and value.
fn put_fixed(s: &mut dyn Sink, fixed: &[NetField<'_>]) -> Result<(), IpcError> {
    s.count(FIXED, fixed.len())?;
    fixed.iter().try_for_each(|f| {
        put_text(s, TAG, &f.name, rules::token)?;
        put_text(s, VALUE, &f.value, rules::form_value)
    })
}

/// Reads a shape.
fn read_shape<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<NetShape<'static>, IpcError> {
    Ok(
        match ShapeCode::from_code(p.byte()?).ok_or(IpcError::BadField)? {
            ShapeCode::Form => NetShape::Form(read_form(p)?),
            ShapeCode::Eleven => NetShape::Eleven(NetOwn {
                lang: read_text(p, TAG, rules::token)?,
                fixed: read_fixed(p)?,
            }),
            ShapeCode::Gemini => NetShape::Gemini,
            ShapeCode::Assembly => NetShape::Assembly {
                model_header: read_text(p, TAG, rules::token)?,
            },
        },
    )
}

/// Reads the fixed fields' count, then each name and value.
fn read_fixed<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<Vec<NetField<'static>>, IpcError> {
    let n = p.count(FIXED)?;
    let field = |p: &mut Payload<'_, R>| {
        let name = read_text(p, TAG, rules::token)?;
        let value = read_text(p, VALUE, rules::form_value)?;
        Ok::<_, IpcError>(NetField { name, value })
    };
    (0..n).map(|_| field(p)).collect()
}

/// Reads a form's language name, list flag, fixed fields, paywall flag and region code.
fn read_form<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<NetForm<'static>, IpcError> {
    let lang = read_text(p, TAG, rules::token)?;
    let list = p.flag()?;
    let fixed = read_fixed(p)?;
    let paywall = p.flag()?;
    let region = if p.flag()? {
        Some(read_text(p, TAG, rules::token)?)
    } else {
        None
    };
    Ok(NetForm {
        lang,
        list,
        fixed,
        paywall,
        region,
    })
}

/// Sends a key's header: its code, then a name.
fn put_header(s: &mut dyn Sink, header: &NetHeader<'_>) -> Result<(), IpcError> {
    match header {
        NetHeader::Bearer => s.byte(HeaderCode::Bearer.code()),
        NetHeader::Named(name) => {
            s.byte(HeaderCode::Named.code())?;
            put_text(s, TAG, name, rules::key_header)
        }
    }
}

/// Reads a key's header.
fn read_header<R: Read + ?Sized>(p: &mut Payload<'_, R>) -> Result<NetHeader<'static>, IpcError> {
    Ok(
        match HeaderCode::from_code(p.byte()?).ok_or(IpcError::BadField)? {
            HeaderCode::Bearer => NetHeader::Bearer,
            HeaderCode::Named => NetHeader::Named(read_text(p, TAG, rules::key_header)?),
        },
    )
}

/// Sends a call: its shape, address, key header, key and wait.
pub(super) fn put_call(s: &mut dyn Sink, call: &NetCall<'_>) -> Result<(), IpcError> {
    put_shape(s, &call.shape)?;
    put_text(s, URL, &call.url, rules::https)?;
    put_header(s, &call.header)?;
    s.field(KEY, &call.key)?;
    rules::key(&call.key)?;
    s.u16(within(call.wait_s, &NET_WAIT_S)?)
}

/// Reads a call, its key into a wiped buffer before its rule is checked.
pub(super) fn read_call<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<NetCall<'static>, IpcError> {
    let shape = read_shape(p)?;
    let url = read_text(p, URL, rules::https)?;
    let header = read_header(p)?;
    let n = p.count(KEY)?;
    let key = Wiped::Kept(p.wiped(n)?);
    rules::key(&key)?;
    let wait_s = within(p.u16()?, &NET_WAIT_S)?;
    Ok(NetCall {
        shape,
        url,
        header,
        key,
        wait_s,
    })
}

/// Checks a piece of `n` bytes against `limits`, adding it to `total`, which stays within the longest dictation.
fn size(limits: NetLimits, n: u32, total: &mut usize) -> Result<u32, IpcError> {
    let bytes = usize::try_from(n).map_err(|_| IpcError::BadField)?;
    *total = total.saturating_add(bytes);
    let whole = bytes > 0 && bytes.is_multiple_of(SAMPLE_BYTES);
    ok_if(whole && bytes <= limits.piece_bytes() && *total <= MAX_RECORD_BYTES)?;
    Ok(n)
}

/// Refuses a run with no model to a shape that needs one.
pub(super) fn named(call: &NetCall<'_>, run: &NetRun<'_>) -> Result<(), IpcError> {
    ok_if(run.model.is_some() || !call.shape.code().needs_model())
}

/// Sends a run: whether it names a model, the model, the language, the limits, the tries, then each piece's size.
pub(super) fn put_run(s: &mut dyn Sink, run: &NetRun<'_>) -> Result<(), IpcError> {
    s.byte(u8::from(run.model.is_some()))?;
    if let Some(model) = &run.model {
        put_text(s, TAG, model, rules::model)?;
    }
    put_text(s, TAG, &run.lang, rules::token)?;
    let l = run.limits;
    s.u32(within(l.max_mb, &NET_MAX_MB)?)?;
    s.u16(within(l.max_s, &NET_MAX_S)?)?;
    s.u16(within(l.per_min, &NET_PER_MIN)?)?;
    s.byte(within(l.at_once, &NET_AT_ONCE)?)?;
    s.byte(within(run.tries, &NET_TRIES)?)?;
    s.count(SIZES, run.sizes.len())?;
    let mut total = 0;
    run.sizes
        .iter()
        .try_for_each(|&n| s.u32(size(l, n, &mut total)?))
}

/// Reads a run.
pub(super) fn read_run<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<NetRun<'static>, IpcError> {
    let model = if p.flag()? {
        Some(read_text(p, TAG, rules::model)?)
    } else {
        None
    };
    let lang = read_text(p, TAG, rules::token)?;
    let limits = NetLimits {
        max_mb: within(p.u32()?, &NET_MAX_MB)?,
        max_s: within(p.u16()?, &NET_MAX_S)?,
        per_min: within(p.u16()?, &NET_PER_MIN)?,
        at_once: within(p.byte()?, &NET_AT_ONCE)?,
    };
    let tries = within(p.byte()?, &NET_TRIES)?;
    let (n, mut total) = (p.count(SIZES)?, 0);
    let sizes = (0..n)
        .map(|_| size(limits, p.u32()?, &mut total))
        .collect::<Result<_, _>>()?;
    Ok(NetRun {
        model,
        lang,
        limits,
        tries,
        sizes,
    })
}

/// `index` when it is below `MAX_SPEECH_PIECES`.
pub(super) fn piece_index(index: u16) -> Result<u16, IpcError> {
    sound_index(index, MAX_SPEECH_PIECES)
}

/// Sends a part of a piece's sound: the piece's index, then its whole samples.
pub(super) fn put_sound(s: &mut dyn Sink, piece: u16, bytes: &[u8]) -> Result<(), IpcError> {
    s.u16(piece_index(piece)?)?;
    s.samples(SOUND, bytes)
}

/// Reads a part of a piece's sound into a wiped buffer, refusing a part sample before its bytes are read.
pub(super) fn read_sound<R: Read + ?Sized>(
    p: &mut Payload<'_, R>,
) -> Result<(u16, Wiped<'static>), IpcError> {
    let piece = piece_index(p.u16()?)?;
    Ok((piece, Wiped::Kept(p.samples(SOUND)?)))
}
