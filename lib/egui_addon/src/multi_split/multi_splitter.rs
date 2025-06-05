use egui::{Align, Layout, Sense, Ui};

use super::multi_splitter_orientation::{MultiSplitterOrientation, MultiSplitterResponse};

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
pub struct MultiSplitter {
    orientation: MultiSplitterOrientation,
    // should be inferior to 1
    ratios: Vec<f32>,
}

impl MultiSplitter {
    /// Create a new splitter with the given orientation and a ratio of 0.5.
    pub fn with_orientation(orientation: MultiSplitterOrientation) -> Self {
        Self {
            orientation,
            ratios: vec![0.25, 0.25, 0.25],
        }
    }

    /// Create a new vertical splitter with a ratio of 0.5.
    #[inline]
    pub fn vertical() -> Self {
        Self::with_orientation(MultiSplitterOrientation::Vertical)
    }

    /// Create a new horizontal splitter with a ratio of 0.5.
    #[inline]
    pub fn horizontal() -> Self {
        Self::with_orientation(MultiSplitterOrientation::Horizontal)
    }

    /// Set the ratio of the splitter.
    ///
    /// The ratio sets where the splitter splits the current UI, where, depending on the
    /// orientation, 0.0 would mean split at the very top/left and 1.0 would mean split at the very
    /// bottom/right respectively. The ratio must be in the range 0.0..=1.0.
    pub fn ratios(mut self, ratios: Vec<f32>) -> Self {
        self.ratios = ratios;
        self
    }

    #[inline]
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut [Ui]) -> R,
    ) -> MultiSplitterResponse<R> {
        self.show_dyn(ui, Box::new(add_contents))
    }

    pub fn show_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_contents: Box<dyn FnOnce(&mut [Ui]) -> R + 'c>,
    ) -> MultiSplitterResponse<R> {
        let Self {
            orientation,
            ratios,
        } = self;

        {
            debug_assert!(
                (0.0..=1.0).contains(
                    &ratios
                        .iter()
                        .map(|ratio| {
                            debug_assert!((0.0..=1.0).contains(ratio));
                            *ratio
                        })
                        .sum::<f32>()
                )
            );
        }

        let (rect, _splitter_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

        // let ratio = ratios[0];

        // let line_pos_1 = orientation
        //     .t((
        //         lerp(orientation.p(rect.min)..=orientation.p(rect.max), ratio),
        //         orientation.rev().p(rect.min),
        //     ))
        //     .into();
        // let line_pos_2 = line_pos_1 + orientation.t((0.0, orientation.r(&rect))).into();

        // let line_pos_1 = ui.painter().round_pos_to_pixels(line_pos_1);
        // let line_pos_2 = ui.painter().round_pos_to_pixels(line_pos_2);

        // let first_rect = {
        //     let mut rect = rect;
        //     *orientation.m(&mut rect.max) = orientation.p(line_pos_1) - orientation.v(i_spacing);
        //     rect
        // };
        // let second_rect = {
        //     let mut rect = rect;
        //     *orientation.m(&mut rect.min) = orientation.p(line_pos_1) + orientation.v(i_spacing);
        //     rect
        // };

        // let mut first_ui = ui.child_ui(first_rect, Layout::top_down(Align::Min));
        // let mut second_ui = ui.child_ui(second_rect, Layout::top_down(Align::Min));
        let mut remaining_rect = rect;
        let (mut uis, lines): (Vec<_>, Vec<_>) = ratios
            .iter()
            .map(|ratio| {
                let _i_spacing = &ui.style().spacing.item_spacing;
                let line_pos_1 = orientation
                    .t((
                        orientation.p(remaining_rect.min) + orientation.rev().r(&rect) * ratio, //lerp(orientation.p(rect.min)..=orientation.p(rect.max), *ratio),
                        orientation.rev().p(remaining_rect.min),
                    ))
                    .into();
                let line_pos_2 =
                    line_pos_1 + orientation.t((0.0, orientation.r(&remaining_rect))).into();

                let mut patition_rect = {
                    let mut rect = remaining_rect;
                    *orientation.m(&mut rect.max) = orientation.p(line_pos_1); // - orientation.v(i_spacing);
                    rect
                };
                *orientation.m(&mut remaining_rect.min) = *orientation.m(&mut patition_rect.max);

                let line_pos_1 =
                    egui::emath::GuiRounding::round_to_pixels(line_pos_1, ui.pixels_per_point());
                let line_pos_2 =
                    egui::emath::GuiRounding::round_to_pixels(line_pos_1, ui.pixels_per_point());
                let cui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(patition_rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                (cui, [line_pos_1, line_pos_2])
            })
            .unzip();

        uis.push(
            ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(remaining_rect)
                    .layout(Layout::top_down(Align::Min)),
            ),
        );

        let body_returned = add_contents(&mut uis);

        for line in lines {
            ui.painter()
                .line_segment(line, ui.visuals().widgets.noninteractive.bg_stroke);
        }

        MultiSplitterResponse {
            // splitter_response,
            body_returned,
            // first_response: ui.interact(first_rect, first_ui.id(), Sense::hover()),
            // second_response: ui.interact(second_rect, second_ui.id(), Sense::hover()),
        }
    }
}
