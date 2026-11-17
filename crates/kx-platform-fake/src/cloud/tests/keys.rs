//! A key's test: answered as scripted in turn, then `Ok`; checked by its own rule; held while told; and the key the
//! service takes, which refuses another in a run or a test.

use super::*;
use std::thread;

#[test]
fn a_key_test_answers_as_scripted_in_turn_then_ok() {
    let fake = FakeCloud::new();
    fake.script_test(Err(CloudFault::Key));
    fake.script_test(Err(CloudFault::Network));
    let answers: Vec<_> = (0..3).map(|_| fake.test(key_test())).collect();
    let want = [Err(CloudFault::Key), Err(CloudFault::Network), Ok(())];
    assert_eq!(answers, want);
    let tests = fake.calls().tests;
    let told: Vec<_> = tests.iter().map(|t| t.answer).collect();
    assert_eq!(told, want.map(Some));
    let (asked, test) = (&tests[0], key_test());
    assert_eq!(
        (&asked.url, &asked.header, &asked.shape),
        (&test.url, &test.header, &test.shape)
    );
    assert_eq!((asked.job, asked.wait_s), (test.job, test.wait_s));
    assert!(fake.calls().runs.is_empty(), "a test sends no sound");
}

#[test]
fn a_key_test_is_checked_by_its_own_rule() {
    let fake = FakeCloud::new();
    let odd = CloudTest {
        key: secret("two words"),
        ..key_test()
    };
    assert_eq!(fake.test(odd), Err(CloudFault::Range("key")));
    assert!(fake.calls().tests.is_empty());
}

#[test]
fn a_slow_key_test_returns_only_when_released() {
    let fake = FakeCloud::new();
    fake.script_test(Err(CloudFault::Timeout));
    fake.hold_tests(true);
    let port: Arc<dyn CloudSpeech> = Arc::new(fake.clone());
    let call = thread::spawn(move || port.test(key_test()));
    assert!(fake.wait(WITHIN, |calls| calls.tests.len() == 1));
    assert_eq!(fake.calls().tests[0].answer, None, "still held");
    fake.hold_tests(false);
    assert_eq!(call.join().unwrap(), Err(CloudFault::Timeout));
    assert_eq!(fake.calls().tests[0].answer, Some(Err(CloudFault::Timeout)));
}

#[test]
fn the_service_refuses_a_key_other_than_its_own() {
    let fake = FakeCloud::new();
    fake.take_key(Some(secret(KEY)));
    fake.script(vec![text(SAID)]);
    let other = CloudAsk {
        key: secret("gsk_other-1"),
        ..ask(&[2])
    };
    let (heard, tell) = hearer();
    let _refused = fake.send(other, tell).unwrap();
    assert_eq!(notes(&heard), [CloudNote::Fault(CloudFault::Key)]);
    fake.script(vec![text(SAID)]);
    let (_run, heard) = sent(&fake, &[2]);
    assert_eq!(notes(&heard), [piece(0, SAID), CloudNote::Done]);
    let wrong = CloudTest {
        key: secret("gsk_other-1"),
        ..key_test()
    };
    fake.script_test(Err(CloudFault::Rate));
    assert_eq!(fake.test(wrong), Err(CloudFault::Key));
    assert_eq!(fake.test(key_test()), Err(CloudFault::Rate), "it waited");
    fake.take_key(None);
    let any = CloudTest {
        key: secret("gsk_other-1"),
        ..key_test()
    };
    assert_eq!(fake.test(any), Ok(()));
    let shown = format!("{fake:?} {:?}", fake.calls());
    assert!(
        !shown.contains(KEY) && !shown.contains("gsk_other"),
        "{shown}"
    );
}
