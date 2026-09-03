{
  description = "Development environment for the vertical Zellij sidebar plugin";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = function:
        nixpkgs.lib.genAttrs systems (system:
          function {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            };
          });
    in
    {
      devShells = forAllSystems ({ pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              (rust-bin.stable.latest.default.override {
                targets = [ "wasm32-wasip1" ];
              })
              clang
              lld
              zellij
            ];

            shellHook = ''
              export RUST_BACKTRACE=1
              echo "vertical-sidebar development shell"
              echo "Build with: cargo build --target wasm32-wasip1"
            '';
          };
        });
    };
}
