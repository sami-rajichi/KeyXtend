//! `WinHTTP`'s error codes as `NetError`s: the adapter's table, so a stall, a failed connection and a bad answer differ.

use crate::NetError;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinHttp::{
    ERROR_WINHTTP_AUTO_PROXY_SERVICE_ERROR, ERROR_WINHTTP_AUTODETECTION_FAILED,
    ERROR_WINHTTP_BAD_AUTO_PROXY_SCRIPT, ERROR_WINHTTP_CANNOT_CONNECT,
    ERROR_WINHTTP_CHUNKED_ENCODING_HEADER_SIZE_OVERFLOW, ERROR_WINHTTP_CLIENT_AUTH_CERT_NEEDED,
    ERROR_WINHTTP_CONNECTION_ERROR, ERROR_WINHTTP_HEADER_COUNT_EXCEEDED,
    ERROR_WINHTTP_HEADER_SIZE_OVERFLOW, ERROR_WINHTTP_INVALID_HEADER,
    ERROR_WINHTTP_INVALID_SERVER_RESPONSE, ERROR_WINHTTP_LOGIN_FAILURE,
    ERROR_WINHTTP_NAME_NOT_RESOLVED, ERROR_WINHTTP_OPERATION_CANCELLED,
    ERROR_WINHTTP_RESEND_REQUEST, ERROR_WINHTTP_RESPONSE_DRAIN_OVERFLOW,
    ERROR_WINHTTP_SCRIPT_EXECUTION_ERROR, ERROR_WINHTTP_SECURE_CERT_CN_INVALID,
    ERROR_WINHTTP_SECURE_CERT_DATE_INVALID, ERROR_WINHTTP_SECURE_CERT_REV_FAILED,
    ERROR_WINHTTP_SECURE_CERT_REVOKED, ERROR_WINHTTP_SECURE_CERT_WRONG_USAGE,
    ERROR_WINHTTP_SECURE_CHANNEL_ERROR, ERROR_WINHTTP_SECURE_FAILURE,
    ERROR_WINHTTP_SECURE_FAILURE_PROXY, ERROR_WINHTTP_SECURE_INVALID_CA,
    ERROR_WINHTTP_SECURE_INVALID_CERT, ERROR_WINHTTP_TIMEOUT,
    ERROR_WINHTTP_UNABLE_TO_DOWNLOAD_SCRIPT,
};

/// Each code the rules tell apart; any other is `Internal`.
const CODES: &[(u32, NetError)] = &[
    // No byte within the stall time, while resolving, connecting, sending or reading.
    (ERROR_WINHTTP_TIMEOUT, NetError::Stall),
    // The name, the connection or the proxy failed.
    (ERROR_WINHTTP_NAME_NOT_RESOLVED, NetError::Net),
    (ERROR_WINHTTP_CANNOT_CONNECT, NetError::Net),
    (ERROR_WINHTTP_CONNECTION_ERROR, NetError::Net),
    (ERROR_WINHTTP_OPERATION_CANCELLED, NetError::Net),
    (ERROR_WINHTTP_RESEND_REQUEST, NetError::Net),
    (ERROR_WINHTTP_LOGIN_FAILURE, NetError::Net),
    (ERROR_WINHTTP_AUTO_PROXY_SERVICE_ERROR, NetError::Net),
    (ERROR_WINHTTP_AUTODETECTION_FAILED, NetError::Net),
    (ERROR_WINHTTP_BAD_AUTO_PROXY_SCRIPT, NetError::Net),
    (ERROR_WINHTTP_UNABLE_TO_DOWNLOAD_SCRIPT, NetError::Net),
    (ERROR_WINHTTP_SCRIPT_EXECUTION_ERROR, NetError::Net),
    // TLS failed: the handshake, or a certificate Windows does not trust for this host.
    (ERROR_WINHTTP_SECURE_FAILURE, NetError::Net),
    (ERROR_WINHTTP_SECURE_FAILURE_PROXY, NetError::Net),
    (ERROR_WINHTTP_SECURE_CHANNEL_ERROR, NetError::Net),
    (ERROR_WINHTTP_SECURE_INVALID_CA, NetError::Net),
    (ERROR_WINHTTP_SECURE_INVALID_CERT, NetError::Net),
    (ERROR_WINHTTP_SECURE_CERT_CN_INVALID, NetError::Net),
    (ERROR_WINHTTP_SECURE_CERT_DATE_INVALID, NetError::Net),
    (ERROR_WINHTTP_SECURE_CERT_REVOKED, NetError::Net),
    (ERROR_WINHTTP_SECURE_CERT_REV_FAILED, NetError::Net),
    (ERROR_WINHTTP_SECURE_CERT_WRONG_USAGE, NetError::Net),
    (ERROR_WINHTTP_CLIENT_AUTH_CERT_NEEDED, NetError::Net),
    // The server's answer could not be read.
    (ERROR_WINHTTP_INVALID_SERVER_RESPONSE, NetError::Server),
    (ERROR_WINHTTP_INVALID_HEADER, NetError::Server),
    (ERROR_WINHTTP_HEADER_SIZE_OVERFLOW, NetError::Server),
    (ERROR_WINHTTP_HEADER_COUNT_EXCEEDED, NetError::Server),
    (
        ERROR_WINHTTP_CHUNKED_ENCODING_HEADER_SIZE_OVERFLOW,
        NetError::Server,
    ),
    (ERROR_WINHTTP_RESPONSE_DRAIN_OVERFLOW, NetError::Server),
];

/// The error a Win32 `code` stands for.
pub(super) fn kind(code: u32) -> NetError {
    CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(NetError::Internal, |(_, kind)| *kind)
}

/// The error `e` stands for; one that holds no Win32 code is `Internal`.
pub(super) fn of(e: &windows::core::Error) -> NetError {
    WIN32_ERROR::from_error(e).map_or(NetError::Internal, |c| kind(c.0))
}

/// The error the thread's last Win32 code stands for.
pub(super) fn last() -> NetError {
    of(&windows::core::Error::from_thread())
}
