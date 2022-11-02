// Benchmark of diff using the hyperAST, compared against https://github.com/GumTreeDiff/gumtree

// algorithm: gumtree zs changedisttiller rted
// implementation: gumtree (gumtree, gumtreesimple)

// validity: baseline gumtree, identity comparison
// performances: baseline gumtree, time/memory
// code: repository (reuse ASE repositories and add some code so that gumtree works on whole commits ) / files (reuse gumtree dataset)

// scenario #1: buggy/fixed
// scenario #2: consecutive commits
// scenario #2: quadratic commits ? consequence of usage ? related to precision of diff (because if we do not loose information (in result) we should get consitent results)

// RQ 1: validity: is our implementation computing the same edit scripts that gumtree ?
// RQ 2: performances: how our performances compare for the task of computing edit scripts on consecutive commits ? on a set of buggy/fixed files ?
// RQ 3: scaling: what is the maximum number of commits that can be incremetally processed while staying in RAM ? 
//                what is the maximum size of the window where we can compute all combination of edit scripts ?
#[cfg(test)]
mod random_sample_diff;
#[cfg(test)]
mod window_combination;
#[cfg(test)]
mod swap_diff;
#[cfg(test)]
mod buggy_fixed;

fn main() {

}