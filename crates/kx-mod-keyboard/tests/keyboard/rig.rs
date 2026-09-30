//! The keyboard face as the UI meets it: the module started on a fake host, its face taken from the services,
//! and the JSON it draws from.

#![allow(
    clippy::unwrap_used,
    reason = "these fixtures run only in tests, where a failed setup should fail the test"
)]

use crate::host::{FakeHost, Fakes, fake, id};
use kx_mod_keyboard::KeyboardModule;
use kx_module_api::Capability::InjectInput;
use kx_module_api::ports::{
    FaceAction, FaceError, FaceReply, KeyId, KeyboardFace, KeyboardFaceKey,
};
use kx_module_api::{Module, ModuleCx, ServiceKey};
use serde_json::Value;
use std::sync::Arc;

/// A redraw of the state alone.
pub const STATE: FaceReply = FaceReply {
    board: false,
    state: true,
    notice: None,
};

/// A redraw of the board alone.
pub const BOARD: FaceReply = FaceReply {
    board: true,
    state: false,
    notice: None,
};

/// The started module, its host and fakes, and the face it provides.
pub struct Rig {
    /// The host the module last started on.
    pub host: FakeHost,
    /// The fakes behind the typing services.
    pub fakes: Fakes,
    /// The module, which keeps its face across restarts.
    pub module: KeyboardModule,
    /// The face, as the UI takes it.
    pub face: Arc<dyn KeyboardFace>,
}

/// Starts `module` on a host with `change` over the defaults and `fakes` stored; returns the host and the face.
fn start(
    module: &mut KeyboardModule,
    fakes: &Fakes,
    change: Option<(&str, toml::Value)>,
) -> (FakeHost, Arc<dyn KeyboardFace>) {
    let mut host = fake(change);
    fakes.store(&mut host, None);
    host.granted = vec![InjectInput];
    module.start(&mut ModuleCx::new(&mut host)).unwrap();
    let (_, service) = &host.services[&KeyboardFaceKey::ID];
    let face = service.downcast_ref::<Arc<dyn KeyboardFace>>().unwrap();
    let face = Arc::clone(face);
    (host, face)
}

impl Rig {
    /// The module started on layouts `names`, the first shown, with `change` over the defaults.
    pub fn new(names: &[&str], change: Option<(&str, toml::Value)>) -> Self {
        let fakes = Fakes::golden(names);
        let mut module = KeyboardModule::default();
        let (host, face) = start(&mut module, &fakes, change);
        Self {
            host,
            fakes,
            module,
            face,
        }
    }

    /// Restarts the module as a settings change does, with `change` over the defaults.
    pub fn restart(&mut self, change: Option<(&str, toml::Value)>) {
        self.module.stop();
        let (host, face) = start(&mut self.module, &self.fakes, change);
        self.host = host;
        self.face = face;
    }

    /// Does `action`.
    pub fn act(&self, action: FaceAction) -> Result<FaceReply, FaceError> {
        self.face.act(action)
    }

    /// A click on key `id`, on the language key's back part when `back`.
    pub fn click(&self, id: &str, back: bool) -> Result<FaceReply, FaceError> {
        let id = KeyId::new(id);
        self.act(FaceAction::Tap { id, back })
    }

    /// A click on key `id`.
    pub fn tap(&self, id: &str) -> Result<FaceReply, FaceError> {
        self.click(id, false)
    }

    /// Key `id` sent to the open panel, at `rows` rows per page.
    pub fn panel(&self, id: &str, rows: usize) -> Result<FaceReply, FaceError> {
        let key = KeyId::new(id);
        self.act(FaceAction::PanelKey { key, rows })
    }

    /// The app in front switched to golden layout `name`, and the relabel timer ran.
    pub fn switch(&self, name: &str) -> FaceReply {
        self.fakes.source.set_foreground(Some(id(name)));
        assert!(self.fakes.layouts.switch(id(name)));
        self.act(FaceAction::Relabel).unwrap()
    }

    /// The state the UI draws.
    pub fn state(&self) -> Value {
        serde_json::from_str(&self.face.state_json()).unwrap()
    }

    /// The board the UI draws.
    pub fn board(&self) -> Value {
        serde_json::from_str(&self.face.board_json()).unwrap()
    }

    /// The short name the language key shows now.
    pub fn shown(&self) -> String {
        self.state()["lang"]["cur"]["short"]
            .as_str()
            .unwrap()
            .to_owned()
    }
}
