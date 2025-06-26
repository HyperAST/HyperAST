use std::time::Instant;

use crossbeam::channel::{Receiver, Sender, unbounded};
use drawers::ValuesSectionDebug;
use eframe::{App, CreationContext};
use egui::{CollapsingHeader, Context, Pos2, ScrollArea, Ui, Vec2};
use egui_graphs::events::Event;
use egui_graphs::{Edge, Graph, Node};
use fdg::fruchterman_reingold::{FruchtermanReingold, FruchtermanReingoldConfiguration};
use fdg::nalgebra::{Const, OPoint};
use fdg::{Force, ForceGraph};
use petgraph::Directed;
use petgraph::stable_graph::{DefaultIx, NodeIndex};
use petgraph::visit::IntoNodeReferences;

pub mod drawers {
    use egui::Ui;

    pub struct ValuesConfigButtonsStartReset {
        pub simulation_stopped: bool,
    }

    pub fn draw_start_reset_buttons(
        ui: &mut egui::Ui,
        mut values: ValuesConfigButtonsStartReset,
    ) -> (bool, bool) {
        ui.vertical(|ui| {
            ui.label("Stop or start simulation again or reset to default settings.");
            ui.horizontal(|ui| {
                let start_simulation_stopped = values.simulation_stopped;
                if ui
                    .button(match values.simulation_stopped {
                        true => "start",
                        false => "stop",
                    })
                    .clicked()
                {
                    values.simulation_stopped = !values.simulation_stopped;
                };

                let mut reset_pressed = false;
                if ui.button("reset").clicked() {
                    reset_pressed = true;
                }

                if start_simulation_stopped != values.simulation_stopped || reset_pressed {
                    (values.simulation_stopped, reset_pressed)
                } else {
                    (false, false)
                }
            })
            .inner
        })
        .inner
    }

    pub struct ValuesSectionDebug {
        pub zoom: f32,
        pub pan: [f32; 2],
        pub fps: f32,
    }

    pub fn draw_section_debug(ui: &mut egui::Ui, values: ValuesSectionDebug) {
        ui.label(format!("zoom: {:.5}", values.zoom));
        ui.label(format!("pan: [{:.5}, {:.5}]", values.pan[0], values.pan[1]));
        ui.label(format!("FPS: {:.1}", values.fps));
    }

    pub struct ValuesConfigSlidersSimulation {
        pub dt: f32,
        pub cooloff_factor: f32,
        pub scale: f32,
    }

    pub fn draw_simulation_config_sliders(
        ui: &mut Ui,
        mut values: ValuesConfigSlidersSimulation,
        mut on_change: impl FnMut(f32, f32, f32),
    ) {
        let start_dt = values.dt;
        let mut delta_dt = 0.;
        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut values.dt, 0.00..=0.7).text("dt"))
                .changed()
            {
                delta_dt = values.dt - start_dt;
            };
        });

        let start_cooloff_factor = values.cooloff_factor;
        let mut delta_cooloff_factor = 0.;
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Slider::new(&mut values.cooloff_factor, 0.00..=1.).text("cooloff_factor"),
                )
                .changed()
            {
                delta_cooloff_factor = values.cooloff_factor - start_cooloff_factor;
            };
        });

        let start_scale = values.scale;
        let mut delta_scale = 0.;
        ui.horizontal(|ui| {
            if ui
                .add(egui::Slider::new(&mut values.scale, 1.0..=1000.).text("scale"))
                .changed()
            {
                delta_scale = values.scale - start_scale;
            };
        });

        if delta_dt != 0. || delta_cooloff_factor != 0. || delta_scale != 0. {
            on_change(delta_dt, delta_cooloff_factor, delta_scale);
        }
    }
}

pub mod settings {
    use super::*;
    pub struct SettingsGraph {
        // WIP
        weight_multiplier: f32,
    }

    impl Default for SettingsGraph {
        fn default() -> Self {
            Self {
                weight_multiplier: 1.0,
            }
        }
    }

