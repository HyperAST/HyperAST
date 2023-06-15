# Architecture

This document describes the architecture of the project. It is suppose to be highly technical and is not meant to be read to use the HyperAST.

## Overview of the structure
It is a description of the front branch the 23th of Mai 2023.

- - [ ] `benchmark` for HyperAST
    - [~] `main` ⚠️ main and reserializing share the same 70 first lines of code (types definition)
        - `struct Info`
        - `struct Instance`
        - `multi_commit_ref_ana()`
        - `single_commit_ref_ana()`
    - [ ] `reserializing` ⚠️
    - [ ] `write_serializer`
- - [ ] `benchmark_diffs` for 
    - [ ] 
- - [ ] `client`
    - [ ] 
- - [ ] `cvs`
    - [ ] 
- - [ ] `gen`
    - [ ] 
- - [ ] `hyper_app` : [egui-eframe template](https://github.com/emilk/eframe_template) app
    - [ ] `app` the complete egui app where everithing is called
    - [ ] `app/code_editor`
    - [ ] `app/tree_view`
    - [ ] `app/ts_highlight`
    - [ ] `app/code_aspect`
    - [ ] `app/code_editor`
    - [ ] `app/code_tracking`
    - [ ] `app/commit`
    - [ ] `app/egui_utils`
    - [ ] `app/interactive_split`
    - [ ] `app/long_tracking`
    - [ ] `app/long_tracking0`
    - [ ] `app/multi_split`
    - [ ] `app/single_repo`
    - [ ] `app/split_from_side_panel`
    - [ ] `app/split`
    - [ ] `app/syntax_hightlight_async`
    - [ ] `app/syntax_hightlight_ts` syntax hightlight for tree sitter
    - [ ] `app/types` 
    - [ ] `app/utils` just function to save a file
- - [ ] `hyper_ast`
    - [ ] `/cyclomatic`
    - [ ] `/filter`
    - [ ] `/impact`
    - [ ] `/store`
    - [ ] `/tests`
    - [ ] `/tree_gen`
    - [x] `/usage` => ⚠️ completely commented
    - [x] `compat` small compatibility layer
    - [x] `full` FullNode (contains the local and global node)
    - [ ] `hashed` lot of hashing functions for the nodes
    - [ ] `nodes` 
    - [ ] `position`
    - [ ] `types`
    - [ ] `utils`    
- - [ ] `hyper_diff`
    - [ ] 
- - [ ] `hyper_view_try`
    - [ ] 
- - [ ] `ref-mining-evaluation`
    - [ ] 
- - [ ] `tree-sitter_types`
    - [ ] 