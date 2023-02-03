pub mod script_generator;
pub mod script_generator2;
pub mod action_vec;
pub mod action_tree;


pub trait Actions {
    fn len(&self) -> usize;
}