    impl SettingsGraph {
        pub fn show(&mut self, ui: &mut Ui) {
            CollapsingHeader::new("Graph")
                .default_open(false)
                .show(ui, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.weight_multiplier, 0.00..=10.)
                            .text("weight mult"),
                    );
                });
        }
    }

    pub struct SettingsInteraction {
        pub dragging_enabled: bool,
        pub node_clicking_enabled: bool,
        pub node_selection_enabled: bool,
        pub node_selection_multi_enabled: bool,
        pub edge_clicking_enabled: bool,
        pub edge_selection_enabled: bool,
        pub edge_selection_multi_enabled: bool,
    }

    impl Default for SettingsInteraction {
        fn default() -> Self {
            Self {
                dragging_enabled: true,
                node_clicking_enabled: false,
                node_selection_enabled: true,
                node_selection_multi_enabled: false,
                edge_clicking_enabled: false,
                edge_selection_enabled: false,
                edge_selection_multi_enabled: false,
            }
        }
    }

    impl SettingsInteraction {
        pub fn show(&mut self, ui: &mut Ui) {
            CollapsingHeader::new("Interaction").show(ui, |ui| {
                    if ui.checkbox(&mut self.dragging_enabled, "dragging_enabled").clicked() && self.dragging_enabled {
                        self.node_clicking_enabled = true;
                    };
                    ui.label("To drag use LMB click + drag on a node.");

                    ui.add_space(5.);

                    ui.add_enabled_ui(!(self.dragging_enabled || self.node_selection_enabled || self.node_selection_multi_enabled), |ui| {
                        ui.vertical(|ui| {
                            ui.checkbox(&mut self.node_clicking_enabled, "node_clicking_enabled");
                            ui.label("Check click events in last events");
                        }).response.on_disabled_hover_text("node click is enabled when any of the interaction is also enabled");
                    });

                    ui.add_space(5.);

                    ui.add_enabled_ui(!self.node_selection_multi_enabled, |ui| {
                        ui.vertical(|ui| {
                            if ui.checkbox(&mut self.node_selection_enabled, "node_selection_enabled").clicked() && self.node_selection_enabled {
                                self.node_clicking_enabled = true;
                            };
                            ui.label("Enable select to select nodes with LMB click. If node is selected clicking on it again will deselect it.");
                        }).response.on_disabled_hover_text("node_selection_multi_enabled enables select");
                    });

                    if ui.checkbox(&mut self.node_selection_multi_enabled, "node_selection_multi_enabled").changed() && self.node_selection_multi_enabled {
                        self.node_clicking_enabled = true;
                        self.node_selection_enabled = true;
                    }
                    ui.label("Enable multiselect to select multiple nodes.");

                    ui.add_space(5.);

                    ui.add_enabled_ui(!(self.edge_selection_enabled || self.edge_selection_multi_enabled), |ui| {
                        ui.vertical(|ui| {
                            ui.checkbox(&mut self.edge_clicking_enabled, "edge_clicking_enabled");
                            ui.label("Check click events in last events");
                        }).response.on_disabled_hover_text("edge click is enabled when any of the interaction is also enabled");
                    });

                    ui.add_space(5.);

                    ui.add_enabled_ui(!self.edge_selection_multi_enabled, |ui| {
                        ui.vertical(|ui| {
                            if ui.checkbox(&mut self.edge_selection_enabled, "edge_selection_enabled").clicked() && self.edge_selection_enabled {
                                self.edge_clicking_enabled = true;
                            };
                            ui.label("Enable select to select edges with LMB click. If edge is selected clicking on it again will deselect it.");
                        }).response.on_disabled_hover_text("edge_selection_multi_enabled enables select");
                    });

                    if ui.checkbox(&mut self.edge_selection_multi_enabled, "edge_selection_multi_enabled").changed() && self.edge_selection_multi_enabled {
                        self.edge_clicking_enabled = true;
                        self.edge_selection_enabled = true;
                    }
                    ui.label("Enable multiselect to select multiple edges.");
                });
        }
    }

    pub struct SettingsNavigation {
        pub fit_to_screen_enabled: bool,
        pub zoom_and_pan_enabled: bool,
        pub zoom_speed: f32,
    }

    impl Default for SettingsNavigation {
        fn default() -> Self {
            Self {
                zoom_speed: 0.05,
                fit_to_screen_enabled: true,
                zoom_and_pan_enabled: true,
            }
        }
    }

    impl SettingsNavigation {
        pub fn show(&mut self, ui: &mut Ui) {
            CollapsingHeader::new("Navigation")
                .default_open(true)
                .show(ui, |ui| {
                    if ui
                        .checkbox(&mut self.fit_to_screen_enabled, "fit_to_screen")
                        .changed()
                        && self.fit_to_screen_enabled
                    {
                        self.zoom_and_pan_enabled = false
                    };
                    ui.label("Enable fit to screen to fit the graph to the screen on every frame.");

                    ui.add_space(5.);

                    ui.add_enabled_ui(!self.fit_to_screen_enabled, |ui| {
                        ui.vertical(|ui| {
                            ui.checkbox(&mut self.zoom_and_pan_enabled, "zoom_and_pan");
                            ui.label("Zoom with ctrl + mouse wheel, pan with middle mouse drag.");
                        })
                        .response
                        .on_disabled_hover_text("disable fit_to_screen to enable zoom_and_pan");
                    });
                    ui.add_space(5.);

                    // let zoom_speed = self.zoom_speed;
                    // let mut dt =
                    // let mut delta_dt = 0.;
                    ui.horizontal(|ui| {
                        if ui
                            .add(
                                egui::Slider::new(&mut self.zoom_speed, 0.00..=1.)
                                    .text("zoom speed"),
                            )
                            .changed()
                        {
                            // delta_dt = values.dt - zoom_speed;
                        };
                    });
                });
        }
    }

    #[derive(Default)]
    pub struct SettingsStyle {
        pub labels_always: bool,
    }

    impl SettingsStyle {
        pub fn show(&mut self, ui: &mut Ui) {
            CollapsingHeader::new("Style").show(ui, |ui| {
                ui.checkbox(&mut self.labels_always, "labels_always");
                ui.label("Wheter to show labels always or when interacted only.");
            });
        }
    }

    pub struct SettingsSimulation {
        pub dt: f32,
        pub cooloff_factor: f32,
        pub scale: f32,
    }

    impl Default for SettingsSimulation {
        fn default() -> Self {
            Self {
                dt: 0.0001,
                // dt: 0.03,
                cooloff_factor: 0.85,
                scale: 1000.,
            }
        }
    }
}

