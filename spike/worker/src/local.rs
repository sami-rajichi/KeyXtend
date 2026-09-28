//! whisper.cpp's server on this PC: started once in a job, so the model loads once and the server dies with us.

use std::net::{IpAddr, SocketAddr, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use spike_core::voicecfg::LocalConfig;
use spike_core::weak::{Caps, Job};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::engines::{self, Target};
use crate::http::{self, HTTP_OK, Request};

/// Server switches (`whisper-server --help`).
const MODEL_FLAG: &str = "-m";
/// Listen address switch.
const HOST_FLAG: &str = "--host";
/// Listen port switch.
const PORT_FLAG: &str = "--port";
/// Thread count switch.
const THREADS_FLAG: &str = "-t";

/// What one readiness check means.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    /// The model is loaded.
    Ready,
    /// The server stopped before it was ready.
    Stopped(ExitStatus),
    /// The wait is over and it is still not ready.
    Late,
    /// Check again after a pause.
    Wait,
}

/// The running local server.
pub struct Local {
    cfg: LocalConfig,
    child: Child,
    job: Job,
}

impl Local {
    /// Starts the server, capped by `caps` in weak mode, and waits until its model is loaded.
    pub fn start(cfg: &LocalConfig, caps: Option<&Caps>) -> Result<Local, String> {
        let (addr, poll) = (addr(cfg)?, Duration::from_millis(cfg.ready_poll_ms));
        if !Path::new(&cfg.model).is_file() {
            return Err(format!("model not found: {}", cfg.model));
        }
        if TcpStream::connect_timeout(&addr, poll).is_ok() {
            return Err(format!("port {} is already in use", cfg.port));
        }
        let job = Job::new(caps)?;
        let threads = caps.map_or(cfg.threads, |c| cfg.threads.min(c.affinity.count_ones()));
        let (port, threads) = (cfg.port.to_string(), threads.to_string());
        let args = [
            MODEL_FLAG,
            &cfg.model,
            HOST_FLAG,
            &cfg.host,
            PORT_FLAG,
            &port,
            THREADS_FLAG,
            &threads,
        ];
        let child = Command::new(&cfg.server)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()
            .map_err(|e| format!("cannot start {}: {e}", cfg.server))?;
        let mut local = Local {
            cfg: cfg.clone(),
            child,
            job,
        };
        local.job.assign(&local.child)?;
        local.wait_ready(poll)?;
        Ok(local)
    }

    /// Polls until the server says it is ready, stops, or takes too long.
    fn wait_ready(&mut self, poll: Duration) -> Result<(), String> {
        let wait_ms = self.cfg.ready_wait_ms;
        let until = Instant::now() + Duration::from_millis(wait_ms);
        loop {
            let exit = self.child.try_wait().ok().flatten();
            let ready = exit.is_none() && is_ready(&self.cfg);
            match step(exit, ready, Instant::now(), until) {
                Step::Ready => return Ok(()),
                Step::Stopped(code) => {
                    return Err(format!("the local server stopped at start ({code})"));
                }
                Step::Late => {
                    return Err(format!("the local server was not ready after {wait_ms} ms"));
                }
                Step::Wait => std::thread::sleep(poll),
            }
        }
    }

    /// The words in `wav`, spoken in `lang`; the server always gets it, `auto` included.
    pub fn text(&self, wav: &[u8], lang: &str) -> Result<String, String> {
        let t = Target {
            host: &self.cfg.host,
            port: self.cfg.port,
            secure: false,
            path: &self.cfg.path,
            timeout_ms: self.cfg.timeout_ms,
            key: None,
        };
        engines::ask(&t, wav, Some(lang), &[])
    }
}

