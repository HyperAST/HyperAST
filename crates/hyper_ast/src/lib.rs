// #![feature(min_specialization)]
#![feature(exact_size_is_empty)]
#![feature(slice_index_methods)]
#![feature(let_chains)]

pub mod compat;
#[cfg(feature = "legion")]
pub mod cyclomatic;
pub mod filter;
pub mod full;
pub mod hashed;
pub mod impact;
pub mod nodes;
pub mod position;
pub mod store;
pub mod tree_gen;
pub mod types;
pub mod usage;
pub mod scripting;
pub mod utils;

pub trait PrimInt: num::PrimInt + num::traits::NumAssign + std::fmt::Debug {}
impl<T> PrimInt for T where T: num::PrimInt + num::traits::NumAssign + std::fmt::Debug {}

mod slice_interning;

pub mod test_utils;

#[cfg(test)]
mod tests;

mod graph {
    //! An overlay graph for the hyperast's subtrees
    //!
    //! First targeted usage for this feature is tree-sitter-stack-graph.
    //! Need:
    //! - attributes both on edges and nodes
    //! - directed edges
    //!
    //! Some other refs:
    //! - Heuristics for semi-external depth first search on directed graphs. JF Sibeyn, J Abello, U Meyer
    //!
    //! Types of edges:
    //! - forward
    //! - backward
    //! - cross-edge
    //!
    //! Consequences (not final mem layout):
    //! - node on current subtree root: node(attr)
    //! - node on child subtree: child(attr, path, node)
    //! - forward edge: fwd(source, attr, path, sink)
    //! - backward edge: bck(sink, attr, path, source)
    //! - cross-edge: cross(source, source_path, attr, sink_path, sink)
    //!
    //!
    //! Conclusion for impl planning:
    //!  too many possible layout with difficult to predict consequences
    //!
    //! The big issue is about merging nodes:
    //! - a node can be created in the subtree but also its parent, and have additional attrs from parent
    //!
    //! but it's ok actually, just some book keeping,
    //! ie. source or sink are
    //! offsets (like all nodes at subtree + created ones)
    //! or string (node are considered to have a uniq name per subtree)

    mod heap {
        //! ref impl, as simple as possible, no dedup, no additional arena.
        //! Issues might appear on large trees
        enum Value {
            Float(f32),
        }

        struct Path(Vec<u16>);

        struct Attrs(std::collections::HashMap<String, Value>);

        struct Nodes(Vec<Node>);

        enum Node {
            Current(Attrs),
            Child(Attrs, Path),
        }

        struct Edges(Vec<Edge>);

        struct N(u8);
        struct Source(N);
        struct Sink(N);
        enum Edge {
            Fwd(Attrs, Path, Sink),
            Bck(Attrs, Path, Source),
            Cross(Source, Path, Attrs, Path, Sink),
        }
    }

    mod enum_ {
        //! also a simple impl, but can abuse the type erasure provided by the ECS
        //! its definitely a trade-off, but does not solve memory footprint issues, just reduce some indirections.
    }

    // will most likely find an inbetween, with stuff in heap some in enums and some deduplicated global & immutable values

    mod packed {
        //! vec of symbols that's used to reconstitute nodes and edges, something similar to the predicates in the rust wrapper of tree-sitter
        //! using a global index to deduplicate stuff and in each subtree a list of same size things that can point on the deduplicated elements.
        //! Looks like it could be efficient but difficult to make correct and fast
    }

    mod parent_llist {
        //! like the structral positions with path stores.
        //! should work better for very dense places
    }
}
