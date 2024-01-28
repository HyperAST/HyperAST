{
  description = "HyperAST";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          apps.hyperast-webapi = {
            type = "app";
            program = "${self.packages.${system}.hyperast-webapi}/bin/client";
          };
          packages = {
            hyperast-webapi = pkgs.rustPlatform.buildRustPackage rec {
              pname = "HyperAST-WebAPI";
              version = "0.0.1";
              src = pkgs.lib.cleanSource ./.;
              buildAndTestSubdir = "client";
              OPENSSL_NO_VENDOR = 1;
              release = true;
              doCheck = false;
              buildInputs = with pkgs; [
                openssl
              ];

              nativeBuildInputs = with pkgs; [
                cmake
                pkg-config
                (rust-bin.fromRustupToolchainFile ./rust-toolchain)
              ];

              PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "tree-sitter-cpp-0.20.0" = "sha256-bRHI/zQgBKiaHD+erO8gCCxLAX+qePRZOkTlva/Dcx0=";
                  "tree-sitter-java-0.20.9" = "sha256-lb3dJJs9QEu1qkuhbJn7ssy6wg0lxhDI9KGcYWOD0FM=";
                  "tree-sitter-typescript-0.20.3" = "sha256-q8vJnJZdWzsiHHJSPGoM938U5AxuOIuGrx1r6F+cdK4=";
                  "tree-sitter-xml-0.20.9" = "sha256-fG5tdBzOigZkRjdR2WLCBWM3pQYTLoNIxLd0B4G+2cM=";
                };
              };

            };
          };

          devShell = pkgs.mkShell rec {
            buildInputs = with pkgs;
              [
                clang
                llvmPackages.bintools
                (rust-bin.fromRustupToolchainFile ./rust-toolchain)

                pkg-config
                nixfmt
                cmake
                trunk
              ];

            libraries = with pkgs; [
              openssl
              glibc
            ];

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
          };
        }
      );
}
