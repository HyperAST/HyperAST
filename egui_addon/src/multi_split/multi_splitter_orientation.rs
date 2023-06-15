use epaint::{Pos2, Rect, Vec2};

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum MultiSplitterOrientation {
    Horizontal,
    Vertical,
}

/// The response of showing a Splitter
pub struct MultiSplitterResponse<R> {
    /// The return value of the closure passed into show.
    pub body_returned: R,
    // /// The response of the top or left UI depending on the splitter's orientation.
    // pub first_response: Response,
    // /// The response of the bottom or right UI depending on the splitter's orientation.
    // pub second_response: Response,
    // /// The response of the whole splitter widget.
    // pub splitter_response: Response,
}

impl MultiSplitterOrientation {
    pub(crate) fn rev(self) -> Self {
        match self {
            MultiSplitterOrientation::Vertical => MultiSplitterOrientation::Horizontal,
            MultiSplitterOrientation::Horizontal => MultiSplitterOrientation::Vertical,
        }
    }
    fn v(self, v: &Vec2) -> f32 {
        match self {
            MultiSplitterOrientation::Vertical => v.x,
            MultiSplitterOrientation::Horizontal => v.y,
        }
    }
    pub(crate) fn p(self, p: Pos2) -> f32 {
        match self {
            MultiSplitterOrientation::Vertical => p.x,
            MultiSplitterOrientation::Horizontal => p.y,
        }
    }
    pub(crate) fn m(self, p: &mut Pos2) -> &mut f32 {
        match self {
            MultiSplitterOrientation::Vertical => &mut p.x,
            MultiSplitterOrientation::Horizontal => &mut p.y,
        }
    }
    pub(crate) fn r(self, r: &Rect) -> f32 {
        match self {
            MultiSplitterOrientation::Vertical => r.height(),
            MultiSplitterOrientation::Horizontal => r.width(),
        }
    }
    pub(crate) fn t<T>(self, (a, b): (T, T)) -> (T, T) {
        match self {
            MultiSplitterOrientation::Vertical => (a, b),
            MultiSplitterOrientation::Horizontal => (b, a),
        }
    }
}
