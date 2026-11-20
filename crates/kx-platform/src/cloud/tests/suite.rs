//! The port's shared behaviour suite over the worker's adapter, the scripted worker playing the service.

use super::key::answer;
use super::*;
use kx_platform_fake::CloudAnswer;
use kx_platform_fake::cloud_suite::{Served, Tell, check};

/// The adapter and the worker the suite plays.
struct Played {
    host: Host,
    script: Script<NetFrames>,
}

/// The job's fault for `fault`, one of those the suite plays.
fn net_fault(fault: CloudFault) -> NetFault {
    match fault {
        CloudFault::Key => NetFault::Key,
        CloudFault::Network => NetFault::Network,
        other => panic!("the suite plays no {other:?}"),
    }
}

/// The job's note for piece `index`'s answer.
fn note(index: u16, said: CloudAnswer) -> NetNote {
    match said {
        CloudAnswer::Text(said) => text(index, said.expose()),
        CloudAnswer::Fault(f) => fault(net_fault(f)),
    }
}

impl Served for Played {
    fn speech(&self) -> &dyn CloudSpeech {
        &self.host
    }

    fn run(
        &self,
        ask: CloudAsk,
        answers: Vec<CloudAnswer>,
        tell: Tell,
    ) -> Result<Box<dyn CloudRun>, CloudFault> {
        let run = self.host.send(ask, tell)?;
        let mut ended = false;
        for (index, said) in (0..).zip(answers) {
            let note = note(index, said);
            ended |= matches!(note, NetNote::Fault { .. });
            self.script.say(Ok(note));
        }
        if !ended {
            self.script.say(Ok(NetNote::Done { id: ID }));
        }
        Ok(run)
    }

    fn test(&self, ask: CloudTest, said: Result<(), CloudFault>) -> Result<(), CloudFault> {
        let note = said.map_or_else(|f| fault(net_fault(f)), |()| NetNote::Done { id: ID });
        let asked = ask;
        let host = &self.host;
        let (calls, called) = mpsc::channel();
        self.script.tell(calls);
        std::thread::scope(|s| {
            let back = s.spawn(move || host.test(asked));
            answer(&self.script, note)(&called);
            back.join().expect("the test returns")
        })
    }

    fn drop_held(&self, ask: CloudAsk, answers: Vec<CloudAnswer>, tell: Tell) -> bool {
        let (calls, called) = mpsc::channel();
        self.script.tell(calls);
        let Ok(run) = self.host.send(ask, tell) else {
            return false;
        };
        drop(run);
        for (index, said) in (0..).zip(answers) {
            self.script.say(Ok(note(index, said)));
        }
        let cancel = Call::Net(NetAsk::Cancel { id: ID });
        std::iter::from_fn(|| called.recv_timeout(LONG).ok()).any(|call| call == cancel)
    }
}

#[test]
fn the_worker_adapter_keeps_the_ports_shared_promises() {
    let (host, script, _) = host();
    check(&Played { host, script });
}
