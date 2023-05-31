# GUI

# Intro
The gui is written in full Rust using [egui](https://github.com/emilk/egui) and [eframe](https://docs.rs/eframe/latest/eframe/).

It is made to analyze any git repository. 

# Objectives (Example of use cases)

- Count the number of commits per author
- Understand where a bug was comming from
- Match patterns in the entire data base ()
- See how a piece a code has evolved over times (= commits)

# How to use it

This is a quick overview of how to use the app.

## Run the app

Open 2 terminals and run the following commands:

On the first one in `HyperAST/hyper_app/` :
``` bash
HyperAST/hyper_app/ > trunk serve
```

On the seconde one in `HyperAST/` run :
```bash
HyperAST/ > cargo run -p client --release
```

You can then access the app on the url given by the first command.

## Use the app

### Single Repository

(Only github is supported for now.)

The app needs 2 informations to work:
- The path to the repository
- The branch to analyze by giving the commit hash

You can then use the Rhai scripting language to write your own queries on this repositories.

Some function are supported by default to interact with the repo :
- `is_directory()` : tells if it is a directory
- `children()` : return the children of a directory
- `is_file()` : tells if it a file
- `is_type_decl()` : tells if it is a type declaration

TODO : understand how to use the scripting language -> clear explanation about it

### Multi Repo (not supported yet)

### Semantic Diff (not supported yet)

### Code Tracking

Nothing for now ?
TODO : ask quentin about that

### Long Tracking

Avancer de plus en plus loin dans les commits de façon interactive.

### Aspect Views

Not very useful for now. 
