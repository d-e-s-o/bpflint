{
  lib,
  rust-bin,
  makeRustPlatform,
}: let
  manifest = (lib.importTOML ./Cargo.toml).package;
  fs = lib.fileset;
  rustBinary = rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  rustPlatform = makeRustPlatform {
    cargo = rustBinary;
    rustc = rustBinary;
  };
in
  rustPlatform.buildRustPackage {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    cargoBuildFlags = [ "-p" "bpflinter" ];
    src = fs.toSource {
      root = ./.;
      fileset = fs.unions [
        ./Cargo.toml
        ./Cargo.lock
        ./src
        ./make
        ./lints
        ./cli
        ./examples
        ./build.rs
      ];
    };

    meta = {
      mainProgram = "bpflinter";
      description = "A linter for BPF C programs.";
      homepage = "https://github.com/d-e-s-o/bpflint";
      license = lib.licenses.mit;
    };
  }
