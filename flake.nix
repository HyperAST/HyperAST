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

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
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
          buildAndTestSubdir = "crates/backend";
          OPENSSL_NO_VENDOR = 1;
          release = true;
          doCheck = false;
          buildInputs = with pkgs; [
            # misc libraries
            openssl
            cacert
          ];
          nativeBuildInputs = with pkgs; [
            # misc libraries
            cmake
            pkg-config
            openssl
            cacert

            # Rust
            (rust-bin.fromRustupToolchainFile ./rust-toolchain)
          ];
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
        };
      in {
        apps = {
          hyperast-backend = {
            type = "app";
            program = "${hyperast-backend}/bin/backend";
          };
          hyperast-scripting = {
            type = "app";
            program = "${hyperast-backend}/bin/scripting";
          };
        };

        packages = {
          hyperast = hyperast-backend;

          hyperast-dockerImage = pkgs.dockerTools.buildLayeredImage {
            name = "HyperAST";
            tag = "0.2.0";
            runAsRoot = ''
              ln -s  ${hyperast-backend}/bin/scripting /scripting
              ln -s  ${hyperast-backend}/bin/backend /backend
            '';
            config = {
              Cmd = ["/backend -- 0.0.0.0:8888"];
              Env = [
                "GIT_SSL_CAINFO=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
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
