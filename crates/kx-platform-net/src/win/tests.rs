//! The adapter's error table, and requests to a local server on the loopback that speaks plain HTTP behind an HTTPS
//! address, with no proxy: each is refused, and the first byte sent is TLS's, never a plain GET or a POST's key.

use super::{WinHttp, codes, post};
use crate::{Call, Field, Hop, Http, NetError, Target, Verb};
use kx_ipc::STALL_S;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use windows::Win32::Networking::WinHttp::{
    ERROR_WINHTTP_CANNOT_CONNECT, ERROR_WINHTTP_INTERNAL_ERROR,
    ERROR_WINHTTP_INVALID_SERVER_RESPONSE, ERROR_WINHTTP_NAME_NOT_RESOLVED,
    ERROR_WINHTTP_SECURE_FAILURE, ERROR_WINHTTP_SECURE_INVALID_CA, ERROR_WINHTTP_TIMEOUT,
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ACCESS_TYPE_NO_PROXY,
};

/// TLS's first byte: a handshake record.
const HANDSHAKE: u8 = 0x16;
/// The plain HTTP answer the local server sends.
const PLAIN: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
/// The longest a server read waits, so a broken client fails the test instead of hanging it.
const READ_WAIT: Duration = Duration::from_secs(10);

#[test]
fn the_error_table_tells_stalls_connections_and_answers_apart() {
    assert_eq!(codes::kind(ERROR_WINHTTP_TIMEOUT), NetError::Stall);
    for code in [
        ERROR_WINHTTP_NAME_NOT_RESOLVED,
        ERROR_WINHTTP_CANNOT_CONNECT,
        ERROR_WINHTTP_SECURE_FAILURE,
        ERROR_WINHTTP_SECURE_INVALID_CA,
    ] {
        assert_eq!(codes::kind(code), NetError::Net, "{code}");
    }
    assert_eq!(
        codes::kind(ERROR_WINHTTP_INVALID_SERVER_RESPONSE),
        NetError::Server
    );
    assert_eq!(
        codes::kind(ERROR_WINHTTP_INTERNAL_ERROR),
        NetError::Internal
    );
}

#[test]
fn the_product_client_uses_the_system_proxy() {
    assert_eq!(WinHttp::new().access, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY);
    assert_eq!(
        WinHttp::default().access,
        WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY
    );
}

#[test]
fn a_plain_http_server_behind_an_https_address_is_refused() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let done = Arc::new(AtomicBool::new(false));
    let server = {
        let done = Arc::clone(&done);
        std::thread::spawn(move || serve(&listener, &done))
    };
    let target = Target::parse(&format!("https://{}:{port}/m.bin", Ipv4Addr::LOCALHOST)).unwrap();
    let hop = Hop {
        target: &target,
        from: 0,
        stall_s: *STALL_S.start(),
    };
    let direct = WinHttp {
        access: WINHTTP_ACCESS_TYPE_NO_PROXY,
    };
    let got = direct.send(&hop).err();
    done.store(true, Ordering::SeqCst);
    drop(TcpStream::connect((Ipv4Addr::LOCALHOST, port)));
    let first = server.join().unwrap();
    assert_eq!(got, Some(NetError::Net));
    assert_eq!(first, Some(HANDSHAKE), "the request starts with TLS");
}

#[test]
fn a_post_to_a_plain_http_server_never_sends_its_key_in_clear() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let done = Arc::new(AtomicBool::new(false));
    let server = {
        let done = Arc::clone(&done);
        std::thread::spawn(move || serve(&listener, &done))
    };
    let target = Target::parse(&format!("https://{}:{port}/v1", Ipv4Addr::LOCALHOST)).unwrap();
    let fields = [Field {
        name: "Authorization",
        value: b"Bearer kx-test-key-not-real",
    }];
    let call = Call {
        target: &target,
        verb: Verb::Post,
        fields: &fields,
        body: b"sound",
        keep: &[],
        stall_s: *STALL_S.start(),
        stop: &AtomicBool::new(false),
    };
    let direct = WinHttp {
        access: WINHTTP_ACCESS_TYPE_NO_PROXY,
    };
    let got = direct.call(&call).err();
    done.store(true, Ordering::SeqCst);
    drop(TcpStream::connect((Ipv4Addr::LOCALHOST, port)));
    let first = server.join().unwrap();
    assert_eq!(got, Some(NetError::Net));
    assert_eq!(first, Some(HANDSHAKE), "the request starts with TLS");
}

#[test]
fn the_header_block_takes_its_room_once() {
    let target = Target::parse("https://api.example/v1").unwrap();
    let fields = [
        Field {
            name: "Authorization",
            value: b"Bearer kx-test-key-not-real",
        },
        Field {
            name: "Content-Type",
            value: b"multipart/form-data; boundary=b",
        },
    ];
    let call = Call {
        target: &target,
        verb: Verb::Post,
        fields: &fields,
        body: b"",
        keep: &[],
        stall_s: *STALL_S.start(),
        stop: &AtomicBool::new(false),
    };
    let block = post::block(&call).unwrap();
    let want = "Authorization: Bearer kx-test-key-not-real\r\nContent-Type: multipart/form-data; boundary=b\r\n";
    assert_eq!(String::from_utf16(&block).unwrap(), want);
    assert_eq!(
        block.capacity(),
        block.len(),
        "no growth left a copy of the key"
    );
}

#[test]
fn deliver_refuses_a_head_that_would_break_before_it_connects() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let target = Target::parse(&format!("https://{}:{port}/v1", Ipv4Addr::LOCALHOST)).unwrap();
    let fields = [Field {
        name: "X-Key",
        value: b"a\r\nX-Evil: 1",
    }];
    let call = Call {
        target: &target,
        verb: Verb::Post,
        fields: &fields,
        body: b"",
        keep: &[],
        stall_s: *STALL_S.start(),
        stop: &AtomicBool::new(false),
    };
    let direct = WinHttp {
        access: WINHTTP_ACCESS_TYPE_NO_PROXY,
    };
    assert_eq!(direct.deliver(&call).err(), Some(NetError::Header));
}

/// Answers each connection with `PLAIN` until `done`; gives the first byte the first client sent.
fn serve(listener: &TcpListener, done: &AtomicBool) -> Option<u8> {
    let mut first = None;
    for stream in listener.incoming() {
        if done.load(Ordering::SeqCst) {
            break;
        }
        let Ok(mut s) = stream else { continue };
        let _ = s.set_read_timeout(Some(READ_WAIT));
        let mut byte = [0u8; 1];
        if s.read(&mut byte).is_ok_and(|n| n == 1) {
            first.get_or_insert(byte[0]);
        }
        let _ = s.write_all(PLAIN);
    }
    first
}
