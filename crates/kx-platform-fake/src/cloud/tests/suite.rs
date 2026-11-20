//! The port's shared behaviour suite over the fake, which plays the service from its scripts.

use super::super::suite::{Served, Tell, check};
use super::*;
use kx_module_api::ports::CloudSpeech;

/// The fake as the suite drives it.
struct Scripted(FakeCloud);

impl Served for Scripted {
    fn speech(&self) -> &dyn CloudSpeech {
        &self.0
    }

    fn run(
        &self,
        ask: CloudAsk,
        answers: Vec<CloudAnswer>,
        tell: Tell,
    ) -> Result<Box<dyn CloudRun>, CloudFault> {
        self.0.script(answers);
        self.0.send(ask, tell)
    }

    fn test(&self, ask: CloudTest, answer: Result<(), CloudFault>) -> Result<(), CloudFault> {
        self.0.script_test(answer);
        self.0.test(ask)
    }

    fn drop_held(&self, ask: CloudAsk, answers: Vec<CloudAnswer>, tell: Tell) -> bool {
        let at = self.0.calls().runs.len();
        let Ok(run) = self.0.send(ask, tell) else {
            return false;
        };
        drop(run);
        let told = self.0.answer(at, answers);
        !told && self.0.calls().runs[at].state == CloudState::Dropped
    }
}

#[test]
fn the_fake_keeps_the_ports_shared_promises() {
    check(&Scripted(FakeCloud::new()));
}
