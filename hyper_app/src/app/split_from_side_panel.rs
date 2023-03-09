use std::ops::RangeInclusive;

use egui::{*};



/// State regarding panels.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SplitState {
    pub rect: Rect,
}

impl SplitState {
    pub fn load(ctx: &Context, bar_id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(bar_id))
    }

    /// The size of the panel (from previous frame).
    pub fn size(&self) -> Vec2 {
        self.rect.size()
    }

    fn store(self, ctx: &Context, bar_id: Id) {
        ctx.data_mut(|d| d.insert_persisted(bar_id, self));
    }
}

/// A panel that covers the entire left or right side of a [`Ui`] or screen.
///
/// The order in which you add panels matter!
/// The first panel you add will always be the outermost, and the last you add will always be the innermost.
///
/// ⚠ Always add any [`CentralPanel`] last.
///
/// See the [module level docs](crate::containers::panel) for more details.
///
/// ```
/// # egui::__run_test_ctx(|ctx| {
/// egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
///    ui.label("Hello World!");
/// });
/// # });
/// ```
///
/// See also [`TopBottomPanel`].
#[must_use = "You should call .show()"]
pub struct SplitPanel {
    id: Id,
    frame: Option<Frame>,
    resizable: bool,
    show_separator_line: bool,
    default_width: f32,
    width_range: RangeInclusive<f32>,
}

impl SplitPanel {

    /// The id should be globally unique, e.g. `Id::new("my_panel")`.
    pub fn new(id: impl Into<Id>) -> Self {
        Self {
            id: id.into(),
            frame: None,
            resizable: true,
            show_separator_line: true,
            default_width: 200.0,
            width_range: 96.0..=f32::INFINITY,
        }
    }

    /// Can panel be resized by dragging the edge of it?
    ///
    /// Default is `true`.
    ///
    /// If you want your panel to be resizable you also need a widget in it that
    /// takes up more space as you resize it, such as:
    /// * Wrapping text ([`Ui::horizontal_wrapped`]).
    /// * A [`ScrollArea`].
    /// * A [`Separator`].
    /// * A [`TextEdit`].
    /// * …
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Show a separator line, even when not interacting with it?
    ///
    /// Default: `true`.
    pub fn show_separator_line(mut self, show_separator_line: bool) -> Self {
        self.show_separator_line = show_separator_line;
        self
    }

    // /// The initial wrapping width of the [`SidePanel`].
    // pub fn default_width(mut self, default_width: f32) -> Self {
    //     self.default_width = default_width;
    //     self.width_range = self.width_range.start().at_most(default_width)
    //         ..=self.width_range.end().at_least(default_width);
    //     self
    // }

    // /// Minimum width of the panel.
    // pub fn min_width(mut self, min_width: f32) -> Self {
    //     self.width_range = min_width..=self.width_range.end().at_least(min_width);
    //     self
    // }

    // /// Maximum width of the panel.
    // pub fn max_width(mut self, max_width: f32) -> Self {
    //     self.width_range = self.width_range.start().at_most(max_width)..=max_width;
    //     self
    // }

    // /// The allowable width range for the panel.
    // pub fn width_range(mut self, width_range: RangeInclusive<f32>) -> Self {
    //     self.default_width = clamp_to_range(self.default_width, width_range.clone());
    //     self.width_range = width_range;
    //     self
    // }

    // /// Enforce this exact width.
    // pub fn exact_width(mut self, width: f32) -> Self {
    //     self.default_width = width;
    //     self.width_range = width..=width;
    //     self
    // }

    // /// Change the background color, margins, etc.
    // pub fn frame(mut self, frame: Frame) -> Self {
    //     self.frame = Some(frame);
    //     self
    // }
}

