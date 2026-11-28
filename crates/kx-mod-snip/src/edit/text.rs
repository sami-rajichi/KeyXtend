//! The text being typed: it opens where Text clicks, takes the keyboard's letters at its caret as a panel's line does
//! (ADR-0040), reads right to left when its first letter does, and ends on Enter or a click elsewhere.

use kx_module_api::caret::{self, Side};
use kx_module_api::ports::{Drawing, Mark, PanelKey, Point, TextEdit};
use kx_module_api::{Redacted, is_hidden, is_rtl};

use super::history::Step;
use super::{Canvas, Did};

/// What the line's Space adds.
const SPACE: &str = " ";

/// The text being typed: where it starts, the text and its caret, which never print.
#[derive(Clone, Debug)]
pub(super) struct Typing {
    at: Point,
    text: Redacted<String>,
    /// A byte offset on a char boundary of the text.
    caret: Redacted<usize>,
}

impl Typing {
    /// What the view draws: the text, its caret in UTF-16 units, and its direction.
    pub(super) fn view(&self) -> TextEdit {
        let text = self.text.expose();
        TextEdit {
            at: self.at,
            text: self.text.clone(),
            caret: Redacted::new(caret::units(text, self.at())),
            rtl: is_rtl(text),
        }
    }

    /// The text shows something besides spaces and hidden marks, so it would join the drawings.
    pub(super) fn shows(&self) -> bool {
        let text = self.text.expose();
        text.chars().any(|c| !c.is_whitespace() && !is_hidden(c))
    }

    /// The caret, on a char boundary of the text.
    fn at(&self) -> usize {
        caret::clamped(self.text.expose(), *self.caret.expose())
    }

    /// Makes the text `head` then `tail`, with the caret after `head`; true when either changed.
    fn set(&mut self, head: &str, tail: &str) -> bool {
        let text = format!("{head}{tail}");
        let moved = self.go_to(head.len());
        let changed = text != *self.text.expose();
        self.text = Redacted::new(text);
        moved || changed
    }

    /// Moves the caret to `to`; true when it moved.
    fn go_to(&mut self, to: usize) -> bool {
        let moved = to != self.at();
        self.caret = Redacted::new(to);
        moved
    }

    /// Erases up to `erase` characters before the caret and adds `text` at it.
    fn write(&mut self, erase: u16, text: &str) -> bool {
        let line = self.text.expose().clone();
        let (head, tail) = line.split_at(self.at());
        let mut head = head.to_owned();
        for _ in 0..erase {
            head.pop();
        }
        head.push_str(text);
        self.set(&head, tail)
    }

    /// Removes the drawn grapheme after the caret.
    fn delete(&mut self) -> bool {
        let line = self.text.expose().clone();
        let at = self.at();
        let tail = line.get(caret::on(&line, at)..).unwrap_or_default();
        self.set(caret::before(&line, at), tail)
    }
}

impl Canvas {
    /// Text is being typed, so the editor's line shows and takes the letters.
    #[must_use]
    pub fn typing(&self) -> bool {
        self.typing.is_some()
    }

    /// Adds `text` at the caret of the text being typed.
    pub fn write(&mut self, text: &str) -> Did {
        self.retype(|t| t.write(0, text))
    }

    /// Backspace: erases the character before the caret.
    pub fn erase(&mut self) -> Did {
        self.retype(|t| t.write(1, ""))
    }

    /// Delete: removes the grapheme after the caret.
    pub fn delete(&mut self) -> Did {
        self.retype(Typing::delete)
    }

    /// Left or Right moves the caret one grapheme, in the text's reading direction.
    pub fn arrow(&mut self, side: Side) -> Did {
        self.retype(|t| t.go_to(caret::arrow(t.text.expose(), t.at(), side)))
    }

    /// A click on the text being typed puts its caret `units` UTF-16 units in.
    pub fn caret(&mut self, units: u32) -> Did {
        self.retype(|t| t.go_to(caret::of_units(t.text.expose(), units)))
    }

    /// Enter or a click elsewhere: the text being typed ends, and joins the drawings unless it shows nothing. The actor
    /// calls it before a Copy, Save, Close or Snip key, and tells the flow `Changed` first when it is a change.
    pub fn end(&mut self) -> Did {
        let Some(typing) = self.typing.take() else {
            return Did::Stay;
        };
        match self.typed(typing) {
            Some(drawing) => self.apply(Step::Add(drawing)),
            None => Did::Show,
        }
    }

    /// The drawing `typing` makes in the ink and text size picked; `None` when it shows nothing.
    pub(super) fn typed(&self, typing: Typing) -> Option<Drawing> {
        if !typing.shows() {
            return None;
        }
        let text = typing.text.into_inner();
        let rtl = is_rtl(&text);
        let mark = Mark::Text {
            at: typing.at,
            text: Redacted::new(text),
            rtl,
        };
        let (ink, size) = (self.ink, self.text.step);
        Some(Drawing { mark, ink, size })
    }

    /// A key of the editor's line (ADR-0040); `None` when no text is being typed, so the key is not the line's.
    pub fn line(&mut self, key: &PanelKey) -> Option<Did> {
        self.typing.as_ref()?;
        Some(match key {
            PanelKey::Text(text) => self.write(text.expose()),
            PanelKey::Tick => self.write(SPACE),
            PanelKey::Erase => self.erase(),
            PanelKey::Delete => self.delete(),
            PanelKey::Left => self.arrow(Side::Left),
            PanelKey::Right => self.arrow(Side::Right),
            PanelKey::Word { erase, text } => self.retype(|t| t.write(*erase, text.expose())),
            PanelKey::Enter | PanelKey::Esc => self.end(),
            PanelKey::Up | PanelKey::Down => Did::Stay,
        })
    }

    /// Opens a text at `at`, inside the picture.
    pub(super) fn start(&mut self, at: Point) -> Did {
        self.typing = Some(Typing {
            at,
            text: Redacted::new(String::new()),
            caret: Redacted::new(0),
        });
        Did::Show
    }

    /// Edits the text being typed with `edit`, which tells whether it changed.
    fn retype(&mut self, edit: impl FnOnce(&mut Typing) -> bool) -> Did {
        match self.typing.as_mut().map(edit) {
            Some(true) => Did::Show,
            _ => Did::Stay,
        }
    }
}
