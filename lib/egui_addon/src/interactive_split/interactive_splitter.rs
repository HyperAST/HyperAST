use super::interactive_splitter_orientation::{
    InteractiveSplitterOrientation, InteractiveSplitterResponse,
};
use crate::interactive_split::interactive_split_state::InteractiveSplitState;
use egui::{Align, CursorIcon, Frame, Layout, Sense, Ui, lerp};
use epaint::Stroke;

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
pub struct InteractiveSplitter {
    orientation: InteractiveSplitterOrientation,
    ratio: f32,
    resizable: bool,
}

impl InteractiveSplitter {
    /// Create a new splitter with the given orientation and a ratio of 0.5.
    pub fn with_orientation(orientation: InteractiveSplitterOrientation) -> Self {
        Self {
            orientation,
            ratio: 0.5,
            resizable: true,
        }
    }

    /// Create a new vertical splitter with a ratio of 0.5.
    #[inline]
    pub fn vertical() -> Self {
        Self::with_orientation(InteractiveSplitterOrientation::Vertical)
    }

    // /// Create a new horizontal splitter with a ratio of 0.5.
    // #[inline]
    // pub fn horizontal() -> Self {
    //     Self::with_orientation(SplitterOrientation::Horizontal)
    // }

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
    ) -> InteractiveSplitterResponse<R> {
        self.show_dyn(ui, Box::new(add_contents))
    }

    pub fn show_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_contents: Box<dyn FnOnce(&mut Ui, &mut Ui) -> R + 'c>,
    ) -> InteractiveSplitterResponse<R> {
        let Self {
            orientation,
            ratio,
            resizable,
        } = self;

        let outer_clip = ui.clip_rect();

        debug_assert!((0.0..=1.0).contains(&ratio));

        let (rect, splitter_response) =
            ui.allocate_exact_size(ui.available_size_before_wrap(), Sense::hover());

        let id = ui.id();

        let mut ratio = ratio;
        if let Some(state) = InteractiveSplitState::load(ui.ctx(), id) {
            ratio = state.ratio;
        }

        // let line_pos_1 = match orientation {
        //     SplitterOrientation::Vertical => (lerp(rect.min.x..=rect.max.x, ratio), rect.min.y),
        //     SplitterOrientation::Horizontal => {
        //         (rect.min.x, lerp(rect.min.y..=rect.max.y, ratio))
        //     }
        // }.into();
        let line_pos_1 = orientation
            .t((
                lerp(orientation.p(rect.min)..=orientation.p(rect.max), ratio),
                orientation.rev().p(rect.min),
            ))
            .into();

        // let line_pos_2 = line_pos_1 + match orientation {
        //     SplitterOrientation::Vertical => (0.0, rect.height()),
        //     SplitterOrientation::Horizontal => (rect.width(), 0.0),
        // }.into();
        let line_pos_2 = line_pos_1 + orientation.t((0.0, orientation.r(&rect))).into();

        let line_pos_1 =
            egui::emath::GuiRounding::round_to_pixels(line_pos_1, ui.pixels_per_point());
        let line_pos_2 =
            egui::emath::GuiRounding::round_to_pixels(line_pos_2, ui.pixels_per_point());

        let i_spacing = &ui.style().spacing.item_spacing;
        // let top_left_rect = match orientation {
        //     SplitterOrientation::Vertical => {
        //         let mut rect = rect;
        //         rect.max.x = line_pos_1.x - i_spacing.x;
        //         rect
        //     }
        //     SplitterOrientation::Horizontal => {
        //         let mut rect = rect;
        //         rect.max.y = line_pos_1.y - i_spacing.y;
        //         rect
        //     }
        // };
        let first_rect = {
            let mut rect = rect;
            *orientation.m(&mut rect.max) = orientation.p(line_pos_1) - orientation.v(i_spacing);
            rect
        };

        // let second_rect = match orientation {
        //     SplitterOrientation::Vertical => {
        //         let mut rect = rect;
        //         rect.min.x = line_pos_1.x + i_spacing.x;
        //         rect
        //     }
        //     SplitterOrientation::Horizontal => {
        //         let mut rect = rect;
        //         rect.min.y = line_pos_1.y + i_spacing.y;
        //         rect
        //     }
        // };
        let second_rect = {
            let mut rect = rect;
            *orientation.m(&mut rect.min) = orientation.p(line_pos_1) + orientation.v(i_spacing);
            rect
        };

        let mut resize_hover = false;
        let mut is_resizing = false;
        if resizable {
            let resize_id = ui.id().with("__resize");
            if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                let we_are_on_top = ui
                    .ctx()
                    .layer_id_at(pointer)
                    .map_or(true, |top_layer_id| top_layer_id == ui.layer_id());
                let mouse_over_resize_line = we_are_on_top
                    && second_rect.y_range().contains(pointer.y)
                    && (line_pos_1.x - pointer.x).abs()
                        <= ui.style().interaction.resize_grab_radius_side;
                let mouse_in_clip_rect = ui.clip_rect().contains(pointer);
                if ui.input(|i| i.pointer.any_pressed() && i.pointer.any_down())
                    && mouse_over_resize_line
                    && mouse_in_clip_rect
                {
                    ui.ctx().set_dragged_id(resize_id);
                }
                is_resizing = ui.ctx().is_being_dragged(resize_id);
                if is_resizing {
                    // let width = (pointer.x - second_rect.left()).abs();
                    // let width =
                    //     clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
                    // second_rect.min.x = second_rect.max.x - width;
                    let x = pointer.x.clamp(rect.min.x, rect.max.x);
                    let f = x - first_rect.min.x;
                    ratio = (f / rect.width()).clamp(0.1, 0.9);
                }

                let dragging_something_else =
                    ui.input(|i| i.pointer.any_down() || i.pointer.any_pressed());
                resize_hover =
                    mouse_over_resize_line && !dragging_something_else && mouse_in_clip_rect;

                if resize_hover || is_resizing {
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                }
            }
        }

        let mut first_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(first_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        let mut second_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(second_rect)
                .layout(Layout::top_down(Align::Min)),
        );

        // panel_ui.expand_to_include_rect(panel_rect);
        // let frame = frame.unwrap_or_else(|| Frame::side_top_panel(ui.style()));
        // let inner_response = frame.show(&mut panel_ui, |ui| {
        //     ui.set_min_height(ui.max_rect().height()); // Make sure the frame fills the full height
        //     ui.set_min_width(*width_range.start());
        //     add_contents(ui)
        // });
        let frame = None;
        let frame = frame.unwrap_or_else(|| Frame::side_top_panel(ui.style()));
        let inner_response = frame.show(&mut first_ui, |first_ui| {
            // ui1.set_widthmin_height(ui1.max_rect().height()); // Make sure the frame fills the full height
            // ui1.set_min_width(*width_range.start());
            let inner_response = frame.show(&mut second_ui, |second_ui| {
                // ui2.set__height(ui2.max_rect().height()); // Make sure the frame fills the full height
                // ui2.set_min_width(*width_range.start());
                first_ui.set_clip_rect(first_rect.intersect(outer_clip));
                second_ui.set_clip_rect(second_rect.intersect(outer_clip));
                add_contents(first_ui, second_ui)
            });
            inner_response.inner
        });

        let body_returned = inner_response.inner;
        // let body_returned = add_contents(&mut first_ui, &mut second_ui);

        InteractiveSplitState { ratio }.store(ui.ctx(), ui.id());

        {
            let stroke = if is_resizing {
                ui.style().visuals.widgets.active.fg_stroke // highly visible
            } else if resize_hover {
                ui.style().visuals.widgets.hovered.fg_stroke // highly visible
            } else if true {
                //show_separator_line {
                // TOOD(emilk): distinguish resizable from non-resizable
                ui.style().visuals.widgets.noninteractive.bg_stroke // dim
            } else {
                Stroke::NONE
            };
            // TODO(emilk): draw line on top of all panels in this ui when https://github.com/emilk/egui/issues/1516 is done
            // In the meantime: nudge the line so its inside the panel, so it won't be covered by neighboring panel
            // (hence the shrink).
            // let resize_x = rect.shrink(1.0).left();
            // let resize_x = ui.painter().round_to_pixel(resize_x);
            // ui.painter().vline(resize_x, rect.y_range(), stroke);
            ui.painter().line_segment([line_pos_1, line_pos_2], stroke);
        }

        InteractiveSplitterResponse {
            splitter_response,
            body_returned,
            first_response: ui.interact(first_rect, first_ui.id(), Sense::hover()),
            second_response: ui.interact(second_rect, second_ui.id(), Sense::hover()),
        }
    }
}
