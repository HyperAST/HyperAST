use egui::{CursorIcon, Response};
use epaint::{Pos2, Rect, Vec2};

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum SplitterOrientation {
    Horizontal,
    Vertical,
}

/// The response of showing a Splitter
pub struct SplitterResponse<R> {
    /// The return value of the closure passed into show.
    pub body_returned: R,
    /// The response of the top or left UI depending on the splitter's orientation.
    pub first_response: Response,
    /// The response of the bottom or right UI depending on the splitter's orientation.
    pub second_response: Response,
    /// The response of the whole splitter widget.
    pub splitter_response: Response,
}

impl SplitterOrientation {
    pub(crate) fn rev(self) -> Self {
        match self {
            SplitterOrientation::Vertical => SplitterOrientation::Horizontal,
            SplitterOrientation::Horizontal => SplitterOrientation::Vertical,
        }
    }
    pub(crate) fn v(self, v: &Vec2) -> f32 {
        match self {
            SplitterOrientation::Vertical => v.x,
            SplitterOrientation::Horizontal => v.y,
        }
    }
    pub(crate) fn p(self, p: Pos2) -> f32 {
        match self {
            SplitterOrientation::Vertical => p.x,
            SplitterOrientation::Horizontal => p.y,
        }
    }
    pub(crate) fn m(self, p: &mut Pos2) -> &mut f32 {
        match self {
            SplitterOrientation::Vertical => &mut p.x,
            SplitterOrientation::Horizontal => &mut p.y,
        }
    }
    pub(crate) fn r(self, r: &Rect) -> f32 {
        match self {
            SplitterOrientation::Vertical => r.height(),
            SplitterOrientation::Horizontal => r.width(),
        }
    }
    pub(crate) fn t<T>(self, (a, b): (T, T)) -> (T, T) {
        match self {
            SplitterOrientation::Vertical => (a, b),
            SplitterOrientation::Horizontal => (b, a),
        }
    }
}

impl From<SplitterOrientation> for CursorIcon {
    fn from(value: SplitterOrientation) -> Self {
        match value {
            SplitterOrientation::Horizontal => CursorIcon::ResizeVertical,
            SplitterOrientation::Vertical => CursorIcon::ResizeHorizontal,
        }
    }
}