impl Drop for Local {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The server's address: a loopback IP, not a name, so the server cannot resolve it to another address.
fn addr(cfg: &LocalConfig) -> Result<SocketAddr, String> {
    match cfg.host.parse::<IpAddr>() {
        Ok(ip) if ip.is_loopback() => Ok(SocketAddr::new(ip, cfg.port)),
        _ => Err(format!(
            "{}: the local server must listen on a loopback IP, on this PC only",
            cfg.host
        )),
    }
}

/// The step after one check: a stop comes first, then readiness, then the deadline.
fn step(exit: Option<ExitStatus>, ready: bool, now: Instant, until: Instant) -> Step {
    match exit {
        Some(code) => Step::Stopped(code),
        None if ready => Step::Ready,
        None if now >= until => Step::Late,
        None => Step::Wait,
    }
}

/// The server answers OK on its health path, which it does once the model is loaded.
fn is_ready(cfg: &LocalConfig) -> bool {
    let r = Request {
        host: &cfg.host,
        port: cfg.port,
        secure: false,
        path: &cfg.health_path,
        content_type: None,
        bearer: None,
        body: &[],
        timeout_ms: cfg.ready_poll_ms,
    };
    http::get(&r).is_ok_and(|a| a.status == HTTP_OK)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::os::windows::process::ExitStatusExt;

    /// Settings pointing at a free port, with a file that exists as the model.
    fn cfg() -> LocalConfig {
        let mut c = spike_core::config::load()
            .expect("spike.toml loads")
            .voice
            .local;
        c.model = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml").to_string();
        c.port = TcpListener::bind("127.0.0.1:0")
            .expect("binds")
            .local_addr()
            .expect("addr")
            .port();
        c.ready_poll_ms = 1000;
        c
    }

    /// Answers one request with `status` and gives back the settings that point at it.
    fn health_once(status: &'static str) -> (LocalConfig, std::thread::JoinHandle<()>) {
        let l = TcpListener::bind("127.0.0.1:0").expect("binds");
        let mut c = cfg();
        c.port = l.local_addr().expect("addr").port();
        let t = std::thread::spawn(move || {
            let (mut s, _) = l.accept().expect("a client");
            let (mut got, mut buf) = (Vec::new(), [0u8; 1024]);
            while !got.ends_with(b"\r\n\r\n") {
                let n = s.read(&mut buf).expect("reads");
                assert!(n > 0, "client closed");
                got.extend_from_slice(&buf[..n]);
            }
            let reply =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            s.write_all(reply.as_bytes()).expect("answers");
        });
        (c, t)
    }

    #[test]
    fn ready_means_the_health_path_answers_ok() {
        let (c, t) = health_once("200 OK");
        assert!(is_ready(&c));
        t.join().expect("served");
        let (c, t) = health_once("503 Service Unavailable");
        assert!(!is_ready(&c), "still loading the model");
        t.join().expect("served");
    }

    #[test]
    fn only_this_pc_may_host_the_server() {
        let mut c = cfg();
        c.host = "0.0.0.0".into();
        assert!(Local::start(&c, None).is_err_and(|e| e.contains("this PC")));
    }

    #[test]
    fn the_host_must_be_a_loopback_ip_not_a_name() {
        let mut c = cfg();
        for ok in ["127.0.0.1", "::1"] {
            c.host = ok.into();
            assert!(addr(&c).is_ok_and(|a| a.ip().is_loopback()), "{ok}");
        }
        for bad in ["localhost", "0.0.0.0", "192.168.1.2", ""] {
            c.host = bad.into();
            assert!(addr(&c).is_err_and(|e| e.contains("this PC")), "{bad}");
        }
    }

    #[test]
    fn a_stopped_server_fails_first_and_a_ready_one_beats_the_deadline() {
        let (now, stop) = (Instant::now(), ExitStatus::from_raw(1));
        let later = now + Duration::from_millis(1);
        assert_eq!(step(Some(stop), true, now, later), Step::Stopped(stop));
        assert_eq!(step(None, true, later, now), Step::Ready);
        assert_eq!(step(None, false, later, now), Step::Late);
        assert_eq!(
            step(None, false, now, now),
            Step::Late,
            "the deadline itself"
        );
        assert_eq!(step(None, false, now, later), Step::Wait);
    }

    #[test]
    fn a_missing_model_is_named() {
        let mut c = cfg();
        c.model = "D:/no/such/model.bin".into();
        assert!(Local::start(&c, None).is_err_and(|e| e.contains("model not found")));
    }

    #[test]
    fn a_missing_server_is_named() {
        let mut c = cfg();
        c.server = "D:/no/such/server.exe".into();
        assert!(Local::start(&c, None).is_err_and(|e| e.contains("cannot start")));
    }

    #[test]
    fn a_busy_port_is_refused() {
        let l = TcpListener::bind("127.0.0.1:0").expect("binds");
        let mut c = cfg();
        c.port = l.local_addr().expect("addr").port();
        assert!(Local::start(&c, None).is_err_and(|e| e.contains("in use")));
    }
}
