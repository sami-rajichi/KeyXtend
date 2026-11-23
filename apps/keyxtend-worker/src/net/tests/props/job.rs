//! Property: random host asks on a few ids, over a server that answers each request by what it carries, tell each run
//! or test one outcome on its own id, `Gone` at once for an ask on a live id or a part out of step, and nothing for a
//! part, run or test that is not live, even once its outcome was told.

mod fill;

use super::super::bench::{Bench, WAIT, fault, part, send};
use super::super::*;
use super::accepts;
use crate::net::Event;
use fill::{ByFill, FILLED, Kind, STRAY, holds};
use kx_ipc::{NetAsk, NetFault, NetNote};
use proptest::prelude::*;
use std::collections::HashMap;

/// The ids the asks use: few, so an id is often reused while live.
const IDS: u32 = 3;
/// A key test address the server refuses.
const REFUSED: &str = "https://stt.example.com/v1/refused";
/// Each run's starts a minute, as the job tests give.
const PER_MIN: u16 = 20;

/// A run's plan: its pieces' sizes, how they are answered, and its requests at once and tries each.
#[derive(Clone, Debug)]
struct Plan {
    sizes: Vec<u32>,
    kind: Kind,
    at_once: u8,
    tries: u8,
}

/// A part for an id: of a piece, else the run's current one, and of a size, else what that piece still needs.
type Piece = (u32, Option<u16>, Option<usize>);

/// One host ask, or the job taking a request's outcome.
#[derive(Clone, Debug)]
enum Op {
    /// A run on the id, as planned.
    Send(u32, Plan),
    /// A key test on the id, whose key works when true.
    Test(u32, bool),
    /// A part, as `Piece` says.
    Part(Piece),
    /// Cancel on the id.
    Cancel(u32),
    /// The job takes one outcome: waiting when one is due, else only one already told.
    Hand,
}

/// A random plan, its pieces of an even size of at least `FILLED`.
fn plan() -> impl Strategy<Value = Plan> {
    let sizes = prop::collection::vec((2u32..=6).prop_map(|n| n * 2), 1..4);
    let kind = prop::sample::select(Kind::ALL.to_vec());
    (sizes, kind, 1u8..=3, 1u8..=2).prop_map(|(sizes, kind, at_once, tries)| Plan {
        sizes,
        kind,
        at_once,
        tries,
    })
}

/// A random op, parts and outcomes most often so runs fill and end.
fn op() -> impl Strategy<Value = Op> {
    let id = || 0..IDS;
    let send = (id(), plan()).prop_map(|(id, plan)| Op::Send(id, plan));
    let test = (id(), any::<bool>()).prop_map(|(id, works)| Op::Test(id, works));
    let size = prop::option::of((1usize..=6).prop_map(|n| n * 2));
    let part = (id(), prop::option::of(0u16..4), size).prop_map(Op::Part);
    let cancel = id().prop_map(Op::Cancel);
    prop_oneof![2 => send, 1 => test, 4 => part, 1 => cancel, 3 => Just(Op::Hand)]
}

/// A live run as the model holds it: how its pieces are answered, their sizes, the parts' cursor and the texts told.
struct Track {
    kind: Kind,
    sizes: Vec<u32>,
    cursor: (usize, usize),
    told: usize,
}

impl Track {
    /// A run of `plan` with nothing taken or told.
    fn new(plan: Plan) -> Self {
        let (kind, sizes) = (plan.kind, plan.sizes);
        Self {
            kind,
            sizes,
            cursor: (0, 0),
            told: 0,
        }
    }

    /// What its current piece still needs; `FILLED` once every piece is full.
    fn rest(&self) -> usize {
        let (at, got) = self.cursor;
        self.sizes.get(at).map_or(FILLED, |&s| s as usize - got)
    }
}

/// A live run or key test as the model holds it.
enum Life {
    Run(Track),
    /// A key test, whose key works when true.
    Test(bool),
}

impl Life {
    /// True while one of its requests is out, so an outcome will come.
    fn due(&self) -> bool {
        match self {
            Self::Test(_) => true,
            Self::Run(run) => run.cursor.0 > run.told,
        }
    }

    /// The note that ends it.
    fn end(&self, id: u32) -> NetNote {
        match self {
            Self::Test(true) => NetNote::Done { id },
            Self::Test(false) => fault(id, NetFault::Key),
            Self::Run(run) => run.kind.end(id),
        }
    }
}

/// The bench, its server and the live runs and tests by id.
struct Model {
    b: Bench,
    server: Arc<ByFill>,
    lives: HashMap<u32, Life>,
}

