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

### [Doc](https://hyperast.github.io/doc/hyperast/index.html)

## Summary

HyperAST is an AST structured as a Direct Acyclic Graph (DAG) (similar to MerkleDAG used in Git).
An HyperAST is efficiently constructed by leveraging [Git](https://git-scm.com/) and [TreeSitter](https://tree-sitter.github.io/tree-sitter/).

It reimplements the [Gumtree](https://hal.science/hal-01054552/document) algorithm in Rust while using HyperAST as the underlying AST structure.

It implements a use-def solver,
that uses a context-free indexing of references present in subtrees (each subtree has a bloom filter of contained references).

## How to use 

You can use the dedicated [GUI](https://hyperast.github.io/gui/index.html) in your browser. However, in order to use any of the GUI features, you will need to launch/connect to the REST API server. 

### Launch server with Nix (A package manager for reproducible, declarative and reliable systems)
Look [there](https://nixos.org/download) for instruction on how to install Nix on your system.
```sh
nix run .#hyperast-webapi // similar to the prev. mentioned cargo run 
nix run github:HyperAST/HyperAST#hyperast-webapi // here nix handles everything, no need to clone!
```
This will download all dependencies and build locally. 
It can work on any *NIX system (Linux, WSL, MACOSX, ...), but the CPU architecture can be a problem e.g. I could not make it work on an M1.

There is also a development shell provided with all the necessary dependencies installed in a healthy environment to develop and build the project. You can enter the environment with:
```sh
nix develop # from the project root dir
```
### Launch our server with Cargo (You have to handle system dependencies yourself, such as, `rustc`, `openssl` )
```sh
cargo run -p backend --release # from the project root dir, after having cloned the repository
```
Note: Currently HyperAST uses features from the nightly channel, so you should definitely use [rustup](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file), the Rust version manager.

## How to Cite

If you use HyperAST and/or HyperDiff in an academic purpose, please cite the following papers:

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


```bibtex
@inproceedings{ledilavrec:hal-04189855,
  TITLE = {{HyperDiff: Computing Source Code Diffs at Scale}},
  AUTHOR = {Le Dilavrec, Quentin and Khelladi, Djamel Eddine and Blouin, Arnaud and J{\'e}z{\'e}quel, Jean-Marc},
  URL = {https://inria.hal.science/hal-04189855},
  BOOKTITLE = {{ESEC/FSE 2023 - 31st ACM Joint European Software Engineering Conference and Symposium on the Foundations of Software Engineering}},
  PUBLISHER = {{ACM}},
  PAGES = {1-12},
  YEAR = {2023}
}
```

