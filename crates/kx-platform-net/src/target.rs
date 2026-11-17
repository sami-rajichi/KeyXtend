//! A checked HTTPS address: a plain lower-case host, a port and the request path. The host and path go to the OS
//! apart, so no other reading of the raw text can reach another host.

use crate::NetError;
use kx_ipc::{HTTPS, MAX_HOST_BYTES, MAX_URL_BYTES, rules};
use std::borrow::Cow;
use std::fmt;

/// HTTPS's own port, used when an address names none.
const PORT: u16 = 443;
/// Ends the scheme and starts a scheme-relative address.
const SLASHES: &str = "//";
/// Ends a scheme.
const COLON: char = ':';
/// Starts a path, and ends each folder in it.
const SLASH: char = '/';
/// Starts a query.
const QUERY: char = '?';
/// Starts a fragment, which is never sent.
const FRAGMENT: char = '#';
/// A folder that names the folder it is in.
const DOT: &str = ".";
/// A folder that names the folder above.
const DOTS: &str = "..";
/// What ends an address's host and port.
const AUTH_END: [char; 3] = [SLASH, QUERY, FRAGMENT];
/// Marks a user name or an IPv6 literal, neither of which a download address may hold.
const NOT_IN_HOST: [char; 3] = ['@', '[', ']'];
/// The bytes a scheme may hold after its first letter.
const SCHEME_MARKS: &[u8] = b"+-.";

/// A checked HTTPS address, at most `MAX_URL_BYTES` written out; only `parse` and `join` make one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    /// Lower-case letters, digits, dots and hyphens.
    host: String,
    /// The port, not 0.
    port: u16,
    /// The path and query, starting with `/`, without a fragment or `.` and `..` folders.
    path: String,
}

impl Target {
    /// Reads `url`, which must be visible ASCII starting with `kx_ipc::HTTPS` in any case.
    ///
    /// # Errors
    /// `NotHttps` for any other scheme, a user name, an IPv6 literal, a bad host or port, or past `MAX_URL_BYTES`.
    pub fn parse(url: &str) -> Result<Self, NetError> {
        let rest = plain(url).then(|| strip_https(url)).flatten();
        let rest = rest.ok_or(NetError::NotHttps)?;
        let end = rest.find(AUTH_END).unwrap_or(rest.len());
        let (auth, tail) = rest.split_at(end);
        let (host, port) = authority(auth)?;
        Self::checked(host, port, &with_slash(no_fragment(tail)))
    }

    /// The address a redirect's `Location` names, read against this one as RFC 3986 §5.2 says.
    ///
    /// # Errors
    /// `NotHttps` when it names another scheme, is empty or not a plain address, or joins past `MAX_URL_BYTES`.
    pub fn join(&self, to: &str) -> Result<Self, NetError> {
        if to.is_empty() || !plain(to) {
            return Err(NetError::NotHttps);
        }
        if let Some(rest) = to.strip_prefix(SLASHES) {
            return Self::parse(&format!("{HTTPS}{rest}"));
        }
        if has_scheme(to) {
            return Self::parse(to);
        }
        let to = no_fragment(to);
        let path = match to.chars().next() {
            None => self.path.clone(),
            Some(SLASH) => to.to_owned(),
            Some(QUERY) => format!("{}{to}", self.file()),
            _ => format!("{}{to}", self.folder()),
        };
        Self::checked(self.host.clone(), self.port, &path)
    }

    /// A target of these parts, its `.` and `..` folders resolved.
    ///
    /// # Errors
    /// `NotHttps` when, written out, it passes `MAX_URL_BYTES`.
    fn checked(host: String, port: u16, path: &str) -> Result<Self, NetError> {
        let path = tidy(path);
        let made = Self { host, port, path };
        if made.to_string().len() <= MAX_URL_BYTES {
            Ok(made)
        } else {
            Err(NetError::NotHttps)
        }
    }