const EVENTS_LIMIT: usize = 100;

pub struct ForceBasedGraphExplorationApp<
    N: Clone = (),
    E: Clone = (),
    Dn: egui_graphs::DisplayNode<N, E, Directed, u32> = egui_graphs::DefaultNodeShape,
> {
    g: Graph<N, E, Directed, DefaultIx, Dn>,
    sim: ForceGraph<
        f32,
        2,
        Node<N, E, Directed, DefaultIx, Dn>,
        Edge<N, E, Directed, DefaultIx, Dn>,
    >,
    force: FruchtermanReingold<f32, 2>,

    others: Vec<
        Option<(
            Graph<N, E, Directed, DefaultIx, Dn>,
            ForceGraph<
                f32,
                2,
                Node<N, E, Directed, DefaultIx, Dn>,
                Edge<N, E, Directed, DefaultIx, Dn>,
            >,
            FruchtermanReingold<f32, 2>,
        )>,
    >,
    /// the active graph in the `others` vec
    active: usize,

    can_show_graph: bool,
    show_graph: bool,
    show_pattern_list: bool,

    global: Global,
}

struct Global {
    settings_simulation: settings::SettingsSimulation,

    settings_graph: settings::SettingsGraph,
    settings_interaction: settings::SettingsInteraction,
    settings_navigation: settings::SettingsNavigation,
    settings_style: settings::SettingsStyle,

    last_events: Vec<String>,

    simulation_stopped: bool,

    fps: f32,
    last_update_time: Instant,
    frames_last_time_span: usize,

    event_publisher: Sender<Event>,
    event_consumer: Receiver<Event>,

    pan: [f32; 2],
    zoom: f32,
}

