{
  description = "Linting functionality for BPF C programs.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          (final: prev: {
            bpflint = final.callPackage ./derivation.nix {};
          })
        ];
      };
    in {
      packages.default = self.packages.${system}.bpflint;
      packages.bpflint = pkgs.bpflint;

      devShells.default = pkgs.mkShell {
        inputsFrom = [pkgs.bpflint];
        packages = [
        ];
      };
    });
}
