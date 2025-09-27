{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };

        buildInputs = with pkgs; [
          glfw
          pkg-config
          xorg.libX11.dev
          wayland

          # not necessary to run
          gdk-pixbuf # Could not load a pixbuf from icon theme
          librsvg # Could not load a [svg] pixbuf from icon theme
        ];

        nativeBuildInputs = with pkgs; [

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
              rustc
              cargo
              clippy
              rustfmt
            ]
            ++ buildInputs
            ++ nativeBuildInputs;
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          # removing GDK_BACKEND and LIBDECOR_PLUGIN causes libdecor-gtk-WARNING: Failed to initialize GTK
          # but still runs
          GDK_BACKEND = "wayland,x11";
          LIBDECOR_PLUGIN = "default";
        };
      }
    );
}

