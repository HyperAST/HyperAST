use std::fs;
use std::ops::Deref;

use std::collections::HashMap;

use derive_deref::Deref;
use fdg_img::style::text_anchor::{HPos, Pos, VPos};
use fdg_img::style::{Color, IntoFont, RGBAColor, ShapeStyle, TextStyle, BLACK};
use fdg_img::Settings;
use fdg_sim::petgraph::stable_graph::StableGraph;
use fdg_sim::petgraph::visit::{EdgeRef, IntoEdgeReferences};
use fdg_sim::petgraph::Undirected;
use fdg_sim::Dimensions;
use fdg_sim::{force, glam::Vec3, ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use plotters::prelude::{Circle, IntoDrawingArea, PathElement, Rectangle, SVGBackend};
use serde::Serialize;

use crate::preprocess::{
    DChildren, Fields, MultipleChildren, RequiredChildren, Role, SubTypes, TypeSys, T,
};

trait GStyle: Into<macroquad::color::Color> {
    fn weight(&self) -> f32;

}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub(crate) enum NodeSty {
    Role,
    Abstract,
    #[default]
    Concrete,
    Leaf,
}

impl GStyle for NodeSty {
    fn weight(&self) -> f32 {
        match self {
            NodeSty::Role => 2.,
            NodeSty::Abstract => 3.,
            NodeSty::Concrete => 1.,
            NodeSty::Leaf => 0.2,
        }
    }
}

impl From<NodeSty> for macroquad::color::Color {
    fn from(value: NodeSty) -> Self {
        match value {
            NodeSty::Role => macroquad::prelude::RED,
            NodeSty::Abstract => macroquad::prelude::BLUE,
            NodeSty::Concrete => macroquad::prelude::GREEN,
            NodeSty::Leaf => macroquad::prelude::LIME,
        }
    }
}

impl From<&NodeSty> for &macroquad::color::Color {
    fn from(value: &NodeSty) -> Self {
        match value {
            NodeSty::Role => &macroquad::prelude::RED,
            NodeSty::Abstract => &macroquad::prelude::BLUE,
            NodeSty::Concrete => &macroquad::prelude::GREEN,
            NodeSty::Leaf => &macroquad::prelude::LIME,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub(crate) enum RelSty {
    Role,
    Abstract,
    #[default]
    Child,
}
impl GStyle for RelSty {
    fn weight(&self) -> f32 {
        match self {
            RelSty::Role => 2.,
            RelSty::Abstract => 3.,
            RelSty::Child => 1.,
        }
    }
}

impl From<RelSty> for macroquad::color::Color {
    fn from(value: RelSty) -> Self {
        match value {
            RelSty::Role => macroquad::prelude::RED,
            RelSty::Abstract => macroquad::prelude::BLUE,
            RelSty::Child => macroquad::prelude::GREEN,
        }
    }
}

impl From<&RelSty> for &macroquad::color::Color {
    fn from(value: &RelSty) -> Self {
        match value {
            RelSty::Role => &macroquad::prelude::RED,
            RelSty::Abstract => &macroquad::prelude::BLUE,
            RelSty::Child => &macroquad::prelude::GREEN,
        }
    }
}

pub(crate) type NodeData = Weigthed<NodeSty>;
pub(crate) type RelData = Weigthed<RelSty>;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub(crate) struct Weigthed<Sty> {
    sty: Sty,
    pub(crate) weight: f32,
}
impl<Sty> From<Weigthed<Sty>> for f32 {
    fn from(d: Weigthed<Sty>) -> f32 {
        d.weight
    }
}
impl<Sty> From<&Weigthed<Sty>> for f32 {
    fn from(d: &Weigthed<Sty>) -> f32 {
        d.weight
    }
}
impl<Sty: GStyle> From<Sty> for Weigthed<Sty> {
    fn from(sty: Sty) -> Self {
        Self { weight: sty.weight(), sty }
    }
}
impl<Sty: GStyle + Clone> From<&Sty> for Weigthed<Sty> {
    fn from(sty: &Sty) -> Self {
        Self {
            sty: sty.clone(),
            weight: sty.weight(),
        }
    }
}
impl From<Weigthed<NodeSty>> for NodeSty {
    fn from(d: Weigthed<NodeSty>) -> Self {
        d.sty
    }
}
impl From<Weigthed<RelSty>> for RelSty {
    fn from(d: Weigthed<RelSty>) -> Self {
        d.sty
    }
}
impl<'a> From<&'a Weigthed<NodeSty>> for &'a NodeSty {
    fn from(d: &'a Weigthed<NodeSty>) -> Self {
        &d.sty
    }
}
impl<'a> From<&'a Weigthed<RelSty>> for &'a RelSty {
    fn from(d: &'a Weigthed<RelSty>) -> Self {
        &d.sty
    }
}
impl From<Weigthed<NodeSty>> for macroquad::color::Color {
    fn from(d: Weigthed<NodeSty>) -> Self {
        d.sty.into()
    }
}
impl From<Weigthed<RelSty>> for macroquad::color::Color {
    fn from(d: Weigthed<RelSty>) -> Self {
        d.sty.into()
    }
}
impl<'a> From<&'a Weigthed<NodeSty>> for &'a macroquad::color::Color {
    fn from(d: &'a Weigthed<NodeSty>) -> Self {
        (&d.sty).into()
    }
}
impl<'a> From<&'a Weigthed<RelSty>> for &'a macroquad::color::Color {
    fn from(d: &'a Weigthed<RelSty>) -> Self {
        (&d.sty).into()
    }
}

