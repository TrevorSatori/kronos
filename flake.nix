{
  description = "Terminal Music Player Written In Rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.naersk.url = "github:nix-community/naersk/master";
  inputs.utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, utils, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk {  };
      in
        {
          defaultPackage = naersk-lib.buildPackage {
            src = ./.;
            buildInputs = with pkgs; [
              pkg-config
              alsa-lib
            ];
          };

          devShell = with pkgs; mkShell {
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              rustPackages.rust-analyzer
              pkg-config
              alsa-lib
            ];
          };
        }
    );
}
