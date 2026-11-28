//! The picture's clicks: Pen and Highlighter put down then lift, Box, Circle, Arrow, Line, Cover and Crop click a start
//! then an end, Crop's end showing its frame, Steps drops the next number, Text opens where clicked, and the Eraser
//! removes the drawing clicked.

use kx_module_api::ports::{Drawing, Mark, Point, Rect, Scale, Shape, Tool};

use super::history::Step;
use super::{Canvas, Did, hit};

impl Canvas {
    /// A click on the picture at `at`, in its pixels, shown at `scale`: a gesture's start or end, a step, a text's
    /// place, an erase, or a click on Crop's frame; while text is typed, a click elsewhere ends it.
    pub fn press(&mut self, at: Point, scale: Scale) -> Did {
        if self.typing.is_some() {
            return self.end();
        }
        match self.tool {
            Tool::Eraser => self.rub(at, scale),
            Tool::Steps => self.mark(Mark::Step {
                at: self.clamp(at),
                n: self.next_step(),
            }),
            Tool::Text => self.start(self.clamp(at)),
            Tool::Crop if self.waiting.is_some() => self.adjust(at, scale),
            _ => self.click(at, scale),
        }
    }

    /// A pen's or highlighter's path since its first click; its last point lifts it.
    pub fn stroke(&mut self, path: &[Point]) -> Did {
        let pen = matches!(self.tool, Tool::Pen | Tool::Highlighter);
        let Some(from) = self.open.filter(|_| pen) else {
            return Did::Stay;
        };
        self.open = None;
        let points = path.iter().map(|at| self.clamp(*at));
        self.pen(std::iter::once(from).chain(points).collect())
    }

    /// A two-click gesture's start, or its end, which adds its drawing or frames a crop; a shape or crop whose clicks
    /// are one spot at `scale` adds nothing.
    fn click(&mut self, at: Point, scale: Scale) -> Did {
        let at = self.clamp(at);
        let Some(from) = self.open.take() else {
            self.open = Some(at);
            return Did::Show;
        };
        let pen = matches!(self.tool, Tool::Pen | Tool::Highlighter);
        if !pen && hit::one_spot(from, at, hit::slop(&self.fit, scale)) {
            return Did::Show;
        }
        let did = match (self.tool, shape(self.tool)) {
            (Tool::Crop, _) => span(from, at).map_or(Did::Stay, |part| self.wait(part)),
            (Tool::Pen | Tool::Highlighter, _) => self.pen(vec![from, at]),
            (_, Some(shape)) if !flat(shape, from, at) => self.mark(Mark::Shape {
                shape,
                from,
                to: at,
            }),
            _ => Did::Stay,
        };
        did.max(Did::Show)
    }

    /// A pen's or highlighter's path, once it leaves its first point.
    fn pen(&mut self, mut path: Vec<Point>) -> Did {
        path.dedup();
        if path.len() < 2 {
            return Did::Show;
        }
        let mark = if self.tool == Tool::Highlighter {
            Mark::Highlight(path)
        } else {
            Mark::Pen(path)
        };
        self.mark(mark)
    }

    /// The Eraser: removes the top drawing that shows within the slop of `at` at `scale`; an empty spot, or one outside
    /// the picture, does nothing.
    fn rub(&mut self, at: Point, scale: Scale) -> Did {
        if self.clamp(at) != at {
            return Did::Stay;
        }
        let aim = hit::Aim::new(&self.fit, scale, self.sheet.crop.rect());
        match aim.top(&self.sheet.drawings, at) {
            Some(place) => self.apply(Step::Erase(place)),
            None => Did::Stay,
        }
    }

    /// The number after the highest step shown, so Steps count on from what an Undo left.
    fn next_step(&self) -> u16 {
        let shown = self.sheet.drawings.iter().filter_map(|d| match d.mark {
            Mark::Step { n, .. } => Some(n),
            _ => None,
        });
        shown.max().unwrap_or(0).saturating_add(1)
    }

    /// Adds `mark` on top in the ink and thickness picked, as one step.
    fn mark(&mut self, mark: Mark) -> Did {
        let (ink, size) = (self.ink, self.thick.step);
        self.apply(Step::Add(Drawing { mark, ink, size }))
    }
}

/// The shape a two-click tool draws.
const fn shape(tool: Tool) -> Option<Shape> {
    match tool {
        Tool::Box => Some(Shape::Box),
        Tool::Circle => Some(Shape::Circle),
        Tool::Arrow => Some(Shape::Arrow),
        Tool::Line => Some(Shape::Line),
        Tool::Cover => Some(Shape::Cover),
        _ => None,
    }
}

/// A shape that draws nothing: a line of no length, or a box, circle or cover with no width or height.
fn flat(shape: Shape, from: Point, to: Point) -> bool {
    match shape {
        Shape::Line | Shape::Arrow => from == to,
        Shape::Box | Shape::Circle | Shape::Cover => from.x == to.x || from.y == to.y,
    }
}

/// The rectangle from pixel `one` to pixel `two`, both inside, in either order.
fn span(one: Point, two: Point) -> Option<Rect> {
    let side = |a: i32, b: i32| i32::try_from(a.abs_diff(b)).ok()?.checked_add(1);
    Some(Rect {
        x: one.x.min(two.x),
        y: one.y.min(two.y),
        w: side(one.x, two.x)?,
        h: side(one.y, two.y)?,
    })
}
