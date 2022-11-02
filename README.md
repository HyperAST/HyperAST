# HyperAST

The HyperAST is an AST structured as a Direct Acyclic Graph (DAG) (similar to MerkleDAG used in Git).
An HyperAST is efficently constructed by leveraging Git and TreeSitter. 

It also reimplements the Gumtree algorithm in Rust while using the HyperAST as the underlying AST structure.

It also implements a use-def solver,
that uses a context-free indexing of references present in subtrees (each subtree has a bloom filter of contained references).