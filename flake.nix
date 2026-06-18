{
  description = "A Nix-flake-based Rust and Node.js development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    sops-nix = {
      url = "github:Mic92/sops-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks-nix = {
      url = "github:cachix/git-hooks.nix";
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
            inherit system;
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

      checks = forEachSupportedSystem (
        { pkgs, system }: {
          pre-commit-check = inputs.git-hooks-nix.lib.${system}.run {
            src = ./.;
            hooks = {
              encrypt-env = {
                enable = true;
                name = "Encrypt .env to secrets.env";
                entry = "bash -c 'if [ -f .env ]; then cp .env secrets.env && ${pkgs.sops}/bin/sops -e -i secrets.env && git add secrets.env; fi'";
                pass_filenames = false;
              };
              fmt-all = {
                enable = true;
                name = "Run pnpm fmt-all";
                entry = "bash -c 'pnpm fmt-all'";
                pass_filenames = false;
              };
            };
          };
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs, system }:
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
                sops
                age
              ]
              ++ lib.optionals stdenv.isDarwin [
                libiconv
              ];

            # Leave buildInputs empty on Darwin so Nix doesn't hijack the SDK
            buildInputs = [ ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
            };

            # Automatically creates/activates the uv venv
            shellHook = ''
              # 1. Unset the Nix-injected SDK root so xcrun falls back to the host system
              unset SDKROOT
              unset DEVELOPER_DIR

              # 2. Re-assert the true system binary paths ahead of the Nix sandbox
              export PATH="/usr/bin:/usr/sbin:/usr/local/bin:$PATH"

              echo "Loading Hybrid Rust, Python, and Node.js Dev Environment"

              # Node Setup
              export PATH="$PWD/node_modules/.bin:$PATH"

              # Display versions
              echo "Versions:"
              echo "  rust:   $(cargo --version)"
              echo "  node:   $(node --version)"
              echo "  pnpm:   $(pnpm --version)"

              ${inputs.self.checks.${system}.pre-commit-check.shellHook}

              # Auto-decrypt secrets.env on clone or if secrets.env is newer than .env (like after git pull)
              if [ -f secrets.env ] && { [ ! -f .env ] || [ secrets.env -nt .env ]; }; then
                echo "🔓 Decrypting updated secrets.env to .env..."
                if content=$(sops -d secrets.env 2>/dev/null); then
                  echo "$content" > .env
                else
                  echo "⚠️  Skipping decryption: Missing age key."
                fi
              fi
            '';
          };
        }
      );
    };
}
