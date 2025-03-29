# Architecture

This document describes the architecture of the project. It is suppose to be highly technical and is not meant to be read to use the HyperAST.

## Overview of the structure

- **crates/hyperast** core crate representing the HyperAST structure

- **hyper_app** graphical interface to interact with the HyperAST (web and native support)
    - compute metrics
        > see also [Compute code metrics (GUI)](../quickstart/compute_code_metrics.md)
    - code tracking
        > see also [Track code (GUI)](../quickstart/track_code.md)

- **crates/backend** Rest API and server to remotely access HyperAST

- **crates/tsquery** query system for code in hyperast

- **book** your currently reading it!

- **vcs/git** facilities to handle the git control versioning system

- __gen/tree-sitter/*__ tree sitter grammars of supported language
    - Java
    - C++
    - Xml (used for maven's pom.xml)

- **crates/hyper_diff** algorithms to compute AST diffs

- **lib/egui_addon** small addon of functionalities used in hyper_app

- **lib/polyglote** generates node types from tree-sitter grammars
