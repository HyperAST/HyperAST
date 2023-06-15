# How to install Rust toolchain

## Setup Rust

> [How to install Rust](https://www.rust-lang.org/tools/install)

We use the [**nightly toolchain**](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) of the Rust compiler in this project. It is pretty much stable and offers more features for now.

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


// WARN : not working patch
## MacOS
issue with trunk https://stackoverflow.com/questions/72146492/unable-to-execute-trunk-serve

```bash
cargo install wasm-bindgen-cli
```