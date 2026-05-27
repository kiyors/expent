{
  description = "A Nix-flake-based Rust and Node.js development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, ... }@inputs:

    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        # Rust Toolchain setup via fenix
        rustToolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine (
            with stable;
            [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
              targets.wasm32-unknown-unknown.stable.rust-std
            ]
          );

        # Node.js passthrough
        nodejs = prev.nodejs;
      };

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages =
              with pkgs;
              [
                # Rust
                rustToolchain
                openssl
                pkg-config
                rust-analyzer
                bacon

                # Node.js
                nodejs
                pnpm
                biome

                # Utilities
                just
                taplo
              ]
              ++ lib.optionals stdenv.isDarwin [
                libiconv
              ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };

            # Automatically creates/activates the uv venv
            shellHook = ''
              echo "Loading Hybrid Rust, Python, and Node.js Dev Environment"

              # Node Setup
              export PATH="$PWD/node_modules/.bin:$PATH"

              # Display versions
              echo "Versions:"
              echo "  rust:   $(cargo --version)"
              echo "  node:   $(node --version)"
              echo "  pnpm:   $(pnpm --version)"

            '';
          };
        }
      );
    };
}
