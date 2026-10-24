{
  description = "Gala build and development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }@inputs:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      pkgsFor = system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay { inherit inputs nixpkgs; }).overlays.default ];
        };
      in pkgs // {
        inherit (pkgs) llvmPackages_10 as llvm10;
        rust = pkgs.rust-bin.from-overlay?abort "rust-bin.from-overlay missing" or pkgs.rust-bin.stable;
      };
    in flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = pkgsFor system;
        rustToolchain = pkgs.rust-bin."stable-2026-04-14" or pkgs.rust-bin.stable;
        llvm = pkgs.llvm10;
      in {
        devShells.default = pkgs.mkShell rec {
          name = "gala-dev-shell";
          nativeBuildInputs = [
            rustToolchain
            pkgs.rustfmt
            pkgs.clippy
            pkgs.clang
            llvm.lib
            llvm.dev
            pkgs.cargo-deny
            pkgs.cargo-audit
            pkgs.cmake
            pkgs.pkg-config
            pkgs.openssl
            pkgs.git
            pkgs.curl
          ];
          RUSTFLAGS = "-D warnings";
          shellHook = ''
            echo "Gala dev shell — edition 2021, toolchain stable-2026-04-14, LLVM 10"
          '';
        };

        packages.default = pkgs.buildRustPackage rec {
          pname = "gala";
          version = "0.1.0";
          src = ./.;

          # NOTE: cargoHash must be computed for the pinned Cargo.lock.
          # Run: nix build .#packages.<system>.default  and copy the hash
          # reported in the error message into this attribute.
          cargoHash = "sha256:0000000000000000000000000000000000000000000000000000";

          nativeBuildInputs = [
            pkgs.cmake
            pkgs.pkg-config
            pkgs.openssl
            pkgs.clang
            llvm.dev
          ];

          cargoBuildFlags = [ "--workspace" "--release" ];

          doInstall = ''
            mkdir -p $out/bin
            for b in gala gala-fmt gala-lsp gala-cli gala-lint gala-pkg; do
              if [ -x ./target/release/$b ]; then
                install -m 0755 ./target/release/$b "$out/bin/"
              fi
            done
          '';
        };

        packages.gala-bin = packages.default;

        checks.build = packages.default;
        checks.test = pkgs.buildRustPackage rec {
          pname = "gala-test";
          version = "0.1.0";
          src = ./.;
          cargoHash = packages.default.cargoHash;
          nativeBuildInputs = packages.default.nativeBuildInputs;
          doBuild = ''
            cargo test --workspace
          '';
          doInstall = ''
            true
          '';
        };
        checks.clippy = pkgs.buildRustPackage rec {
          pname = "gala-clippy";
          version = "0.1.0";
          src = ./.;
          cargoHash = packages.default.cargoHash;
          nativeBuildInputs = packages.default.nativeBuildInputs;
          doBuild = ''
            cargo clippy --workspace --all-targets -- -D warnings
          '';
          doInstall = ''
            true
          '';
        };
        checks.fmt = pkgs.buildRustPackage rec {
          pname = "gala-fmt-check";
          version = "0.1.0";
          src = ./.;
          cargoHash = packages.default.cargoHash;
          nativeBuildInputs = [ rustToolchain ];
          doBuild = ''
            cargo fmt --check
          '';
          doInstall = ''
            true
          '';
        };
      });

}