pub(crate) fn render(mut w: TypeSys) {
    let mut graph = Graph::default();
    graph.process_types(w);
    let graph = graph.graph;
    // let f = force::fruchterman_reingold_weighted(
    //     25.0, 0.985,
    // );
    let f = force::handy(65.0, 0.975, true, true);
    let mut sim_parameters = SimulationParameters::from_force(f);
    sim_parameters.node_start_size = 10.;
    // generate svg text for your graph
    let settings = Settings {
        sim_parameters,
        iterations: 10000,
        dt: 0.035 / 2.,
        // print_progress: true,
        node_color: RGBAColor(20, 34, 200, 1.),
        text_style: Some(TextStyle {
            font: ("sans-serif", 20).into_font(),
            color: BLACK.to_backend_color(),
            pos: Pos {
                h_pos: HPos::Left,
                v_pos: VPos::Center,
            },
        }),
        ..Default::default()
    };
    let svg = {
        let settings = Some(settings);
        // set up the simulation and settings
        let settings = settings.unwrap_or_default();
        let mut sim = Simulation::from_graph(graph, settings.sim_parameters);
        sim.parameters_mut().dimensions = Dimensions::Two;

        // get the nodes to their x/y positions through the simulation.
        for i in 0..settings.iterations {
            if settings.print_progress && i % 10 == 0 {
                println!("{}/{}", i, settings.iterations);
            }
            sim.update(settings.dt);
        }

        // get the size of the graph (avg of width and height to try to account for oddly shaped graphs)
        let (graph_x, graph_y): (f32, f32) = {
            let mut top = 0.0;
            let mut bottom = 0.0;
            let mut left = 0.0;
            let mut right = 0.0;

            for node in sim.get_graph().node_weights() {
                let loc = node.location;

                // add text width to the rightmost point to make sure text doesn't get cut off
                let rightmost = match settings.text_style.clone() {
                    Some(ts) => {
                        loc.x
                            + ts.font
                                .box_size(&node.name)
                                .ok()
                                .map(|x| x.0 as f32)
                                .unwrap_or(0.0)
                    }
                    None => loc.x,
                };

                if rightmost > right {
                    right = rightmost;
                }

                if loc.x < left {
                    left = loc.x;
                }

                if loc.y > top {
                    top = loc.y
                }

                if loc.y < bottom {
                    bottom = loc.y;
                }
            }

            (
                ((right + settings.node_size as f32) - (left - settings.node_size as f32)),
                ((top + settings.node_size as f32) - (bottom - settings.node_size as f32)),
            )
        };

        let image_scale = 1.5;
        let (image_x, image_y) = (
            (graph_x * image_scale) as u32,
            (graph_y * image_scale) as u32,
        );

        // translate all points by graph average to (0,0)
        let mut location_sum = Vec3::ZERO;
        for node in sim.get_graph().node_weights() {
            location_sum += node.location;
        }

        let avg_vec = location_sum / sim.get_graph().node_count() as f32;
        for node in sim.get_graph_mut().node_weights_mut() {
            node.location -= avg_vec;
        }

        // translate all the points over into the image coordinate space
        for node in sim.get_graph_mut().node_weights_mut() {
            node.location.x += (image_x / 2) as f32;
            node.location.y += (image_y / 2) as f32;
        }

        // SVG string buffer
        let mut buffer = String::new();

        // Plotters (who makes it very easy to make SVGs) backend
        let backend = SVGBackend::with_string(&mut buffer, (image_x, image_y)).into_drawing_area();

        // fill in the background
        backend.fill(&settings.background_color).unwrap();

        // draw all the edges
        for edge in sim.get_graph().edge_references() {
            let source = &sim.get_graph()[edge.source()].location;
            let target = &sim.get_graph()[edge.target()].location;

            let color = match edge.weight().into() {
                &RelSty::Role => settings.edge_color,
                &RelSty::Abstract => fdg_img::style::BLUE.to_rgba(),
                &RelSty::Child => fdg_img::style::GREEN.to_rgba(),
            };
            let stroke_width = match edge.weight().into() {
                &RelSty::Role => settings.edge_size,
                &RelSty::Abstract => settings.edge_size + 1,
                &RelSty::Child => settings.edge_size - 1,
            };
            let style = ShapeStyle {
                color,
                filled: true,
                stroke_width,
            };
            backend
                .draw(&PathElement::new(
                    vec![
                        (source.x as i32, source.y as i32),
                        (target.x as i32, target.y as i32),
                    ],
                    style,
                ))
                .unwrap();
        }

        // draw all the nodes
        for node in sim.get_graph().node_weights() {
            let coord = (node.location.x as i32, node.location.y as i32);
            match <&NodeSty>::from(&node.data).clone() {
                NodeSty::Role => backend.draw(&Rectangle::new(
                    [
                        (
                            coord.0 - settings.node_size as i32 / 2,
                            coord.1 - settings.node_size as i32 / 2,
                        ),
                        (
                            coord.0 + settings.node_size as i32 / 2,
                            coord.1 + settings.node_size as i32 / 2,
                        ),
                    ],
                    ShapeStyle {
                        color: settings.node_color,
                        filled: true,
                        stroke_width: 1,
                    },
                )),
                NodeSty::Concrete => backend.draw(&Circle::new(
                    coord,
                    settings.node_size,
                    ShapeStyle {
                        color: settings.node_color,
                        filled: true,
                        stroke_width: 1,
                    },
                )),
                NodeSty::Abstract => backend.draw(&plotters::element::Circle::new(
                    coord,
                    settings.node_size * 2,
                    ShapeStyle {
                        color: settings.node_color.mix(0.8),
                        filled: true,
                        stroke_width: 1,
                    },
                )),
                NodeSty::Leaf => backend.draw(&Circle::new(
                    coord,
                    settings.node_size / 2,
                    ShapeStyle {
                        color: settings.node_color.mix(0.5),
                        filled: true,
                        stroke_width: 1,
                    },
                )),
            }
            .unwrap();
        }

        // draw the text by nodes
        if let Some(text_style) = settings.text_style {
            for node in sim.get_graph().node_weights() {
                let pos = (
                    node.location.x as i32 + (text_style.font.get_size() / 2.0) as i32,
                    node.location.y as i32,
                );
                backend
                    .draw_text(node.name.as_str(), &text_style, pos)
                    .unwrap();
            }
        }

        drop(backend);

        buffer
    };

    // save the svg on disk (or send it to an svg renderer)
    fs::write("/tmp/fdg_graph.svg", svg.as_bytes()).unwrap();
}