impl SplitPanel {
    /// Show the panel inside a [`Ui`].
    pub fn show_inside<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show_inside_dyn(ui, Box::new(add_contents))
    }

    /// Show the panel inside a [`Ui`].
    fn show_inside_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        let Self {
            id,
            frame,
            resizable,
            show_separator_line,
            default_width,
            width_range,
        } = self;

        let available_rect = ui.available_rect_before_wrap();
        let mut panel_rect = available_rect;
        {
            let mut width = default_width;
            if let Some(state) = SplitState::load(ui.ctx(), id) {
                width = state.rect.width();
            }
            width = clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
            side.set_rect_width(&mut panel_rect, width);
            ui.ctx().check_for_id_clash(id, panel_rect, "SidePanel");
        }

        let mut resize_hover = false;
        let mut is_resizing = false;
        if resizable {
            let resize_id = id.with("__resize");
            if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                let we_are_on_top = ui
                    .ctx()
                    .layer_id_at(pointer)
                    .map_or(true, |top_layer_id| top_layer_id == ui.layer_id());

                let resize_x = side.opposite().side_x(panel_rect);
                let mouse_over_resize_line = we_are_on_top
                    && panel_rect.y_range().contains(&pointer.y)
                    && (resize_x - pointer.x).abs()
                        <= ui.style().interaction.resize_grab_radius_side;

                if ui.input(|i| i.pointer.any_pressed() && i.pointer.any_down())
                    && mouse_over_resize_line
                {
                    ui.memory_mut(|mem| mem.set_dragged_id(resize_id));
                }
                is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));
                if is_resizing {
                    let width = (pointer.x - side.side_x(panel_rect)).abs();
                    let width =
                        clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
                    side.set_rect_width(&mut panel_rect, width);
                }

                let dragging_something_else =
                    ui.input(|i| i.pointer.any_down() || i.pointer.any_pressed());
                resize_hover = mouse_over_resize_line && !dragging_something_else;

                if resize_hover || is_resizing {
                    ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
                }
            }
        }

        let mut panel_ui = ui.child_ui_with_id_source(panel_rect, Layout::top_down(Align::Min), id);
        panel_ui.expand_to_include_rect(panel_rect);
        let frame = frame.unwrap_or_else(|| Frame::side_top_panel(ui.style()));
        let inner_response = frame.show(&mut panel_ui, |ui| {
            ui.set_min_height(ui.max_rect().height()); // Make sure the frame fills the full height
            ui.set_min_width(*width_range.start());
            add_contents(ui)
        });

        let rect = inner_response.response.rect;

        {
            let mut cursor = ui.cursor();
            match side {
                Side::Left => {
                    cursor.min.x = rect.max.x;
                }
                Side::Right => {
                    cursor.max.x = rect.min.x;
                }
            }
            ui.set_cursor(cursor);
        }
        ui.expand_to_include_rect(rect);

        SplitState { rect }.store(ui.ctx(), id);

        {
            let stroke = if is_resizing {
                ui.style().visuals.widgets.active.fg_stroke // highly visible
            } else if resize_hover {
                ui.style().visuals.widgets.hovered.fg_stroke // highly visible
            } else if show_separator_line {
                // TOOD(emilk): distinguish resizable from non-resizable
                ui.style().visuals.widgets.noninteractive.bg_stroke // dim
            } else {
                Stroke::NONE
            };
            // TODO(emilk): draw line on top of all panels in this ui when https://github.com/emilk/egui/issues/1516 is done
            // In the meantime: nudge the line so its inside the panel, so it won't be covered by neighboring panel
            // (hence the shrink).
            let resize_x = side.opposite().side_x(rect.shrink(1.0));
            let resize_x = ui.painter().round_to_pixel(resize_x);
            ui.painter().vline(resize_x, rect.y_range(), stroke);
        }

        inner_response
    }

    /// Show the panel at the top level.
    pub fn show<R>(
        self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show_dyn(ctx, Box::new(add_contents))
    }

    /// Show the panel at the top level.
    fn show_dyn<'c, R>(
        self,
        ctx: &Context,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        let layer_id = LayerId::background();
        let side = self.side;
        let available_rect = ctx.available_rect();
        let clip_rect = ctx.screen_rect();
        let mut panel_ui = Ui::new(ctx.clone(), layer_id, self.id, available_rect, clip_rect);

        let inner_response = self.show_inside_dyn(&mut panel_ui, add_contents);
        let rect = inner_response.response.rect;

        match side {
            Side::Left => ctx.frame_state_mut(|state| {
                state.allocate_left_panel(Rect::from_min_max(available_rect.min, rect.max));
            }),
            Side::Right => ctx.frame_state_mut(|state| {
                state.allocate_right_panel(Rect::from_min_max(rect.min, available_rect.max));
            }),
        }
        inner_response
    }

    /// Show the panel if `is_expanded` is `true`,
    /// otherwise don't show it, but with a nice animation between collapsed and expanded.
    pub fn show_animated<R>(
        self,
        ctx: &Context,
        is_expanded: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let how_expanded = ctx.animate_bool(self.id.with("animation"), is_expanded);

        if 0.0 == how_expanded {
            None
        } else if how_expanded < 1.0 {
            // Show a fake panel in this in-between animation state:
            // TODO(emilk): move the panel out-of-screen instead of changing its width.
            // Then we can actually paint it as it animates.
            let expanded_width = SplitState::load(ctx, self.id)
                .map_or(self.default_width, |state| state.rect.width());
            let fake_width = how_expanded * expanded_width;
            Self {
                id: self.id.with("animating_panel"),
                ..self
            }
            .resizable(false)
            .exact_width(fake_width)
            .show(ctx, |_ui| {});
            None
        } else {
            // Show the real panel:
            Some(self.show(ctx, add_contents))
        }
    }

    /// Show the panel if `is_expanded` is `true`,
    /// otherwise don't show it, but with a nice animation between collapsed and expanded.
    pub fn show_animated_inside<R>(
        self,
        ui: &mut Ui,
        is_expanded: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<InnerResponse<R>> {
        let how_expanded = ui
            .ctx()
            .animate_bool(self.id.with("animation"), is_expanded);

        if 0.0 == how_expanded {
            None
        } else if how_expanded < 1.0 {
            // Show a fake panel in this in-between animation state:
            // TODO(emilk): move the panel out-of-screen instead of changing its width.
            // Then we can actually paint it as it animates.
            let expanded_width = SplitState::load(ui.ctx(), self.id)
                .map_or(self.default_width, |state| state.rect.width());
            let fake_width = how_expanded * expanded_width;
            Self {
                id: self.id.with("animating_panel"),
                ..self
            }
            .resizable(false)
            .exact_width(fake_width)
            .show_inside(ui, |_ui| {});
            None
        } else {
            // Show the real panel:
            Some(self.show_inside(ui, add_contents))
        }
    }

    /// Show either a collapsed or a expanded panel, with a nice animation between.
    pub fn show_animated_between<R>(
        ctx: &Context,
        is_expanded: bool,
        collapsed_panel: Self,
        expanded_panel: Self,
        add_contents: impl FnOnce(&mut Ui, f32) -> R,
    ) -> Option<InnerResponse<R>> {
        let how_expanded = ctx.animate_bool(expanded_panel.id.with("animation"), is_expanded);

        if 0.0 == how_expanded {
            Some(collapsed_panel.show(ctx, |ui| add_contents(ui, how_expanded)))
        } else if how_expanded < 1.0 {
            // Show animation:
            let collapsed_width = SplitState::load(ctx, collapsed_panel.id)
                .map_or(collapsed_panel.default_width, |state| state.rect.width());
            let expanded_width = SplitState::load(ctx, expanded_panel.id)
                .map_or(expanded_panel.default_width, |state| state.rect.width());
            let fake_width = lerp(collapsed_width..=expanded_width, how_expanded);
            Self {
                id: expanded_panel.id.with("animating_panel"),
                ..expanded_panel
            }
            .resizable(false)
            .exact_width(fake_width)
            .show(ctx, |ui| add_contents(ui, how_expanded));
            None
        } else {
            Some(expanded_panel.show(ctx, |ui| add_contents(ui, how_expanded)))
        }
    }

    /// Show either a collapsed or a expanded panel, with a nice animation between.
    pub fn show_animated_between_inside<R>(
        ui: &mut Ui,
        is_expanded: bool,
        collapsed_panel: Self,
        expanded_panel: Self,
        add_contents: impl FnOnce(&mut Ui, f32) -> R,
    ) -> InnerResponse<R> {
        let how_expanded = ui
            .ctx()
            .animate_bool(expanded_panel.id.with("animation"), is_expanded);

        if 0.0 == how_expanded {
            collapsed_panel.show_inside(ui, |ui| add_contents(ui, how_expanded))
        } else if how_expanded < 1.0 {
            // Show animation:
            let collapsed_width = SplitState::load(ui.ctx(), collapsed_panel.id)
                .map_or(collapsed_panel.default_width, |state| state.rect.width());
            let expanded_width = SplitState::load(ui.ctx(), expanded_panel.id)
                .map_or(expanded_panel.default_width, |state| state.rect.width());
            let fake_width = lerp(collapsed_width..=expanded_width, how_expanded);
            Self {
                id: expanded_panel.id.with("animating_panel"),
                ..expanded_panel
            }
            .resizable(false)
            .exact_width(fake_width)
            .show_inside(ui, |ui| add_contents(ui, how_expanded))
        } else {
            expanded_panel.show_inside(ui, |ui| add_contents(ui, how_expanded))
        }
    }
}


fn clamp_to_range(x: f32, range: RangeInclusive<f32>) -> f32 {
    x.clamp(
        range.start().min(*range.end()),
        range.start().max(*range.end()),
    )
}