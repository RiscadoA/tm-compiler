{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem ( system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      with pkgs; {
        devShell = mkShell {
          buildInputs = [
            pkgconfig
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })
            rust-bin.stable.latest.rust-analysis
            rust-analyzer
          ];

          shellHook = ''
            exec zsh
          '';
        };
      }
    );
}