pub fn graph_pretty<'d, N: 'd + Clone + std::fmt::Debug + std::fmt::Display, E: 'd + Clone>(
    cc: &CreationContext<'_>,
    settings_graph: settings::SettingsGraph,
    settings_simulation: settings::SettingsSimulation,
    g: Graph<N, E, Directed, DefaultIx, node::NodeShapeFlex<N>>,
) -> impl App + 'd {
    ForceBasedGraphExplorationApp::<N, E, node::NodeShapeFlex<N>>::with_graph(
        cc,
        settings_graph,
        settings_simulation,
        g,
    )
}

pub fn multi_graph_pretty<
    'a,
    N: 'a + Clone + std::fmt::Debug + std::fmt::Display,
    E: 'a + Clone,
>(
    _cc: &CreationContext<'_>,
    settings_graph: settings::SettingsGraph,
    settings_simulation: settings::SettingsSimulation,
    graph: Vec<Graph<N, E, Directed, DefaultIx, node::NodeShapeFlex<N>>>,
) -> impl App + 'a {
    let mut others: Vec<Option<_>> = graph
        .into_iter()
        .map(|mut g| {
            let (force, sim) = force_sim(&settings_simulation, &mut g);
            Some((g, sim, force))
        })
        .collect();
    let (g, sim, force) = others[0].take().unwrap();
    let mut app = ForceBasedGraphExplorationApp::<N, E, _>::new(
        settings_graph,
        settings_simulation,
        g,
        force,
        sim,
    );
    app.others = others;
    app
}

impl<N: Clone, E: Clone, Dn: egui_graphs::DisplayNode<N, E, Directed, DefaultIx>>
    ForceBasedGraphExplorationApp<N, E, Dn>
{
    fn show_graph(&mut self, ui: &mut Ui) {
        let settings_interaction = &egui_graphs::SettingsInteraction::new()
            .with_node_selection_enabled(self.global.settings_interaction.node_selection_enabled)
            .with_node_selection_multi_enabled(
                self.global
                    .settings_interaction
                    .node_selection_multi_enabled,
            )
            .with_dragging_enabled(self.global.settings_interaction.dragging_enabled)
            .with_node_clicking_enabled(self.global.settings_interaction.node_clicking_enabled)
            .with_edge_clicking_enabled(self.global.settings_interaction.edge_clicking_enabled)
            .with_edge_selection_enabled(self.global.settings_interaction.edge_selection_enabled)
            .with_edge_selection_multi_enabled(
                self.global
                    .settings_interaction
                    .edge_selection_multi_enabled,
            );
        let settings_navigation = &egui_graphs::SettingsNavigation::new()
            .with_zoom_and_pan_enabled(self.global.settings_navigation.zoom_and_pan_enabled)
            .with_fit_to_screen_enabled(self.global.settings_navigation.fit_to_screen_enabled)
            .with_zoom_speed(self.global.settings_navigation.zoom_speed);
        let settings_style = &egui_graphs::SettingsStyle::new()
            .with_labels_always(self.global.settings_style.labels_always);

        // let mut md = egui_graphs::Metadata::load(ui);
        // md.reset_bounds();

        ui.add(
            &mut egui_graphs::GraphView::<N, E, _, _, _, _, _, egui_graphs::LayoutHierarchical>::new(
                &mut self.g,
            )
            .with_interactions(settings_interaction)
            .with_navigations(settings_navigation)
            .with_styles(settings_style)
            .with_events(&self.global.event_publisher),
        );
    }
}

