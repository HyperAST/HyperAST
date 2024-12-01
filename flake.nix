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
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        apps = {
          hyperast-webapi = {
            type = "app";
            program = "${self.packages.${system}.hyperast-webapi}/bin/client";
          };
        };

        packages = {
          hyperast-webapi = pkgs.rustPlatform.buildRustPackage rec {
            pname = "HyperAST-WebAPI";
            version = "0.1.0";
            src = pkgs.lib.cleanSource ./.;
            buildAndTestSubdir = "client";
            OPENSSL_NO_VENDOR = 1;
            release = true;
            doCheck = false;
            buildInputs = with pkgs; [
              # misc libraries
              openssl
            ];
            nativeBuildInputs = with pkgs; [
              # misc libraries
              cmake
              pkg-config
              
              # Rust
              (rust-bin.fromRustupToolchainFile ./rust-toolchain)
            ];
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
          };
        };

        devShell = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            # Rust 
            (rust-bin.fromRustupToolchainFile ./rust-toolchain)
            trunk
            
            # misc
            pkg-config

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
            openssl
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libraries;
        };
      }
    );
}
