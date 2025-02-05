use egui::{CursorIcon, Response};
use epaint::{Pos2, Rect, Vec2};

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum InteractiveSplitterOrientation {
    Horizontal,
    Vertical,
}

/// The response of showing a Splitter
pub struct InteractiveSplitterResponse<R> {
    /// The return value of the closure passed into show.
    pub body_returned: R,
    /// The response of the top or left UI depending on the splitter's orientation.
    pub first_response: Response,
    /// The response of the bottom or right UI depending on the splitter's orientation.
    pub second_response: Response,
    /// The response of the whole splitter widget.
    pub splitter_response: Response,
}

impl InteractiveSplitterOrientation {
    pub(crate) fn rev(self) -> Self {
        match self {
            InteractiveSplitterOrientation::Vertical => InteractiveSplitterOrientation::Horizontal,
            InteractiveSplitterOrientation::Horizontal => InteractiveSplitterOrientation::Vertical,
        }
    }
    pub(crate) fn v(self, v: &Vec2) -> f32 {
        match self {
            InteractiveSplitterOrientation::Vertical => v.x,
            InteractiveSplitterOrientation::Horizontal => v.y,
        }
    }
    pub(crate) fn p(self, p: Pos2) -> f32 {
        match self {
            InteractiveSplitterOrientation::Vertical => p.x,
            InteractiveSplitterOrientation::Horizontal => p.y,
        }
    }
    pub(crate) fn m(self, p: &mut Pos2) -> &mut f32 {
        match self {
            InteractiveSplitterOrientation::Vertical => &mut p.x,
            InteractiveSplitterOrientation::Horizontal => &mut p.y,
        }
    }
    pub(crate) fn r(self, r: &Rect) -> f32 {
        match self {
            InteractiveSplitterOrientation::Vertical => r.height(),
            InteractiveSplitterOrientation::Horizontal => r.width(),
        }
    }
    pub(crate) fn t<T>(self, (a, b): (T, T)) -> (T, T) {
        match self {
            InteractiveSplitterOrientation::Vertical => (a, b),
            InteractiveSplitterOrientation::Horizontal => (b, a),
        }
    }
}

impl From<InteractiveSplitterOrientation> for CursorIcon {
    fn from(value: InteractiveSplitterOrientation) -> Self {
        match value {
            InteractiveSplitterOrientation::Horizontal => CursorIcon::ResizeVertical,
            InteractiveSplitterOrientation::Vertical => CursorIcon::ResizeHorizontal,
        }
    }
}