impl<N: Clone + std::fmt::Debug, E: Clone, Dn: egui_graphs::DisplayNode<N, E, Directed, DefaultIx>>
    ForceBasedGraphExplorationApp<N, E, Dn>
{
    pub fn with_graph(
        _: &CreationContext<'_>,
        settings_graph: settings::SettingsGraph,
        settings_simulation: settings::SettingsSimulation,
        mut g: Graph<N, E, Directed, DefaultIx, Dn>,
    ) -> Self {
        let (force, sim) = force_sim(&settings_simulation, &mut g);

        Self::new(settings_graph, settings_simulation, g, force, sim)
    }

    fn new(
        settings_graph: settings::SettingsGraph,
        settings_simulation: settings::SettingsSimulation,
        g: Graph<N, E, Directed, u32, Dn>,
        force: FruchtermanReingold<f32, 2>,
        sim: petgraph::prelude::StableGraph<
            (Node<N, E, Directed, u32, Dn>, OPoint<f32, Const<2>>),
            Edge<N, E, Directed, u32, Dn>,
        >,
    ) -> Self {
        let (event_publisher, event_consumer) = unbounded();

        Self {
            g,
            sim,
            force,

            others: vec![],
            active: 0,
            can_show_graph: true,
            show_graph: true,
            show_pattern_list: false,

            global: Global {
                event_consumer,
                event_publisher,

                settings_graph,
                settings_simulation,

                settings_interaction: settings::SettingsInteraction::default(),
                settings_navigation: settings::SettingsNavigation::default(),
                settings_style: settings::SettingsStyle::default(),

                last_events: Vec::default(),

                simulation_stopped: false,

                fps: 0.,
                last_update_time: Instant::now(),
                frames_last_time_span: 0,

                pan: [0., 0.],
                zoom: 10.,
            },
        }
    }

    /// applies forces if simulation is running
    fn update_simulation(&mut self) {
        if self.global.simulation_stopped {
            return;
        }
        if self.can_show_graph {
            self.force.apply(&mut self.sim);
        }
    }

    /// sync locations computed by the simulation with egui_graphs::Graph nodes.
    fn sync(&mut self) {
        self.g.g.node_weights_mut().for_each(|node| {
            let sim_computed_point: OPoint<f32, Const<2>> =
                self.sim.node_weight(node.id()).unwrap().1;
            node.set_location(Pos2::new(
                sim_computed_point.coords.x,
                sim_computed_point.coords.y,
            ));
        });
    }

    fn handle_events(&mut self) {
        self.global.event_consumer.try_iter().for_each(|e| {
            if self.global.last_events.len() > EVENTS_LIMIT {
                self.global.last_events.remove(0);
            }
            self.global.last_events.push(format!("{e:?}"));
            // .push(serde_json::to_string(&e).unwrap());

            match e {
                Event::NodeDoubleClick(payload) => {
                    let node_id = NodeIndex::new(payload.id);
                    self.g.remove_node(node_id);
                    self.sim.remove_node(node_id);
                }
                Event::Pan(payload) => self.global.pan = payload.new_pan,
                Event::Zoom(payload) => {
                    if !payload.new_zoom.is_nan() {
                        self.global.zoom = payload.new_zoom; //.clamp(0.03, 20.0);
                    }
                }
                Event::NodeMove(payload) => {
                    let node_id = NodeIndex::new(payload.id);

                    self.sim.node_weight_mut(node_id).unwrap().1.coords.x = payload.new_pos[0];
                    self.sim.node_weight_mut(node_id).unwrap().1.coords.y = payload.new_pos[1];
                }
                _ => {}
            }
        });
    }

    fn draw_section_simulation(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(0., 0.);
            ui.label("Force-Directed Simulation is done with ");
            ui.hyperlink_to("fdg project", "https://github.com/grantshandy/fdg");
        });

        ui.separator();

        let (simulation_stopped, reset) = drawers::draw_start_reset_buttons(
            ui,
            drawers::ValuesConfigButtonsStartReset {
                simulation_stopped: self.global.simulation_stopped,
            },
        );
        if simulation_stopped || reset {
            self.global.simulation_stopped = simulation_stopped;
        }
        if reset {
            dbg!();
            let mut md = egui_graphs::Metadata::load(ui);
            dbg!(md.graph_bounds());
            dbg!(md.zoom);
            // md.zoom = 1.5;
            md.reset_bounds();
            for n in self.sim.node_weights_mut() {
                dbg!(&n.1);
                n.1.x = 1.0;
                n.1.y = 1.0;
            }
            for n in self.g.nodes_iter() {
                dbg!(&n.1.location());
                md.comp_iter_bounds(n.1);
            }
            for n in self.g.g.node_weights_mut() {
                dbg!(&n.location());
                n.set_location(Pos2::new(1.0, 1.0));
                md.comp_iter_bounds(n);
            }
            dbg!(md.graph_bounds());
            md.save(ui);
            self.reset();
            egui_graphs::GraphView::<
                N,
                E,
                _,
                _,
                Dn,
                egui_graphs::DefaultEdgeShape,
                egui_graphs::LayoutStateRandom,
                egui_graphs::LayoutRandom,
            >::clear_cache(ui);
        }

        ui.add_space(10.);

        drawers::draw_simulation_config_sliders(
            ui,
            drawers::ValuesConfigSlidersSimulation {
                dt: self.global.settings_simulation.dt,
                cooloff_factor: self.global.settings_simulation.cooloff_factor,
                scale: self.global.settings_simulation.scale,
            },
            |delta_dt: f32, delta_cooloff_factor: f32, delta_scale: f32| {
                let s = &mut self.global.settings_simulation;
                s.dt += delta_dt;
                s.cooloff_factor += delta_cooloff_factor;
                s.scale += delta_scale;
                self.force = init_force(&s);
            },
        );

        ui.add_space(10.);
    }

    fn draw_section_widget(&mut self, ui: &mut Ui) {
        self.global.settings_navigation.show(ui);
        self.global.settings_graph.show(ui);
        self.global.settings_style.show(ui);
        self.global.settings_interaction.show(ui);
        self.draw_selected_widget(ui);
        self.global.draw_last_event_widget(ui);
    }

    fn draw_selected_widget(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Selected")
            .default_open(true)
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(200.)
                    .show(ui, |ui| {
                        let rm: Vec<_> = self
                            .g
                            .selected_nodes()
                            .iter()
                            .filter_map(|node| {
                                let clicked = ui.button("remove").clicked();
                                ui.label(format!("{node:?}"));
                                let p = &self.g.node(*node).unwrap().payload();
                                ui.label(format!("{p:#?}"));
                                clicked.then_some(*node)
                            })
                            .collect();
                        for idx in rm {
                            self.g.remove_node(idx);
                            self.sim.remove_node(idx);
                        }
                        self.g.selected_edges().iter().for_each(|edge| {
                            ui.label(format!("{edge:?}"));
                        });
                    });
            });
    }

    fn draw_section_debug(&self, ui: &mut Ui) {
        drawers::draw_section_debug(
            ui,
            ValuesSectionDebug {
                zoom: self.global.zoom,
                pan: self.global.pan,
                fps: self.global.fps,
            },
        );
    }

    fn reset(&mut self) {
        self.global.zoom = 10.0;
        self.global.pan = [0., 0.];
        self.global.settings_navigation.fit_to_screen_enabled = false;
        self.global.settings_simulation.dt = 0.001;
        self.force = init_force(&self.global.settings_simulation);
        self.sim = fdg::init_force_graph_uniform(self.g.g.clone(), 1.0);
    }
}

