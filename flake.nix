{
  description = "Traverse - Chain-Independent ZK Storage Path Generator with Isolated Ecosystem Builds";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Import Solana support from zero.nix for e2e testing
    # zero-nix = {
    #   url = "github:timewave-computer/zero.nix/sam-solana";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        
        # Import Solana tools from zero.nix for e2e testing
        solanaTools = {}; # zero-nix.packages.${system} or {};

        # Common source filtering - include all workspace members
        src = craneLib.cleanCargoSource ./.;

        # Common build inputs for all ecosystems
        commonBuildInputs = with pkgs; [
          pkg-config
          openssl
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Common args for all builds
        commonArgs = {
          inherit src;
          buildInputs = commonBuildInputs;
          strictDeps = true;
          cargoVendorDir = null; # Skip vendoring to avoid Cargo.lock conflicts
        };

        # Core crates (shared by all ecosystems)
        coreCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "traverse-core-deps";
          cargoExtraArgs = "--package traverse-core";
        });

        # Ethereum ecosystem build (with Alloy dependencies)
        ethereumCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "traverse-ethereum-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features ethereum --package traverse-ethereum --package traverse-valence --package traverse-cli";
        });

        # Solana ecosystem build (with Solana SDK dependencies) 
        solanaCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "traverse-solana-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features solana --package traverse-solana --package traverse-cli";
        });

        # Cosmos ecosystem build
        cosmosCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "traverse-cosmos-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features cosmos --package traverse-cosmos --package traverse-cli";
        });

      in
      {
        # Isolated ecosystem packages
        packages = {
          # Core package (shared)
          traverse-core = craneLib.buildPackage (commonArgs // {
            pname = "traverse-core";
            cargoArtifacts = coreCargoArtifacts;
            cargoExtraArgs = "--package traverse-core";
          });

          # Ethereum ecosystem (Alloy-based)
          traverse-ethereum = craneLib.buildPackage (commonArgs // {
            pname = "traverse-ethereum";
            cargoArtifacts = ethereumCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features ethereum --package traverse-ethereum";
          });

          traverse-ethereum-cli = craneLib.buildPackage (commonArgs // {
            pname = "traverse-ethereum-cli";
            cargoArtifacts = ethereumCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features ethereum,codegen --bin traverse-cli";
          });

          # Solana ecosystem (Solana SDK-based)
          traverse-solana = craneLib.buildPackage (commonArgs // {
            pname = "traverse-solana";
            cargoArtifacts = solanaCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features solana --package traverse-solana";
          });

          traverse-solana-cli = craneLib.buildPackage (commonArgs // {
            pname = "traverse-solana-cli";
            cargoArtifacts = solanaCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features solana,codegen --bin traverse-cli";
          });

          # Cosmos ecosystem
          traverse-cosmos = craneLib.buildPackage (commonArgs // {
            pname = "traverse-cosmos";
            cargoArtifacts = cosmosCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features cosmos --package traverse-cosmos";
          });

          traverse-cosmos-cli = craneLib.buildPackage (commonArgs // {
            pname = "traverse-cosmos-cli";
            cargoArtifacts = cosmosCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features cosmos,codegen --bin traverse-cli";
          });

          # Default to core
          default = self.packages.${system}.traverse-core;
        };

        # Isolated test derivations for each ecosystem
        checks = {
          # Core tests (no ecosystem dependencies)
          traverse-core-tests = craneLib.cargoTest (commonArgs // {
            pname = "traverse-core-tests";
            cargoArtifacts = coreCargoArtifacts;
            cargoTestExtraArgs = "--package traverse-core";
          });

          # Ethereum ecosystem tests
          traverse-ethereum-tests = craneLib.cargoTest (commonArgs // {
            pname = "traverse-ethereum-tests";
            cargoArtifacts = ethereumCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features ethereum --package traverse-ethereum";
          });

          # Solana ecosystem tests  
          traverse-solana-tests = craneLib.cargoTest (commonArgs // {
            pname = "traverse-solana-tests";
            cargoArtifacts = solanaCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features solana --package traverse-solana";
          });

          # Cosmos ecosystem tests
          traverse-cosmos-tests = craneLib.cargoTest (commonArgs // {
            pname = "traverse-cosmos-tests";
            cargoArtifacts = cosmosCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features cosmos --package traverse-cosmos";
          });

          # Valence tests (no alloy features)
          traverse-valence-tests = craneLib.cargoTest (commonArgs // {
            pname = "traverse-valence-tests";
            cargoArtifacts = coreCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features std,controller,circuit --package traverse-valence";
          });
        };

        # Ecosystem-specific development shells
        devShells = {
          # Default shell (core development)
          default = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ (with pkgs; [
              rustToolchain
              cargo-edit
              cargo-watch
              cargo-nextest
              just
              jq
              nodejs
              direnv
              nix-direnv
            ]);

            shellHook = ''
              echo "Traverse development environment"
              echo "==============================="
              echo ""
              echo "Rust toolchain:"
              echo "  cargo --version       # $(cargo --version)"
              echo "  rustc --version       # $(rustc --version)"
              echo ""
              echo "Available isolated builds:"
              echo "  nix build .#traverse-core          # Core implementation"
              echo "  nix build .#traverse-ethereum      # Ethereum ecosystem"
              echo "  nix build .#traverse-ethereum-cli  # Ethereum CLI"
              echo "  nix build .#traverse-solana        # Solana ecosystem"
              echo "  nix build .#traverse-solana-cli    # Solana CLI"
              echo "  nix build .#traverse-cosmos        # Cosmos ecosystem"
              echo "  nix build .#traverse-cosmos-cli    # Cosmos CLI"
              echo ""
              echo "Isolated ecosystem tests:"
              echo "  nix build .#traverse-core-tests     # Core tests (no conflicts)"
              echo "  nix build .#traverse-ethereum-tests # Ethereum ecosystem tests"
              echo "  nix build .#traverse-solana-tests   # Solana ecosystem tests"
              echo "  nix build .#traverse-cosmos-tests   # Cosmos ecosystem tests"
              echo "  nix build .#traverse-valence-tests  # Valence tests (no alloy)"
              echo ""
              echo "Run all ecosystem tests:"
              echo "  nix flake check                     # Run all isolated tests"
              echo ""
              echo "Ecosystem-specific shells:"
              echo "  nix develop .#ethereum  # Ethereum development"
              echo "  nix develop .#solana    # Solana development"
              echo "  nix develop .#cosmos    # Cosmos development"
              echo ""
            '';
          };

          # Ethereum development shell
          ethereum = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ (with pkgs; [
              rustToolchain
              cargo-edit
              cargo-watch
              cargo-nextest
              just
              jq
              nodejs
            ]);

            shellHook = ''
              echo "Traverse Ethereum Development Environment"
              echo "========================================"
              echo ""
              echo "Build commands:"
              echo "  cargo build --no-default-features --features ethereum"
              echo "  nix build .#traverse-ethereum-cli"
              echo ""
              echo "Test commands:"
              echo "  nix build .#traverse-ethereum-tests # Isolated tests"
              echo "  cargo test --no-default-features --features ethereum # (may fail due to workspace conflicts)"
              echo ""
            '';
          };

          # Solana development shell  
          solana = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ (with pkgs; [
              rustToolchain
              cargo-edit
              cargo-watch
              cargo-nextest
              just
              jq
              nodejs
            ]) ++ (builtins.attrValues solanaTools);

            shellHook = ''
              echo "Traverse Solana Development Environment"
              echo "======================================"
              echo ""
              ${if (solanaTools != {}) then ''
              echo "Solana development tools:"
              ${if (solanaTools ? solana-cli) then ''echo "  solana --version       # Solana CLI"'' else ""}
              ${if (solanaTools ? anchor-cli) then ''echo "  anchor --version       # Anchor framework CLI"'' else ""}
              ${if (solanaTools ? solana-test-validator) then ''echo "  solana-test-validator  # Local Solana validator"'' else ""}
              echo ""
              '' else ''
              echo "Note: Solana tools not available (check zero.nix integration)"
              echo ""
              ''}
              echo "Build commands:"
              echo "  cargo build --no-default-features --features solana"
              echo "  nix build .#traverse-solana-cli"
              echo ""
              echo "Test commands:"
              echo "  nix build .#traverse-solana-tests   # Isolated tests"
              echo "  cargo test --no-default-features --features solana # (may fail due to workspace conflicts)"
              echo ""
              # Set environment variables for Solana development
              export SOLANA_URL="http://127.0.0.1:8899"  # Local test validator
              export ANCHOR_PROVIDER_URL="$SOLANA_URL"
              export SOLANA_CLI_CONFIG="$HOME/.config/solana/cli/config.yml"
            '';
          };

          # Cosmos development shell
          cosmos = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ (with pkgs; [
              rustToolchain
              cargo-edit
              cargo-watch
              cargo-nextest
              just
              jq
              nodejs
            ]);

            shellHook = ''
              echo "Traverse Cosmos Development Environment"
              echo "======================================"
              echo ""
              echo "Build commands:"
              echo "  cargo build --no-default-features --features cosmos"
              echo "  nix build .#traverse-cosmos-cli"
              echo ""
              echo "Test commands:"
              echo "  nix build .#traverse-cosmos-tests   # Isolated tests"
              echo "  cargo test --no-default-features --features cosmos # (may fail due to workspace conflicts)"
              echo ""
            '';
          };
        };
      }
    );
}