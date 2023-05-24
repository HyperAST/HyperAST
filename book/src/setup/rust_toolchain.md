# How to install Rust toolchain

## Setup Rust

How to install [Rust doc here](https://www.rust-lang.org/tools/install)

We use the nightly toolchain of the Rust compiler in this project. It is pretty much stable and offers more features for now.

Those commmands will install the nightly toolchain and set it as default.

```bash
> rustup toolchain install nightly
```

```bash
> rustup default nightly
```


To go back to the stable toolchain, run:

```bash
> rustup default stable
```