fn force_sim<N: Clone, E: Clone, Dn: egui_graphs::DisplayNode<N, E, Directed, DefaultIx>>(
    settings_simulation: &settings::SettingsSimulation,
    g: &mut Graph<N, E, Directed, u32, Dn>,
) -> (
    FruchtermanReingold<f32, 2>,
    petgraph::prelude::StableGraph<
        (Node<N, E, Directed, u32, Dn>, OPoint<f32, Const<2>>),
        Edge<N, E, Directed, u32, Dn>,
    >,
) {
    let mut force = init_force(settings_simulation);
    let mut sim = fdg::init_force_graph_uniform(g.g.clone(), 1.0);
    force.apply(&mut sim);
    g.g.node_weights_mut().for_each(|node| {
        let point: fdg::nalgebra::OPoint<f32, fdg::nalgebra::Const<2>> =
            sim.node_weight(node.id()).unwrap().1;
        node.set_location(Pos2::new(point.coords.x, point.coords.y));
    });
    (force, sim)
}

impl Global {
    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    fn draw_last_event_widget(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Last Events")
            .default_open(true)
            .show(ui, |ui| {
                if ui.button("clear").clicked() {
                    self.last_events.clear();
                }
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        self.last_events.iter().rev().for_each(|event| {
                            ui.label(event);
                        });
                    });
            });
    }
}

impl<
    N: Clone + std::fmt::Debug + std::fmt::Display,
    E: Clone,
    Dn: egui_graphs::DisplayNode<N, E, Directed, u32>,
