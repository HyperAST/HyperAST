# How to install Rust toolchain

## Install Rust

> [rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Setup Nightly Channel

We use the [nightly toolchain](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) of the Rust compiler in this project. It is stable enough and offers useful features.


Those commmands will install the nightly toolchain and set it as default.

```bash
> rustup toolchain install nightly
```

```bash
> rustup default nightly
```


To go back to the stable toolchain after you are done with the project, run:

```bash
> rustup default stable
```

## Troubleshooting :

### MacOS
issue with trunk https://stackoverflow.com/questions/72146492/unable-to-execute-trunk-serve

```bash
cargo install wasm-bindgen-cli
```



./concrete_applications/concrete_applications.md