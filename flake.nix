{
  description = "Traverse - Chain-Independent ZK Storage Path Generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crate2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Import crate2nix tools
        crate2nixTools = pkgs.callPackage "${crate2nix}/tools.nix" {};
        
        # Check if Cargo.nix exists
        cargoNixExists = builtins.pathExists ./Cargo.nix;
        
        # Load crate2nix project if Cargo.nix exists
        cargoNix = if cargoNixExists 
          then import ./Cargo.nix {
            inherit pkgs;
            defaultCrateOverrides = pkgs.defaultCrateOverrides;
          }
          else null;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain
            
            # Essential development tools
            cargo-watch
            cargo-edit
            
            # System dependencies
            pkg-config
            openssl
            
            # Basic development tools
            git
          ];
          
          shellHook = ''
            echo "Traverse development environment"
            echo "Rust toolchain: $(rustc --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo run -- --help    # CLI help"
            echo "  cargo test             # Run tests"
            echo "  cargo build            # Build project"
            echo ""
            ${if cargoNixExists then ''
            echo "Nix build commands (Cargo.nix available):"
            echo "  nix build .#traverse-cli      # Build CLI binary"
            echo "  nix build .#traverse-core     # Build core library"
            echo "  nix build .#traverse-ethereum # Build Ethereum implementation"
            echo "  nix build .#traverse-valence  # Build valence integration"
            '' else ''
            echo "To enable nix builds:"
            echo "  crate2nix generate     # Generate Cargo.nix file"
            echo "  nix flake check        # Re-check after generation"
            ''}
            echo ""
            echo "Using crate2nix for reproducible Rust builds"
          '';
          
          # Environment variables
          RUST_BACKTRACE = "1";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      }
      // (if cargoNixExists then {
        # Package all workspace members using crate2nix (only if Cargo.nix exists)
        packages = {
          default = cargoNix.workspaceMembers.traverse-cli.build;
          traverse-cli = cargoNix.workspaceMembers.traverse-cli.build;
          traverse-core = cargoNix.workspaceMembers.traverse-core.build;
          traverse-ethereum = cargoNix.workspaceMembers.traverse-ethereum.build;
          traverse-valence = cargoNix.workspaceMembers.traverse-valence.build;
        };

        # Apps for easy running
        apps = {
          default = flake-utils.lib.mkApp {
            drv = self.packages.${system}.traverse-cli;
          };
          
          traverse-cli = flake-utils.lib.mkApp {
            drv = self.packages.${system}.traverse-cli;
          };
        };
      } else {})
    );
} 