> App for ForceBasedGraphExplorationApp<N, E, Dn>
{
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        egui::SidePanel::right("right_panel")
            .min_width(250.)
            .show(ctx, |ui| {
                if !self.others.is_empty() {
                    ui.horizontal(|ui| {
                        let left = egui::Button::new("<");
                        let left = ui.add_enabled(self.active != 0, left).clicked();
                        ui.label(format!("{}", self.active));
                        let right = egui::Button::new(">");
                        let right = ui
                            .add_enabled(self.active != self.others.len() - 1, right)
                            .clicked();
                        if left {
                            self.others.swap(self.active, self.active - 1);
                            let new = &mut self.others[self.active];
                            self.active -= 1;
                            let Some(new) = new else { unreachable!() };
                            std::mem::swap(&mut new.0, &mut self.g);
                            std::mem::swap(&mut new.1, &mut self.sim);
                            std::mem::swap(&mut new.2, &mut self.force);
                            // self.force = init_force(&self.global.settings_simulation);
                        } else if right {
                            self.others.swap(self.active, self.active + 1);
                            let new = &mut self.others[self.active];
                            self.active += 1;
                            let Some(new) = new else { unreachable!() };
                            std::mem::swap(&mut new.0, &mut self.g);
                            std::mem::swap(&mut new.1, &mut self.sim);
                            std::mem::swap(&mut new.2, &mut self.force);
                            if self.g.node_count() < 300 {
                                self.can_show_graph = true;
                            }
                            // self.force = init_force(&self.global.settings_simulation);
                        }

                        if left || right {
                            egui_graphs::Metadata::default().save(ui);
                        }

                        ui.add_enabled(
                            self.can_show_graph,
                            egui::Checkbox::new(&mut self.show_graph, "graph"),
                        );
                        ui.checkbox(&mut self.show_pattern_list, "list");
                    });

                    ui.label(format!(
                        "{} connex graph n:{} e:{}",
                        self.others.len(),
                        self.g.node_count(),
                        self.g.edge_count()
                    ));
                }
                ScrollArea::vertical().show(ui, |ui| {
                    CollapsingHeader::new("Simulation")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_simulation(ui));

                    ui.add_space(10.);

                    egui::CollapsingHeader::new("Debug")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_debug(ui));

                    ui.add_space(10.);

                    CollapsingHeader::new("Widget")
                        .default_open(true)
                        .show(ui, |ui| self.draw_section_widget(ui));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_pattern_list {
                if !self.can_show_graph {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} nodes and {} edges is too much to display",
                            self.g.node_count(),
                            self.g.edge_count()
                        ))
                        .color(ui.visuals().warn_fg_color),
                    );
                }

                let size = if self.can_show_graph && self.show_graph {
                    ui.available_size() * egui::vec2(1.0, 0.2)
                } else {
                    ui.available_size()
                };
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::click());
                let ui = &mut ui.new_child(egui::UiBuilder::new().max_rect(rect));
                let total_cols = self.g.node_count();
                let mut rm = None;
                let mut nodes = None;
                crate::hscroll::hscroll_many_columns(ui, 450.0, total_cols, |ui, i| {
                    if nodes.is_none() {
                        nodes = Some(self.g.g.node_references().rev().skip(i));
                    };
                    let nodes = nodes.as_mut().unwrap();
                    let (idx, n) = nodes.next().unwrap();
                    let payload = n.payload();
                    let clicked = ui
                        .horizontal(|ui| {
                            let clicked = ui.button("remove").clicked();
                            let outg = self
                                .g
                                .edges_directed(idx, petgraph::Direction::Outgoing)
                                .count();
                            let inco = self
                                .g
                                .edges_directed(idx, petgraph::Direction::Incoming)
                                .count();
                            ui.label(format!("#:{payload:?} out:{outg:#} in:{inco:#}"));
                            clicked
                        })
                        .inner;
                    ui.label(format!("{:#}", payload));
                    if clicked {
                        rm = Some(idx)
                    }
                });
                drop(nodes);

                if let Some(idx) = rm {
                    self.g.remove_node(idx);
                    self.sim.remove_node(idx);
                }
            }
            if self.can_show_graph && self.show_graph {
                self.show_graph(ui);
            }
        });

        self.handle_events();
        self.sync();
        self.update_simulation();
        self.global.update_fps();
    }
}
mod node {
    use egui::{Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, Vec2, epaint::TextShape};
    use egui_graphs::{DisplayNode, NodeProps};
    use petgraph::{EdgeType, stable_graph::IndexType};

