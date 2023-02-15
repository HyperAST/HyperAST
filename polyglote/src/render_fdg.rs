use std::fs;
use std::ops::Deref;

use std::collections::HashMap;

use fdg_img::style::text_anchor::{HPos, Pos, VPos};
use fdg_img::style::{Color, IntoFont, RGBAColor, TextStyle, BLACK};
use fdg_img::Settings;
use fdg_sim::{force, ForceGraph, ForceGraphHelper, SimulationParameters};

use crate::preprocess::{Child, DChildren, Fields, Role, SubType, SubTypes, TypeSys, T};

pub(crate) fn render(mut w: TypeSys) {
    let graph = init_graph(w);
    // let f = force::fruchterman_reingold_weighted(
    //     25.0, 0.985,
    // );
    let f = force::handy(65.0, 0.975, true, true);
    let mut sim_parameters = SimulationParameters::from_force(f);
    sim_parameters.node_start_size = 10.;
    // generate svg text for your graph
    let svg = fdg_img::gen_image(
        graph,
        Some(Settings {
            sim_parameters,
            iterations: 10000,
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
        }),
    )
    .unwrap();

    // save the svg on disk (or send it to an svg renderer)
    fs::write("/tmp/fdg_graph.svg", svg.as_bytes()).unwrap();
}

pub(crate) fn init_graph(
    mut w: TypeSys,
) -> fdg_sim::petgraph::stable_graph::StableGraph<
    fdg_sim::Node<f32>,
    f32,
    fdg_sim::petgraph::Undirected,
> {
    // initialize a graph
    let mut graph: ForceGraph<f32, f32> = ForceGraph::default();

    let weight = 0.01;

    let mut map = HashMap::with_capacity(w.abstract_types.len() as usize);
    w.abstract_types
        .query_mut::<(&T,)>()
        .with::<(&Child,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });
    w.abstract_types
        .query_mut::<(&T,)>()
        .without::<(&Child,)>()
        .with::<(&SubType,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });
    w.abstract_types
        .query_mut::<(&T,)>()
        .without::<(&Child,)>()
        .without::<(&SubType,)>()
        .with::<(&SubTypes,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });
    w.abstract_types
        .query_mut::<(&T,)>()
        .without::<(&Child,)>()
        .without::<(&SubType,)>()
        .with::<(&DChildren,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });
    w.abstract_types
        .query_mut::<(&T,)>()
        .without::<(&Child,)>()
        .without::<(&SubType,)>()
        .with::<(&Fields,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });
    w.abstract_types
        .query_mut::<(&Role,)>()
        .into_iter()
        .for_each(|(e, (t,))| {
            let handle = graph.add_force_node(t.to_string(), weight);
            map.insert(e, handle);
        });

    w.abstract_types
        .query_mut::<(&SubTypes,)>()
        .into_iter()
        .for_each(|(e, (st,))| {
            for t in st.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, 0.0);
                }
            }
        });

    w.abstract_types
        .query_mut::<(&Fields,)>()
        .into_iter()
        .for_each(|(e, (fi,))| {
            for t in fi.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, weight);
                } else {
                    // panic!()
                }
            }
        });

    w.abstract_types
        .query_mut::<(&DChildren,)>()
        .into_iter()
        .for_each(|(e, (fi,))| {
            for t in fi.deref() {
                if let (Some(a), Some(b)) = (map.get(&e), map.get(&t)) {
                    // dbg!();
                    graph.add_edge(*a, *b, weight);
                }
            }
        });
    graph
}