#[derive(Default)]
pub struct Graph {
    graph: StableGraph<fdg_sim::Node<NodeData>, RelData, Undirected>
}

impl From<Graph> for StableGraph<fdg_sim::Node<NodeData>, RelData, Undirected> {
    fn from(value: Graph) -> Self {
        value.graph
    }
}

impl Graph {

pub(crate) fn process_types(
    &mut self,
    mut w: TypeSys,
) {
    let graph = &mut self.graph;

    let mut map = HashMap::with_capacity(w.types.len() as usize);
    let mut roles = HashMap::with_capacity(w.types.len() as usize);

    w.types
        .query_mut::<(&T,)>()
        .with::<(&DChildren,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), NodeSty::Concrete.into());
            map.insert(e, handle);
        });
    w.types
        .query_mut::<(&T,)>()
        .with::<(&Fields,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            if !map.contains_key(&e) {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Concrete.into());
                map.insert(e, handle);
            }
        });
    w.types
        .query_mut::<(&T,)>()
        .with::<(&SubTypes,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            if !map.contains_key(&e) {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Abstract.into());
                map.insert(e, handle);
            }
        });
    w.types
        .query_mut::<(&T,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            if !map.contains_key(&e) {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Leaf.into());
                map.insert(e, handle);
            }
        });
    w.types
        .query_mut::<(&Role, &DChildren)>()
        .with::<(&MultipleChildren,)>()
        .with::<(&RequiredChildren,)>()
        .into_iter()
        .for_each(|(e, (t, cs))| {
            let k = (t.deref().clone(), cs.deref().clone(), true, true);
            if let Some(handle) = roles.get(&k) {
                map.insert(e, *handle);
            } else {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Role.into());
                map.insert(e, handle);
                roles.insert(k, handle);
            }
        });
    w.types
        .query_mut::<(&Role, &DChildren)>()
        .without::<(&MultipleChildren,)>()
        .with::<(&RequiredChildren,)>()
        .into_iter()
        .for_each(|(e, (t, cs))| {
            let k = (t.deref().clone(), cs.deref().clone(), false, true);
            if let Some(handle) = roles.get(&k) {
                map.insert(e, *handle);
            } else {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Role.into());
                map.insert(e, handle);
                roles.insert(k, handle);
            }
        });
    w.types
        .query_mut::<(&Role, &DChildren)>()
        .without::<(&MultipleChildren,)>()
        .without::<(&RequiredChildren,)>()
        .into_iter()
        .for_each(|(e, (t, cs))| {
            let k = (t.deref().clone(), cs.deref().clone(), false, false);
            if let Some(handle) = roles.get(&k) {
                map.insert(e, *handle);
            } else {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Role.into());
                map.insert(e, handle);
                roles.insert(k, handle);
            }
        });
    w.types
        .query_mut::<(&Role, &DChildren)>()
        .with::<(&MultipleChildren,)>()
        .without::<(&RequiredChildren,)>()
        .into_iter()
        .for_each(|(e, (t, cs))| {
            let k = (t.deref().clone(), cs.deref().clone(), true, false);
            if let Some(handle) = roles.get(&k) {
                map.insert(e, *handle);
            } else {
                let handle = graph.add_force_node(t.to_string(), NodeSty::Role.into());
                map.insert(e, handle);
                roles.insert(k, handle);
            }
        });

    w.types
        .query_mut::<(&SubTypes,)>()
        .into_iter()
        .for_each(|(e, (st,))| {
            for t in st.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, RelSty::Abstract.into());
                }
            }
        });

    w.types
        .query_mut::<(&Fields,)>()
        .into_iter()
        .for_each(|(e, (fi,))| {
            for t in fi.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, RelSty::Role.into());
                } else {
                    // panic!()
                }
            }
        });

    w.types
        .query_mut::<(&DChildren,)>()
        .into_iter()
        .for_each(|(e, (cs,))| {
            for t in cs.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, RelSty::Child.into());
                }
            }
        });
}
}
