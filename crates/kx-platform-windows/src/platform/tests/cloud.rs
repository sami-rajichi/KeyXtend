//! The cloud speech port, as the platform hands it out: it moves to another thread and refuses a bad ask or key test
//! by the value's name before any worker starts.

use super::*;
use kx_module_api::ports::{CloudAsk, CloudFault, CloudNote, CloudTest};
use kx_platform_fake::cloud_suite::{SECOND, ask, key_test, secret};

#[test]
fn the_cloud_speech_port_refuses_a_bad_ask_on_another_thread_before_any_worker() {
    let cloud = WindowsPlatform::new().unwrap().cloud_speech();
    // Never a valid ask or test: either starts the worker, which reaches the network.
    let refused = std::thread::spawn(move || {
        let empty = CloudAsk {
            pieces: Vec::new(),
            ..ask(&[SECOND])
        };
        let keyless = CloudTest {
            key: secret(""),
            ..key_test()
        };
        let sent = cloud.send(empty, Box::new(drop::<CloudNote>)).err();
        (sent, cloud.test(keyless))
    });
    let want = (
        Some(CloudFault::Range("pieces")),
        Err(CloudFault::Range("key")),
    );
    assert_eq!(refused.join().unwrap(), want);
}
