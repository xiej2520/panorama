{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      flake-utils,
      naersk,
      fenix,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
        toolchain = with fenix.packages.${system}; combine [
          stable.cargo
          stable.rustc
          stable.rust-src
          stable.clippy
          latest.rustfmt # nightly rustfmt
        ];

        buildInputs = with pkgs; [
          SDL2
        ];

        nativeBuildInputs = with pkgs; [
          llvmPackages.bintools # lld
        ];
      in
      rec {
        defaultPackage = packages.app;
        packages = {
          app = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = nativeBuildInputs;
            buildInputs = buildInputs;
          };
          container = pkgs.dockerTools.buildImage {
            name = "app";
            config = {
              entrypoint = [ "${packages.app}/bin/app" ];
            };
          };
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs =
            with pkgs;
            [
              nixpkgs-fmt
              toolchain
            ]
            ++ buildInputs
            ++ nativeBuildInputs;
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          RUSTFLAGS = "-Clink-arg=-fuse-ld=lld -Clink-self-contained=-linker";
        };
      }
    );
}
