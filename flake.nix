{
  description = "HyperAST";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShell = pkgs.mkShell rec {
          buildInputs = with pkgs; [
            clang
            llvmPackages.bintools
            rustup
          ];

          RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
          
          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          '';

          RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
            # add libraries here
          ]);

          BINDGEN_EXTRA_CLANG_ARGS = 
            (builtins.map (a: ''-I"${a}/include"'') [
              pkgs.glibc.dev 
            ])
            ++ [
              ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
        };
      }
    );
}
