//! The `KeyboardFace` service: the board and state JSON the UI draws, and each click the UI forwards. The UI calls
//! it from one thread, which runs the relabel timer too, so `Layouts::refresh` is only ever called from there.

mod inner;

use crate::geom::{KeyBox, board};
use crate::panel::navs;
use crate::run::{Ports, TapError, Tapped};
use crate::settings::KeyboardSettings;
use crate::spec::{BoardSpec, Nav};
use crate::view::board_view;
use inner::{Inner, Went};
use kx_module_api::ports::{FaceAction, FaceError, FaceReply, KeyboardFace};
use kx_module_api::{SettingsHandle, keys};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// A reply that redraws the state alone.
const STATE: FaceReply = FaceReply {
    board: false,
    state: true,
    notice: None,
    holding: false,
};

/// The keyboard as the UI sees it: the typing state, size, place and language key behind one lock.
pub struct Face {
    spec: BoardSpec,
    /// Every key at size 1, for the state and the key checks.
    keys: Vec<KeyBox>,
    /// The board keys that drive an open panel.
    navs: HashMap<String, Nav>,
    /// The same keys in board order, which the state lists.
    nav_keys: Vec<(String, Nav)>,
    inner: Mutex<Inner>,
}

/// Shows no key, place or typing state.
impl fmt::Debug for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Face").finish_non_exhaustive()
    }
}

impl Face {
    /// The face of board `spec` over `ports`, with `settings`; `store` saves the size and place the user sets.
    #[must_use]
    pub fn new(
        spec: BoardSpec,
        ports: Ports,
        settings: KeyboardSettings,
        store: SettingsHandle,
    ) -> Self {
        let b = board(&spec, 0.0, 1.0);
        let navs = navs(&spec);
        let inner = Inner::new(&spec, b.plate, ports, settings, store);
        Self {
            keys: b.keys,
            navs: navs.iter().cloned().collect(),
            nav_keys: navs,
            spec,
            inner: Mutex::new(inner),
        }
    }

