// use fdg_macroquad::ApplicationState;
use fdg_sim::{force, petgraph::Undirected};

use crate::render_fdg_custom::{NodeData, NodeSty, RelData, RelSty};

pub(crate) async fn live(
    graph: fdg_sim::petgraph::stable_graph::StableGraph<
        fdg_sim::Node<NodeData>,
        RelData,
        Undirected,
    >,
) {
    fdg_macroquad::run_window(&graph).await; // not working, NaNs are not handled correctly in fdg-sim
    // let mut window = ApplicationState::new(graph);
    // let force = force::fruchterman_reingold2(35.0, 0.9);
    // window.sim.parameters_mut().set_force(force.clone());
    // window.current_force = force.clone();
    // window.available_forces.push(force);
    // window.edge_color_cb = Box::new(|d| match d.into() {
    //     &RelSty::Role => macroquad::prelude::RED,
    //     &RelSty::Abstract => macroquad::prelude::BLUE,
    //     &RelSty::Child => macroquad::prelude::GREEN,
    // });
    // window.node_color_cb = Box::new(|d| match NodeSty::from(d.clone()) {
    //     NodeSty::Role => macroquad::prelude::RED,
    //     NodeSty::Abstract => macroquad::prelude::BLUE,
    //     NodeSty::Concrete => macroquad::prelude::GREEN,
    //     NodeSty::Leaf => macroquad::prelude::LIME,
    // });
    // window.on_drag = Box::new(|d| d.weight = 10000.);
    // window.run().await;
}
