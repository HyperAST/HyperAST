// started from a draft on egui's github

use epaint::{
    emath::{lerp, Align},
    Pos2, Rect, Vec2,
};

use egui::{egui_assert, Layout, Response, Sense, Ui};

/// A splitter which can separate the UI into 2 parts either vertically or horizontally.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// egui::Splitter::vertical().show(ui, |ui_left, ui_right| {
///     ui_left.label("I'm on the left!");
///     ui_right.label("I'm on the right!");
/// });
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct Splitter {
    orientation: SplitterOrientation,
    ratio: f32,
}

impl Splitter {
    /// Create a new splitter with the given orientation and a ratio of 0.5.
    pub fn with_orientation(orientation: SplitterOrientation) -> Self {
        Self {
            orientation,
            ratio: 0.5,
        }
    }

    /// Create a new vertical splitter with a ratio of 0.5.
    #[inline]
    pub fn vertical() -> Self {
        Self::with_orientation(SplitterOrientation::Vertical)
    }

    /// Create a new horizontal splitter with a ratio of 0.5.
    #[inline]
    pub fn horizontal() -> Self {
        Self::with_orientation(SplitterOrientation::Horizontal)
    }

    /// Set the ratio of the splitter.
    ///
    /// The ratio sets where the splitter splits the current UI, where, depending on the
    /// orientation, 0.0 would mean split at the very top/left and 1.0 would mean split at the very
    /// bottom/right respectively. The ratio must be in the range 0.0..=1.0.
    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio;
        self
    }

    #[inline]
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui, &mut Ui) -> R,
    ) -> SplitterResponse<R> {
        self.show_dyn(ui, Box::new(add_contents))
    }

    pub fn show_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_contents: Box<dyn FnOnce(&mut Ui, &mut Ui) -> R + 'c>,
    ) -> SplitterResponse<R> {
        let Self { orientation, ratio } = self;

        egui_assert!((0.0..=1.0).contains(&ratio));

        let (rect, splitter_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

        let line_pos_1 = orientation
            .t((
                lerp(orientation.p(rect.min)..=orientation.p(rect.max), ratio),
                orientation.rev().p(rect.min),
            ))
            .into();
        let line_pos_2 = line_pos_1 + orientation.t((0.0, orientation.r(&rect))).into();

        let line_pos_1 = ui.painter().round_pos_to_pixels(line_pos_1);
        let line_pos_2 = ui.painter().round_pos_to_pixels(line_pos_2);

        let i_spacing = &ui.style().spacing.item_spacing;
        let first_rect = {
            let mut rect = rect;
            *orientation.m(&mut rect.max) = orientation.p(line_pos_1) - orientation.v(i_spacing);
            rect
        };
        let second_rect = {
            let mut rect = rect;
            *orientation.m(&mut rect.min) = orientation.p(line_pos_1) + orientation.v(i_spacing);
            rect
        };

        let mut first_ui = ui.child_ui(first_rect, Layout::top_down(Align::Min));
        let mut second_ui = ui.child_ui(second_rect, Layout::top_down(Align::Min));

        let body_returned = add_contents(&mut first_ui, &mut second_ui);

        ui.painter().line_segment(
            [line_pos_1, line_pos_2],
            ui.visuals().widgets.noninteractive.bg_stroke,
        );

        SplitterResponse {
            splitter_response,
            body_returned,
            first_response: ui.interact(first_rect, first_ui.id(), Sense::hover()),
            second_response: ui.interact(second_rect, second_ui.id(), Sense::hover()),
        }
    }
}

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
    fn rev(self) -> Self {
        match self {
            SplitterOrientation::Vertical => SplitterOrientation::Horizontal,
            SplitterOrientation::Horizontal => SplitterOrientation::Vertical,
        }
    }
    fn v(self, v: &Vec2) -> f32 {
        match self {
            SplitterOrientation::Vertical => v.x,
            SplitterOrientation::Horizontal => v.y,
        }
    }
    fn p(self, p: Pos2) -> f32 {
        match self {
            SplitterOrientation::Vertical => p.x,
            SplitterOrientation::Horizontal => p.y,
        }
    }
    fn m(self, p: &mut Pos2) -> &mut f32 {
        match self {
            SplitterOrientation::Vertical => &mut p.x,
            SplitterOrientation::Horizontal => &mut p.y,
        }
    }
    fn r(self, r: &Rect) -> f32 {
        match self {
            SplitterOrientation::Vertical => r.height(),
            SplitterOrientation::Horizontal => r.width(),
        }
    }
    fn t<T>(self, (a, b): (T, T)) -> (T, T) {
        match self {
            SplitterOrientation::Vertical => (a, b),
            SplitterOrientation::Horizontal => (b, a),
        }
    }
}