    /// Takes new services and settings after a restart, keeping the latches, a dead key's wait and an open list
    /// whose design stays.
    pub fn renew(&self, ports: Ports, settings: KeyboardSettings, store: SettingsHandle) {
        self.lock().renew(ports, settings, store);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A click on key `id`; while the list is open, a key that drives it does nothing here, as the UI sends it
    /// as `panel_key`, so it never reaches the app.
    fn tap(&self, id: &str, back: bool) -> Result<FaceReply, FaceError> {
        let went = {
            let mut inner = self.lock();
            if inner.listing() && self.navs.contains_key(id) {
                return Ok(FaceReply::default());
            }
            inner.tap(id, back)
        };
        self.reply(went)
    }

    /// The pointer went down on key `id` (spec §4.6): it types now, answering as a tap does and saying whether it
    /// holds; while the list is open, a key that drives it waits for `panel_key`, as in `tap`.
    fn press(&self, id: &str) -> Result<FaceReply, FaceError> {
        self.known(id)?;
        let (went, holding) = {
            let mut inner = self.lock();
            if inner.listing() && self.navs.contains_key(id) {
                return Ok(FaceReply::default());
            }
            let went = inner.press(id);
            (went, inner.holding())
        };
        Ok(FaceReply {
            holding,
            ..self.reply(went)?
        })
    }

    /// The hold timer: a repeat that went out redraws only when its burst ended, as the latches stay latched until
    /// then; any other due repeat answers as a tap does. Each reply says whether the key is still held.
    fn tick(&self) -> Result<FaceReply, FaceError> {
        let (went, holding) = {
            let mut inner = self.lock();
            let went = inner.tick();
            (went, inner.holding())
        };
        let Some(went) = went else {
            return Ok(FaceReply {
                holding,
                ..FaceReply::default()
            });
        };
        if matches!(went.tapped, Ok(Tapped::Done)) {
            return Ok(FaceReply {
                state: went.ended,
                holding,
                ..FaceReply::default()
            });
        }
        Ok(FaceReply {
            holding,
            ..self.reply(went)?
        })
    }

    /// The pointer let go of key `id`, wherever it is: it typed at the press, so only its burst ends, and only when
    /// `id` is the key held.
    fn let_go(&self, id: &str) -> Result<FaceReply, FaceError> {
        self.known(id)?;
        Ok(self.lock().let_go(id))
    }

    /// Refuses a key that is not on the board, changing nothing, since the UI redraws nothing on an error.
    fn known(&self, id: &str) -> Result<(), FaceError> {
        if self.keys.iter().any(|k| k.id == id) {
            Ok(())
        } else {
            Err(FaceError::UnknownKey)
        }
    }

    /// What to redraw after `went`: as its typing says, and the state too when a burst ended.
    fn reply(&self, went: Went) -> Result<FaceReply, FaceError> {
        let reply = self.answer(went.tapped)?;
        Ok(FaceReply {
            state: reply.state || went.ended,
            ..reply
        })
    }

    /// What to redraw after a tap or a repeat did `tapped`.
    fn answer(&self, tapped: Result<Tapped, TapError>) -> Result<FaceReply, FaceError> {
        match tapped {
            Ok(Tapped::Done) => Ok(STATE),
            Ok(Tapped::Later | Tapped::Empty) => Ok(FaceReply::default()),
            Ok(Tapped::Lang { back }) => Ok(self.lock().lang(back)),
            Ok(Tapped::Stale) => Ok(FaceReply {
                state: true,
                ..self.refresh()
            }),
            Err(TapError::Inject(_)) => Ok(FaceReply {
                notice: Some(keys::KEYS_NOT_SENT),
                ..STATE
            }),
            Err(TapError::UnknownKey) => Err(FaceError::UnknownKey),
        }
    }

    /// Reads the layouts again, without the lock, since the change event may call back into the face.
    fn refresh(&self) -> FaceReply {
        let layouts = self.lock().layouts();
        let changed = layouts.refresh();
        let moved = self.lock().seen();
        FaceReply {
            state: changed || moved,
            ..FaceReply::default()
        }
    }

    /// Key `key` for the open list, at `rows` rows per page.
    fn panel_key(&self, key: &str, rows: usize) -> Result<FaceReply, FaceError> {
        match self.navs.get(key) {
            Some(&nav) => self.lock().panel(nav, rows),
            None => self.known(key).map(|()| FaceReply::default()),
        }
    }
}

impl KeyboardFace for Face {
    fn board_json(&self) -> String {
        let (settings, size) = self.lock().sizing();
        json(&board_view(&self.spec, &settings, size))
    }

    fn state_json(&self) -> String {
        json(&self.lock().state(&self.keys, &self.nav_keys))
    }

    fn lang(&self) -> String {
        let layouts = self.lock().layouts();
        layouts.current().label.iso.clone()
    }

    fn row_px(&self) -> f64 {
        self.spec.row_px
    }

    fn act(&self, action: FaceAction) -> Result<FaceReply, FaceError> {
        match action {
            FaceAction::Tap { id, back } => self.tap(id.as_str(), back),
            FaceAction::Press { id } => self.press(id.as_str()),
            FaceAction::Tick => self.tick(),
            FaceAction::LetGo { id } => self.let_go(id.as_str()),
            FaceAction::Leave => Ok(self.lock().leave()),
            FaceAction::Relabel => Ok(self.refresh()),
            FaceAction::PanelKey { key, rows } => self.panel_key(key.as_str(), rows),
            other => self.lock().act(other),
        }
    }
}

/// `view` as JSON; the views hold only strings, numbers and flags, so it always serialises.
fn json(view: &impl serde::Serialize) -> String {
    serde_json::to_string(view).unwrap_or_default()
}
