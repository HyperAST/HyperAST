use fdg_macroquad::ApplicationState;
use fdg_sim::{force, petgraph::Undirected};

use crate::render_fdg_custom::{NodeData, NodeSty, RelData, RelSty};

pub(crate) async fn live(
    graph: fdg_sim::petgraph::stable_graph::StableGraph<
        fdg_sim::Node<NodeData>,
        RelData,
        Undirected,
    >,
) {
    // fdg_macroquad::run_window(&graph).await; // not working, NaNs are not handled correctly in fdg-sim
    let mut window = ApplicationState::new(graph);
    let force = force::fruchterman_reingold2(35.0, 0.9);
    window.sim.parameters_mut().set_force(force.clone());
    window.current_force = force.clone();
    window.available_forces.push(force);
    window.on_drag = Box::new(|d| d.weight *= 10000.);
    window.on_undrag = Box::new(|d| if d.weight > 1000. {d.weight /= 10000.});
    window.run_colored().await;
}