    #[derive(Clone)]
    pub struct NodeShapeFlex<N> {
        label: N,
        loc: Pos2,

        size_x: f32,
        size_y: f32,
    }
    impl<N: Clone + std::fmt::Debug + std::fmt::Display> From<NodeProps<N>> for NodeShapeFlex<N> {
        fn from(node_props: NodeProps<N>) -> Self {
            Self {
                loc: node_props.location(),
                label: node_props.payload.clone(),

                size_x: 0.,
                size_y: 0.,
            }
        }
    }

    impl<N: Clone + std::fmt::Debug + std::fmt::Display, E: Clone, Ty: EdgeType, Ix: IndexType>
        DisplayNode<N, E, Ty, Ix> for NodeShapeFlex<N>
    {
        fn is_inside(&self, pos: Pos2) -> bool {
            let rect = Rect::from_center_size(self.loc, Vec2::new(self.size_x, self.size_y));

            rect.contains(pos)
        }

        fn closest_boundary_point(&self, dir: Vec2) -> Pos2 {
            find_intersection(self.loc, self.size_x / 2., self.size_y / 2., dir)
        }

        fn shapes(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape> {
            // find node center location on the screen coordinates
            let center = ctx.meta.canvas_to_screen_pos(self.loc);
            let color = ctx.ctx.style().visuals.text_color();

            // create label
            let galley = ctx.ctx.fonts(|f| {
                f.layout_no_wrap(
                    format!("{}", self.label),
                    FontId::new(ctx.meta.canvas_to_screen_size(10.), FontFamily::Monospace),
                    color,
                )
            });

            // we need to offset label by half its size to place it in the center of the rect
            let offset = Vec2::new(-galley.size().x / 2., -galley.size().y / 2.);

            // create the shape and add it to the layers
            let shape_label = TextShape::new(center + offset, galley, color);

            let rect = shape_label.visual_bounding_rect();
            let points = rect_to_points(rect);
            let shape_rect =
                Shape::convex_polygon(points, Color32::default(), Stroke::new(1., color));

            // update self size
            self.size_x = rect.size().x;
            self.size_y = rect.size().y;

            vec![shape_rect, shape_label.into()]
        }

        fn update(&mut self, state: &NodeProps<N>) {
            self.label = state.payload.clone();
            self.loc = state.location();
        }
    }

    fn find_intersection(center: Pos2, size_x: f32, size_y: f32, direction: Vec2) -> Pos2 {
        if (direction.x.abs() * size_y) > (direction.y.abs() * size_x) {
            // intersects left or right side
            let x = if direction.x > 0.0 {
                center.x + size_x / 2.0
            } else {
                center.x - size_x / 2.0
            };
            let y = center.y + direction.y / direction.x * (x - center.x);
            Pos2::new(x, y)
        } else {
            // intersects top or bottom side
            let y = if direction.y > 0.0 {
                center.y + size_y / 2.0
            } else {
                center.y - size_y / 2.0
            };
            let x = center.x + direction.x / direction.y * (y - center.y);
            Pos2::new(x, y)
        }
    }

    fn rect_to_points(rect: Rect) -> Vec<Pos2> {
        let top_left = rect.min;
        let bottom_right = rect.max;
        let top_right = Pos2::new(bottom_right.x, top_left.y);
        let bottom_left = Pos2::new(top_left.x, bottom_right.y);

        vec![top_left, top_right, bottom_right, bottom_left]
    }
}

fn init_force(settings: &settings::SettingsSimulation) -> FruchtermanReingold<f32, 2> {
    FruchtermanReingold {
        conf: FruchtermanReingoldConfiguration {
            dt: settings.dt,
            cooloff_factor: settings.cooloff_factor,
            scale: settings.scale,
        },
        ..Default::default()
    }
}
