{
  description = "HyperAST";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        filter = inputs.nix-filter.lib;
        hyperast-backend = pkgs.rustPlatform.buildRustPackage {
            pname = "HyperAST";
            version = "0.1.0";
            src = filter {
              root = ./.;
              exclude = [
                ./.vscode
                ./.github/workflows
                ./.direnv
                ./target
                ./flake.lock
                ./flake.nix
                ./LICENCES
                ./README.md
              ];
            };
            buildAndTestSubdir = "client";
            OPENSSL_NO_VENDOR = 1;
            # LIBGIT2_NO_VENDOR = 1;
            LIBGIT2_SYS_USE_PKG_CONFIG = 1;
            release = true;
            doCheck = false;
            dontPatchELF = true;
            buildInputs = with pkgs; [
              # misc libraries
              zlib
              openssl
              # libgit2 
            ];
            nativeBuildInputs = with pkgs; [
              # misc libraries
              cmake
              pkg-config
              zlib
              # libgit2 
              openssl
              perl
              
              # Rust
              (rust-bin.fromRustupToolchainFile ./rust-toolchain)
            ];
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
          };
      in
      {
        apps = {
          hyperast-backend = {
            type = "app";
            program = "${hyperast-backend}/bin/client";
          };
           hyperast-scripting = {
            type = "app";
            program = "${hyperast-backend}/bin/scripting";
          };
        };

        packages = rec {
          hyperast-webapi = hyperast-backend;

          hyperast-api-dockerImage = pkgs.dockerTools.buildImage {
            name = "HyperAST";
            tag = "0.2.0";
             runAsRoot = ''
              ln -s  ${hyperast-backend}/bin/scripting /scripting
              ln -s  ${hyperast-backend}/bin/client /client
            '';
            config = {
              Cmd = [ "/client -- 0.0.0.0:8888" ];
            };
          };
        };

        devShell = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            # Rust 
            (rust-bin.fromRustupToolchainFile ./rust-toolchain)
            trunk
            
            # misc
            cmake
            pkg-config
            dive
            perl

            # Nix
            nixfmt
          ];
          libraries = with pkgs; [
            # x11 libraries
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11

            # wayland libraries
            wayland
            
            # GUI libraries
            libxkbcommon
            libGL
            fontconfig

            # misc libraries
            # openssl
            # libgit2.dev 
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        };
      }
    );
}
