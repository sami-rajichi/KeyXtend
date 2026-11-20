//! "Use the local model" keeps a take on this PC for good: its Try again writes with the local model again, sends
//! nothing and keeps the recording.

use kx_module_api::ports::PanelButton;

use super::*;

impl Rig {
    /// The newest run writes `text` in one piece, and the keyboard types it.
    fn writes(&mut self, text: &str) {
        self.tell(SpeechNote::Piece {
            index: 0,
            text: secret(text),
        });
        self.tell(SpeechNote::Done(done(1)));
        self.typed();
    }
}

#[test]
fn after_the_local_model_wrote_it_try_again_types_with_it_and_sends_nothing() {
    let mut rig = Rig::build(groq);
    let fault = CloudAnswer::Fault(CloudFault::Credit);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("half"), fault]);
    rig.dictate();
    rig.act(VoiceClick::Local);
    rig.writes("local");
    rig.act(VoiceClick::Button(PanelButton::Again));
    rig.writes("again");
    assert_eq!(rig.typed_texts(), ["local", "again"]);
    assert_eq!(
        rig.fake.cloud.calls().runs.len(),
        1,
        "only the first upload"
    );
    let run = rig.fake.speech.runs()[0].clone();
    let local: Vec<bool> = run.agains.iter().map(|a| a.model.is_some()).collect();
    assert_eq!(local, [true, true], "the local model wrote both");
    assert!(run.held && !run.closed, "the recording stays");
}

#[test]
fn a_pick_moved_to_the_local_model_while_the_service_stays_on_says_it_moved_not_off() {
    let mut rig = Rig::build(groq);
    rig.talk();
    rig.actor.dos(vec![crate::panel::Do::Set(
        crate::settings::PICKS,
        table(""),
    )]);
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Moved));
    assert!(rig.fake.cloud.calls().runs.is_empty());
}
