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
            let note = match said {
                CloudAnswer::Text(said) => text(index, said.expose()),
                CloudAnswer::Fault(f) => fault(net_fault(f)),
            };
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
}

#[test]
fn the_worker_adapter_keeps_the_ports_shared_promises() {
    let (host, script, _) = host();
    check(&Played { host, script });
}
