pub mod declaration;
pub mod element;
pub mod elements;
pub mod integration;
pub mod java_element;
pub mod label_value;
#[allow(unused)]
// TODO refactor the entire partial analysis to use interprete a declaration semantic
pub mod partial_analysis;
pub mod reference;
pub mod solver;

pub mod usage;

pub mod bytes_interner;

#[cfg(test)]
mod stack_graph_test;
#[cfg(test)]
mod test_solver;
#[cfg(test)]
mod wanted_test;
