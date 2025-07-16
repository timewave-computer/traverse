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


        
        # Create source derivations with ecosystem-specific workspace files
        coreSrc = pkgs.runCommand "core-source" {} ''
          cp -r ${./.} $out
          chmod -R +w $out
          cp $out/workspace-configs/Cargo.toml.core $out/Cargo.toml
          cp $out/workspace-configs/Cargo.lock.core $out/Cargo.lock
          # Clean up git and other development files
          rm -rf $out/.git $out/.github $out/target $out/result*
        '';

        # Core-only source (just traverse-core, no valence)
        coreOnlySrc = pkgs.runCommand "core-only-source" {} ''
          cp -r ${./.} $out
          chmod -R +w $out
          # Create core-only workspace configuration
          cat > $out/Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/traverse-core",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Timewave Labs"]
license = "Apache-2.0"
repository = "https://github.com/timewave-computer/traverse"
homepage = "https://github.com/timewave-computer/traverse"
description = "Chain-independent ZK storage path generator for blockchain state verification"
keywords = ["zk", "blockchain", "cosmos", "storage", "proof"]
categories = ["cryptography", "development-tools"]

[workspace.dependencies]
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }
hex = { version = "0.4", default-features = false, features = ["alloc"] }
sha2 = { version = "0.10", default-features = false }
keccak = { version = "0.1", default-features = false }
tiny-keccak = { version = "2.0", default-features = false, features = ["keccak"] }
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
proptest = "1.0"
tempfile = "3.0"
anyhow = { version = "1.0", default-features = false }
dotenv = "0.15"
toml = "0.8"
EOF
          # Remove Cargo.lock since we'll let Cargo generate it
          rm -f $out/Cargo.lock
          # Remove all non-core crates
          find $out/crates -name "traverse-*" -not -name "traverse-core" -type d -exec rm -rf {} +
          # Clean up git and other development files
          rm -rf $out/.git $out/.github $out/target $out/result*
        '';
        
        ethereumSrc = pkgs.runCommand "ethereum-source" {} ''
          cp -r ${./.} $out
          chmod -R +w $out
          cp $out/workspace-configs/Cargo.toml.ethereum $out/Cargo.toml
          cp $out/workspace-configs/Cargo.lock.ethereum $out/Cargo.lock
          # Remove Solana crate to avoid any conflicts
          rm -rf $out/crates/traverse-solana
          # Remove Cosmos crate to avoid potential conflicts
          rm -rf $out/crates/traverse-cosmos
          # Remove non-Ethereum CLI crates
          rm -rf $out/crates/traverse-cli-solana
          rm -rf $out/crates/traverse-cli-cosmos
          # Clean up git and other development files
          rm -rf $out/.git $out/.github $out/target $out/result*
        '';
        
        solanaSrc = pkgs.runCommand "solana-source" {} ''
          cp -r ${./.} $out
          chmod -R +w $out
          cp $out/workspace-configs/Cargo.toml.solana $out/Cargo.toml
          cp $out/workspace-configs/Cargo.lock.solana $out/Cargo.lock
          # Remove Ethereum crate to avoid any conflicts
          rm -rf $out/crates/traverse-ethereum
          # Remove Cosmos crate to avoid potential conflicts
          rm -rf $out/crates/traverse-cosmos
          # Remove non-Solana CLI crates
          rm -rf $out/crates/traverse-cli-ethereum
          rm -rf $out/crates/traverse-cli-cosmos
          # Clean up git and other development files
          rm -rf $out/.git $out/.github $out/target $out/result*
        '';

        # Cosmos source with cosmos workspace
        cosmosSrc = pkgs.runCommand "cosmos-source" {} ''
          cp -r ${./.} $out
          chmod -R +w $out
          cp $out/workspace-configs/Cargo.toml.cosmos $out/Cargo.toml
          cp $out/workspace-configs/Cargo.lock.cosmos $out/Cargo.lock
          # Remove other ecosystem crates to avoid conflicts
          rm -rf $out/crates/traverse-ethereum
          rm -rf $out/crates/traverse-solana
          # Remove non-Cosmos CLI crates
          rm -rf $out/crates/traverse-cli-ethereum
          rm -rf $out/crates/traverse-cli-solana
          # Clean up git and other development files
          rm -rf $out/.git $out/.github $out/target $out/result*
        '';

        # Full source for builds that need everything (currently unused)
        fullSrc = ./.;

        # Common build inputs for all ecosystems
        commonBuildInputs = with pkgs; [
          openssl
          # SSL certificate support
          cacert
          curl
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Native build inputs (available at build time)
        commonNativeBuildInputs = with pkgs; [
          pkg-config # pkg-config must be in nativeBuildInputs to be available during build
          git # Git must be available during build for cargo to fetch dependencies
        ];

        # Common args for all builds (without src - that's ecosystem-specific)
        commonArgs = {
          buildInputs = commonBuildInputs;
          nativeBuildInputs = commonNativeBuildInputs;
          strictDeps = true;
          cargoVendorDir = null; # Skip vendoring to avoid Cargo.lock conflicts
          # Override all cargo commands to avoid --locked flag
          cargoTestCommand = "cargo test --release";
          cargoCheckCommand = "cargo check --release";
          cargoBuildCommand = "cargo build --release";
          cargoTestExtraArgs = ""; # Ensure no --locked flag is passed to tests
          # SSL certificate configuration
          SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          CARGO_NET_GIT_FETCH_WITH_CLI = "true";
          # Git configuration for SSL
          GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          # OpenSSL configuration for cargo builds
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };

        # Core-only crates (just traverse-core)
        coreOnlyCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = coreOnlySrc;
          pname = "traverse-core-only-deps";
          cargoExtraArgs = "--package traverse-core";
        });

        # Core + Valence crates (shared by all ecosystems) - use core source filter
        coreCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = coreSrc;
          pname = "traverse-core-deps";
          cargoExtraArgs = "--package traverse-core --package traverse-valence";
        });

        # Ethereum ecosystem build (with Alloy dependencies) - use ethereum source filter
        ethereumCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = ethereumSrc;
          pname = "traverse-ethereum-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features ethereum,std --package traverse-ethereum --package traverse-valence";
        });

        # Solana ecosystem build (with Solana SDK dependencies) - use solana source filter
        solanaCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = solanaSrc;
          pname = "traverse-solana-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features solana,anchor --package traverse-solana";
        });

        # Cosmos ecosystem build
        cosmosCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = cosmosSrc;
          pname = "traverse-cosmos-deps";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features cosmos,std --package traverse-cosmos --package traverse-cli-core --package traverse-cli-cosmos";
        });

        # Ethereum CLI dependencies
        ethereumCliCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = ethereumSrc;
          pname = "traverse-ethereum-cli-deps";
          cargoArtifacts = ethereumCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features ethereum,std --package traverse-cli-core --package traverse-cli-ethereum";
        });

        # Solana CLI dependencies
        solanaCliCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = solanaSrc;
          pname = "traverse-solana-cli-deps";
          cargoArtifacts = solanaCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features solana,std,anchor --package traverse-cli-core --package traverse-cli-solana";
        });

        # Cosmos CLI dependencies
        cosmosCliCargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          src = cosmosSrc;
          pname = "traverse-cosmos-cli-deps";
          cargoArtifacts = cosmosCargoArtifacts;
          cargoExtraArgs = "--no-default-features --features cosmos,std --package traverse-cli-core --package traverse-cli-cosmos";
        });

      in
      {
        # Isolated ecosystem packages
        packages = {
          # Core package (shared)
                  traverse-core = craneLib.buildPackage (commonArgs // {
          src = coreSrc;
          pname = "traverse-core";
          cargoArtifacts = coreCargoArtifacts;
          cargoExtraArgs = "--package traverse-core";
          cargoTestCommand = "true"; # Skip tests during build
          doCheck = false; # Disable checks to avoid test compilation
        });

          # Ethereum ecosystem (Alloy-based)
          traverse-ethereum = craneLib.buildPackage (commonArgs // {
            src = ethereumSrc;
            pname = "traverse-ethereum";
            cargoArtifacts = ethereumCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features ethereum,std --package traverse-ethereum";
            cargoTestCommand = "true"; # Skip tests during build
            doCheck = false; # Disable checks to avoid test compilation
          });

          traverse-ethereum-cli = craneLib.buildPackage (commonArgs // {
            src = ethereumSrc;
            pname = "traverse-ethereum-cli";
            cargoArtifacts = ethereumCliCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features ethereum,std --bin traverse-ethereum -p traverse-cli-ethereum";
          });

          # Solana ecosystem (Solana SDK-based)
          traverse-solana = craneLib.buildPackage (commonArgs // {
            src = solanaSrc;
            pname = "traverse-solana";
            cargoArtifacts = solanaCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features solana,anchor --package traverse-solana";
            cargoTestCommand = "true"; # Skip tests during build
            doCheck = false; # Disable checks to avoid test compilation
          });

          traverse-solana-cli = craneLib.buildPackage (commonArgs // {
            src = solanaSrc;
            pname = "traverse-solana-cli";
            cargoArtifacts = solanaCliCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features solana,std,anchor --bin traverse-solana -p traverse-cli-solana";
            cargoTestCommand = "true"; # Skip tests during build
            doCheck = false; # Disable checks to avoid test compilation
          });

          # Cosmos ecosystem
          traverse-cosmos = craneLib.buildPackage (commonArgs // {
            src = cosmosSrc;
            pname = "traverse-cosmos";
            cargoArtifacts = cosmosCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features cosmos,std --package traverse-cosmos";
            cargoTestCommand = "true"; # Skip tests during build
            doCheck = false; # Disable checks to avoid test compilation
          });

          traverse-cosmos-cli = craneLib.buildPackage (commonArgs // {
            src = cosmosSrc;
            pname = "traverse-cosmos-cli";
            cargoArtifacts = cosmosCliCargoArtifacts;
            cargoExtraArgs = "--no-default-features --features cosmos,std --bin traverse-cosmos -p traverse-cli-cosmos";
          });

          # Default to core
          default = self.packages.${system}.traverse-core;
        };

        # Isolated test derivations for each ecosystem
        checks = {
          # Core tests (no ecosystem dependencies, no valence)
          traverse-core-tests = craneLib.cargoTest (commonArgs // {
            src = coreOnlySrc;
            pname = "traverse-core-tests";
            cargoArtifacts = coreOnlyCargoArtifacts;
            cargoTestExtraArgs = "--package traverse-core";
            cargoTestCommand = "cargo test --release --package traverse-core";
            # Override check phase to remove --locked flag
            checkPhase = ''
              runHook preCheck
              cargo test --release --package traverse-core
              runHook postCheck
            '';
            doCheck = true;
          });

          # Valence tests (with core dependencies)
          traverse-valence-tests = craneLib.cargoTest (commonArgs // {
            src = coreSrc;
            pname = "traverse-valence-tests";
            cargoArtifacts = coreCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features std,controller,circuit --package traverse-valence";
            cargoTestCommand = "cargo test --release --no-default-features --features std,controller,circuit --package traverse-valence";
            # Override check phase to remove --locked flag
            checkPhase = ''
              runHook preCheck
              cargo test --release --no-default-features --features std,controller,circuit --package traverse-valence
              runHook postCheck
            '';
            doCheck = true;
          });

          # Ethereum ecosystem tests (lib tests only)
          traverse-ethereum-tests = craneLib.cargoTest (commonArgs // {
            src = ethereumSrc;
            pname = "traverse-ethereum-tests";
            cargoArtifacts = ethereumCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features ethereum,std --package traverse-ethereum --lib";
            cargoTestCommand = "cargo test --release --no-default-features --features ethereum,std --package traverse-ethereum --lib";
            # Override check phase to remove --locked flag
            checkPhase = ''
              runHook preCheck
              cargo test --release --no-default-features --features ethereum,std --package traverse-ethereum --lib
              runHook postCheck
            '';
            doCheck = true;
          });

          # Solana ecosystem tests (fallback implementation, lib tests only)
          traverse-solana-tests = craneLib.cargoTest (commonArgs // {
            src = solanaSrc;
            pname = "traverse-solana-tests";
            cargoArtifacts = solanaCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features std --package traverse-solana --lib";
            cargoTestCommand = "cargo test --release --no-default-features --features std --package traverse-solana --lib";
            # Override check phase to remove --locked flag
            checkPhase = ''
              runHook preCheck
              cargo test --release --no-default-features --features std --package traverse-solana --lib
              runHook postCheck
            '';
            doCheck = true;
          });

          # Cosmos ecosystem tests (fallback implementation, lib tests only)
          traverse-cosmos-tests = craneLib.cargoTest (commonArgs // {
            src = cosmosSrc;
            pname = "traverse-cosmos-tests";
            cargoArtifacts = cosmosCargoArtifacts;
            cargoTestExtraArgs = "--no-default-features --features cosmos,std --package traverse-cosmos --lib";
            cargoTestCommand = "cargo test --release --no-default-features --features cosmos,std --package traverse-cosmos --lib";
            # Override check phase to remove --locked flag
            checkPhase = ''
              runHook preCheck
              cargo test --release --no-default-features --features cosmos,std --package traverse-cosmos --lib
              runHook postCheck
            '';
            doCheck = true;
          });

          # Valence tests (disabled - complex struct initialization issues)
          # traverse-valence-tests = craneLib.cargoTest (commonArgs // {
          #   src = coreSrc;
          #   pname = "traverse-valence-tests";
          #   cargoArtifacts = coreCargoArtifacts;
          #   cargoTestExtraArgs = "--no-default-features --features std,controller,circuit --package traverse-valence";
          # });
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
              echo "================================"
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
              echo "  nix build .#traverse-core-tests     # Core tests (no dependencies)"
              echo "  nix build .#traverse-valence-tests  # Valence tests (with alloy support)"
              echo "  nix build .#traverse-ethereum-tests # Ethereum ecosystem tests"
              echo "  nix build .#traverse-solana-tests   # Solana ecosystem tests"
              echo "  nix build .#traverse-cosmos-tests   # Cosmos ecosystem tests"
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
              git # Ensure git is available for cargo
            ]);

            # SSL certificate configuration for development
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            CARGO_NET_GIT_FETCH_WITH_CLI = "true";
            GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

            shellHook = ''
              echo "Traverse Ethereum Development Environment"
              echo "========================================="
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
              git # Ensure git is available for cargo
            ]) ++ (builtins.attrValues solanaTools);

            # SSL certificate configuration for development
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            CARGO_NET_GIT_FETCH_WITH_CLI = "true";
            GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

            shellHook = ''
              echo "Traverse Solana Development Environment"
              echo "======================================="
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
              git # Ensure git is available for cargo
            ]);

            # SSL certificate configuration for development
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            CARGO_NET_GIT_FETCH_WITH_CLI = "true";
            GIT_SSL_CAINFO = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";

            shellHook = ''
              echo "Traverse Cosmos Development Environment"
              echo "======================================="
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