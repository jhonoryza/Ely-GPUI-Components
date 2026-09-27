use gpui::SharedString;

use super::view::Frame;

/// What a shape is.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeKind {
    Rect,
    Ellipse,
    /// A regular polygon of so many sides, three or more.
    Polygon(u8),
    /// From the frame's top left to its bottom right.
    Line,
    /// A line with a head at its end.
    Arrow,
    Text(SharedString),
    /// Points in canvas units from the frame's top left.
    Path(Vec<(f32, f32)>),
    /// A square of paper with words on it.
    Note(SharedString),
}

/// Something on a canvas: its key, its name, what it is, its frame, its hue among the chart colors, and whether it hides or is locked.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub key: SharedString,
    pub name: SharedString,
    pub kind: ShapeKind,
    pub frame: Frame,
    pub hue: usize,
    pub hidden: bool,
    pub locked: bool,
}

impl Shape {
    pub fn new(
        key: impl Into<SharedString>,
        name: impl Into<SharedString>,
        kind: ShapeKind,
        frame: Frame,
    ) -> Self {
        if let ShapeKind::Polygon(sides) = kind {
            assert!(sides >= 3, "a polygon of {sides} sides");
        }
        Self {
            key: key.into(),
            name: name.into(),
            kind,
            frame,
            hue: 0,
            hidden: false,
            locked: false,
        }
    }

    pub fn hue(mut self, hue: usize) -> Self {
        self.hue = hue;
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn locked(mut self, locked: bool) -> Self {
        self.locked = locked;
        self
    }
}

/// A named board on the canvas that designs sit on.
#[derive(Clone, Debug, PartialEq)]
pub struct Artboard {
    pub key: SharedString,
    pub name: SharedString,
    pub frame: Frame,
}

impl Artboard {
    pub fn new(key: impl Into<SharedString>, name: impl Into<SharedString>, frame: Frame) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            frame,
        }
    }
}

/// A line across the canvas to line things up with.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Guide {
    /// At this canvas x.
    Vertical(f32),
    /// At this canvas y.
    Horizontal(f32),
}

/// The corners of a regular polygon of `sides` in `frame`, the first at the top.
pub(crate) fn polygon(frame: &Frame, sides: u8) -> Vec<(f32, f32)> {
    let (cx, cy) = frame.center();
    (0..sides)
        .map(|ix| {
            let turn = std::f32::consts::TAU * f32::from(ix) / f32::from(sides)
                - std::f32::consts::FRAC_PI_2;
            (
                cx + frame.w / 2.0 * turn.cos(),
                cy + frame.h / 2.0 * turn.sin(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Frame, polygon};

    #[test]
    fn a_polygon_starts_at_the_top_and_goes_round() {
        let corners = polygon(&Frame::new(0.0, 0.0, 100.0, 100.0), 4);
        let round = |(x, y): (f32, f32)| (x.round(), y.round());
        assert_eq!(
            corners.into_iter().map(round).collect::<Vec<_>>(),
            [(50.0, 0.0), (100.0, 50.0), (50.0, 100.0), (0.0, 50.0)]
        );
    }
}
