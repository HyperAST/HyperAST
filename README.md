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

## How to use 

You will have access to the [GUI](https://hyperast.github.io/gui/index.html). However, in order to use any of the GUI features, you will need to launch/connect to the REST API server. 

### Launch server with Cargo (You have to handle depencies such as rustup, openssl, ... yourself)
```sh
cargo run -p client --release
```
### Launch server with Nix (A package manager for reproducible, declarative and reliable systems)
Look [there](https://nixos.org/download) for instruction on how to install Nix on your system.
```sh
nix run github:LeDilavrecQuentin/HyperAST#hyperast-webapi
```
This will download all dependencies and build locally. 
This will work on any *NIX system (Linux, WSL, MACOSX, ...).

There is also a devShell provided with all the necessary dependencies installed in a healthy environment to develop and build the project. You can enter the environement with:
```sh
nix develop # from the project root
```

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