    /// Refuses this redirect unless `hosts` takes its host: by name, or under a `rules::UNDER` entry's domain.
    ///
    /// # Errors
    /// `Unlisted` when no listed host takes this one.
    pub fn listed(&self, hosts: &[Cow<'_, str>]) -> Result<(), NetError> {
        self.any(hosts, rules::covers)
    }

    /// Refuses this first address unless a plain entry of `hosts` names its host (`rules::names`).
    ///
    /// # Errors
    /// `Unlisted` when no plain entry is this host.
    pub fn named(&self, hosts: &[Cow<'_, str>]) -> Result<(), NetError> {
        self.any(hosts, rules::names)
    }

    /// `Ok` when `takes` holds for an entry of `hosts` and this host, else `Unlisted`.
    fn any(&self, hosts: &[Cow<'_, str>], takes: fn(&str, &str) -> bool) -> Result<(), NetError> {
        if hosts.iter().any(|h| takes(h, &self.host)) {
            Ok(())
        } else {
            Err(NetError::Unlisted)
        }
    }

    /// The host, in lower case.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The port.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The path and query, starting with `/`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The path without its query.
    fn file(&self) -> &str {
        self.path.split(QUERY).next().unwrap_or_default()
    }

    /// The path up to and with its last `/`.
    fn folder(&self) -> &str {
        let file = self.file();
        file.rfind(SLASH).map_or(file, |at| &file[..=at])
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{HTTPS}{}", self.host)?;
        if self.port != PORT {
            write!(f, "{COLON}{}", self.port)?;
        }
        f.write_str(&self.path)
    }
}

/// True when `text` is visible ASCII within `MAX_URL_BYTES`.
fn plain(text: &str) -> bool {
    text.len() <= MAX_URL_BYTES && text.bytes().all(|b| b.is_ascii_graphic())
}

/// What follows `HTTPS` at the start of `url`, matched in any case.
fn strip_https(url: &str) -> Option<&str> {
    let n = HTTPS.len();
    let head = url.get(..n)?;
    head.eq_ignore_ascii_case(HTTPS).then(|| &url[n..])
}

/// True when `to` starts with a scheme: a letter, then letters, digits, `+`, `-` or `.`, then `:`.
fn has_scheme(to: &str) -> bool {
    let Some(end) = to.find([COLON, SLASH, QUERY, FRAGMENT]) else {
        return false;
    };
    let scheme = &to.as_bytes()[..end];
    let ok = |b: &u8| b.is_ascii_alphanumeric() || SCHEME_MARKS.contains(b);
    to[end..].starts_with(COLON)
        && scheme.first().is_some_and(u8::is_ascii_alphabetic)
        && scheme.iter().all(ok)
}

/// The host and port in `auth`, refusing a user name, an IPv6 literal, a bad host or a bad port.
fn authority(auth: &str) -> Result<(String, u16), NetError> {
    if auth.contains(NOT_IN_HOST) {
        return Err(NetError::NotHttps);
    }
    let (host, port) = match auth.rsplit_once(COLON) {
        Some((host, port)) => (host, port_of(port)?),
        None => (auth, PORT),
    };
    let host = host.to_ascii_lowercase();
    let fits = !host.is_empty() && host.len() <= MAX_HOST_BYTES;
    if fits && rules::host(&host).is_ok() {
        Ok((host, port))
    } else {
        Err(NetError::NotHttps)
    }
}

/// `text` as a port: digits only, from 1 to 65535.
fn port_of(text: &str) -> Result<u16, NetError> {
    let digits = !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit());
    let port = digits.then(|| text.parse::<u16>().ok()).flatten();
    port.filter(|p| *p != 0).ok_or(NetError::NotHttps)
}

/// `text` without its fragment.
fn no_fragment(text: &str) -> &str {
    text.split(FRAGMENT).next().unwrap_or_default()
}

/// `tail` as a request path: it starts with `/`.
fn with_slash(tail: &str) -> String {
    if tail.starts_with(SLASH) {
        tail.to_owned()
    } else {
        format!("{SLASH}{tail}")
    }
}

/// `path`, which starts with `/`, with its `.` and `..` folders resolved (RFC 3986 §5.2.4); the query is kept.
fn tidy(path: &str) -> String {
    let (file, query) = path.split_at(path.find(QUERY).unwrap_or(path.len()));
    let mut kept: Vec<&str> = Vec::new();
    let mut parts = file.split(SLASH).skip(1).peekable();
    while let Some(part) = parts.next() {
        let dots = part == DOT || part == DOTS;
        if part == DOTS {
            kept.pop();
        } else if !dots {
            kept.push(part);
        }
        // A last `.` or `..` names a folder, so the path keeps its closing `/`.
        if dots && parts.peek().is_none() {
            kept.push("");
        }
    }
    let mut out = String::with_capacity(path.len());
    for part in kept {
        out.push(SLASH);
        out.push_str(part);
    }
    if out.is_empty() {
        out.push(SLASH);
    }
    out + query
}
