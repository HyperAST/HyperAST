# HyperAST

[![CICD badge]][CICD]
[![DOI](https://zenodo.org/badge/14164618.svg)](https://doi.org/10.1145/3551349.3560423)
![](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)

[CICD badge]: https://github.com/HyperAST/HyperAST/actions/workflows/deploy.yml/badge.svg
[CICD]: https://github.com/HyperAST/HyperAST/actions/workflows/deploy.yml

> [Home Page HyperAST](https://hyperast.github.io/)

### [Book](https://hyperast.github.io/book/index.html)

#### [Getting Started](https://hyperast.github.io/book/quickstart/quickstart.html)

##### [Compute code Metrics(GUI)](https://hyperast.github.io/book/quickstart/compute_code_metrics.html)

##### [Track Code(GUI)](https://hyperast.github.io/book/quickstart/track_code.html)

---

### [GUI](https://hyperast.github.io/gui/index.html)

---

### [Doc](https://hyperast.github.io/doc/hyper_ast/index.html)

## Summary

The HyperAST is an AST structured as a Direct Acyclic Graph (DAG) (similar to MerkleDAG used in Git).
An HyperAST is efficiently constructed by leveraging Git and TreeSitter.

It reimplements the Gumtree algorithm in Rust while using the HyperAST as the underlying AST structure.

It implements a use-def solver,
that uses a context-free indexing of references present in subtrees (each subtree has a bloom filter of contained references).

## How to Cite

If you use the HyperAST is an academic purpose, please cite the following paper:

```bibtex
@inproceedings{ledilavrec:hal-03764541,
  TITLE = {{HyperAST: Enabling Efficient Analysis of Software Histories at Scale}},
  AUTHOR = {Le Dilavrec, Quentin and Khelladi, Djamel Eddine and Blouin, Arnaud and J{\'e}z{\'e}quel, Jean-Marc},
  URL = {https://hal.inria.fr/hal-03764541},
  BOOKTITLE = {{ASE 2022 - 37th IEEE/ACM International Conference on Automated Software Engineering}},
  PUBLISHER = {{IEEE}},
  PAGES = {1-12},
  YEAR = {2022}
}
```