impl Model {
    /// Takes `op` and checks the notes it told.
    fn apply(&mut self, op: Op) -> Result<(), TestCaseError> {
        let before = self.b.out.len();
        match op {
            Op::Send(id, plan) => {
                let run = run_of(&plan.sizes, plan.tries, plan.at_once, PER_MIN);
                self.b.ask(send(id, call(), run));
                self.start(id, before, Life::Run(Track::new(plan)))
            }
            Op::Test(id, works) => {
                let url = if works { MODELS } else { REFUSED };
                let call = call_to(url, NetShape::Form(form()));
                self.b.ask(NetAsk::Test { id, call });
                self.start(id, before, Life::Test(works))
            }
            Op::Part(piece) => self.sound(piece, before),
            Op::Cancel(id) => {
                self.lives.remove(&id);
                self.b.ask(NetAsk::Cancel { id });
                self.expect(before, &[])
            }
            Op::Hand => self.hand(),
        }
    }

    /// After a send or test on `id`: `Gone` at once if it was live, which ends both, else nothing and `life` starts.
    fn start(&mut self, id: u32, before: usize, life: Life) -> Result<(), TestCaseError> {
        let taken = self.lives.remove(&id).is_some();
        if !taken {
            self.lives.insert(id, life);
        }
        let gone = taken.then(|| fault(id, NetFault::Gone));
        self.expect(before, gone.as_slice())
    }

    /// Checks that the notes told since `before` are exactly `want`.
    fn expect(&self, before: usize, want: &[NetNote]) -> Result<(), TestCaseError> {
        prop_assert_eq!(&self.b.out[before..], want);
        Ok(())
    }

    /// A part: a live run takes it as the model says, `Gone` when out of step; any other is dropped unread.
    fn sound(&mut self, (id, piece, n): Piece, before: usize) -> Result<(), TestCaseError> {
        let Some(Life::Run(run)) = self.lives.get_mut(&id) else {
            let stray = sound(n.unwrap_or(FILLED), STRAY);
            self.b.ask(part(id, piece.unwrap_or(0), stray));
            return self.expect(before, &[]);
        };
        let at = u16::try_from(run.cursor.0).unwrap_or(u16::MAX);
        let (piece, n) = (piece.unwrap_or(at), n.unwrap_or_else(|| run.rest()));
        let took = accepts(&run.sizes, &mut run.cursor, piece, n);
        let fill = run.kind.fill();
        self.b.ask(part(id, piece, sound(n, fill)));
        if !took {
            self.lives.remove(&id);
        }
        let gone = (!took).then(|| fault(id, NetFault::Gone));
        self.expect(before, gone.as_slice())
    }

    /// The job takes one outcome, waiting only when one is due, and each note it told is checked in order.
    fn hand(&mut self) -> Result<(), TestCaseError> {
        let before = self.b.out.len();
        let event = if self.lives.values().any(Life::due) {
            Some(self.b.rx.recv_timeout(WAIT).expect("a due outcome"))
        } else {
            self.b.rx.try_recv().ok()
        };
        if let Some(Event::Told(told)) = event {
            self.b.job.told(told, &mut self.b.out).unwrap();
        }
        for note in &self.b.out[before..] {
            heard(&mut self.lives, note)?;
        }
        Ok(())
    }

    /// Hands every due outcome over, then checks what is left: runs waiting for parts, and no stray part sent.
    fn settle(&mut self) -> Result<(), TestCaseError> {
        while self.lives.values().any(Life::due) {
            self.hand()?;
        }
        for (id, life) in &self.lives {
            let waits = matches!(life, Life::Run(run) if run.cursor.0 < run.sizes.len());
            prop_assert!(waits, "{id} live with nothing due");
        }
        let sent = lock(&self.server.0).clone();
        let stray = sent.iter().any(|body| holds(body, STRAY));
        prop_assert!(!stray, "a part for no live run was sent");
        Ok(())
    }
}

/// Checks `note` against `lives`: a text run's next text, or the end of a live run or test, which ends it.
fn heard(lives: &mut HashMap<u32, Life>, note: &NetNote) -> Result<(), TestCaseError> {
    let id = note.id();
    let Some(life) = lives.get_mut(&id) else {
        return Err(TestCaseError::fail(format!("{note:?} for nothing live")));
    };
    if let (NetNote::Text { index, .. }, Life::Run(run)) = (note, &mut *life) {
        prop_assert_eq!(run.kind, Kind::Text, "a text from failing pieces");
        prop_assert_eq!(usize::from(*index), run.told, "texts in order");
        run.told += 1;
        prop_assert!(run.told <= run.cursor.0, "a text for no sent piece");
        return Ok(());
    }
    let early = matches!(life, Life::Run(run) if run.told != run.sizes.len());
    let done = *note == NetNote::Done { id };
    prop_assert!(!(early && done), "done before every text");
    prop_assert_eq!(note, &life.end(id));
    lives.remove(&id);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn each_run_or_test_tells_one_outcome_on_its_id_and_nothing_once_it_is_not_live(
        ops in prop::collection::vec(op(), 0..24),
    ) {
        let server = Arc::new(ByFill::default());
        let b = Bench::new(server.clone(), &Arc::new(Still::default()));
        let mut model = Model { b, server, lives: HashMap::new() };
        for op in ops {
            model.apply(op)?;
        }
        model.settle()?;
    }
}
