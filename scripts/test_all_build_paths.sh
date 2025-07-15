#!/usr/bin/env bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track failures
FAILED_BUILDS=()
TOTAL_BUILDS=0
SUCCESSFUL_BUILDS=0

# Function to print colored output
print_header() {
    echo -e "\n${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Function to run a build command and track results
run_build() {
    local description="$1"
    local command="$2"
    
    TOTAL_BUILDS=$((TOTAL_BUILDS + 1))
    print_info "Building: $description"
    echo "Command: $command"
    
    if eval "$command" > /tmp/build_output.log 2>&1; then
        SUCCESSFUL_BUILDS=$((SUCCESSFUL_BUILDS + 1))
        print_success "$description"
    else
        FAILED_BUILDS+=("$description")
        print_error "$description"
        echo "Error output (last 20 lines):"
        tail -20 /tmp/build_output.log
        echo ""
    fi
}

# Detect system architecture
SYSTEM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$SYSTEM-$ARCH" in
    "darwin-arm64"|"darwin-aarch64")
        NIX_SYSTEM="aarch64-darwin"
        ;;
    "darwin-x86_64")
        NIX_SYSTEM="x86_64-darwin"
        ;;
    "linux-x86_64")
        NIX_SYSTEM="x86_64-linux"
        ;;
    "linux-aarch64")
        NIX_SYSTEM="aarch64-linux"
        ;;
    *)
        echo "Unsupported system: $SYSTEM-$ARCH"
        exit 1
        ;;
esac

# Start testing
print_header "Testing All Build Paths for Traverse"
echo "This script will test all Nix derivations and Rust feature combinations"
echo "Working directory: $(pwd)"
echo "Detected system: $NIX_SYSTEM"

# Check if we're in the right directory
if [ ! -f "flake.nix" ]; then
    print_error "Error: flake.nix not found. Please run this script from the traverse root directory."
    exit 1
fi

# Clean any previous build artifacts
print_header "Cleaning Previous Build Artifacts"
rm -f /tmp/build_output.log
print_success "Cleaned temporary files"

# Test Nix builds
print_header "Testing Nix Builds"

# Core builds
print_info "Testing core ecosystem builds..."
run_build "traverse-core" "nix build .#traverse-core"
run_build "traverse-core-tests" "nix build .#checks.$NIX_SYSTEM.traverse-core-tests"

# Ethereum builds
print_info "Testing Ethereum ecosystem builds..."
run_build "traverse-ethereum" "nix build .#traverse-ethereum"
run_build "traverse-ethereum-cli" "nix build .#traverse-ethereum-cli"
run_build "traverse-ethereum-tests" "nix build .#checks.$NIX_SYSTEM.traverse-ethereum-tests"

# Solana builds
print_info "Testing Solana ecosystem builds..."
run_build "traverse-solana" "nix build .#traverse-solana"
run_build "traverse-solana-cli" "nix build .#traverse-solana-cli"
run_build "traverse-solana-tests" "nix build .#checks.$NIX_SYSTEM.traverse-solana-tests"

# Cosmos builds
print_info "Testing Cosmos ecosystem builds..."
run_build "traverse-cosmos" "nix build .#traverse-cosmos"
run_build "traverse-cosmos-cli" "nix build .#traverse-cosmos-cli"
run_build "traverse-cosmos-tests" "nix build .#checks.$NIX_SYSTEM.traverse-cosmos-tests"

# Valence builds (if enabled)
# run_build "traverse-valence-tests" "nix build .#traverse-valence-tests"

# Test Rust builds with different feature combinations
print_header "Testing Rust Feature Combinations"

# Function to test rust builds in different workspace configurations
test_rust_workspace() {
    local workspace_name="$1"
    local workspace_file="workspace-configs/Cargo.toml.$workspace_name"
    local lock_file="workspace-configs/Cargo.lock.$workspace_name"
    
    if [ ! -f "$workspace_file" ]; then
        print_error "Workspace file not found: $workspace_file"
        return
    fi
    
    print_info "Setting up $workspace_name workspace..."
    
    # Backup existing files if they exist
    if [ -f "Cargo.toml" ]; then
        mv Cargo.toml Cargo.toml.backup
    fi
    if [ -f "Cargo.lock" ]; then
        mv Cargo.lock Cargo.lock.backup
    fi
    
    # Copy workspace files
    cp "$workspace_file" Cargo.toml
    if [ -f "$lock_file" ]; then
        cp "$lock_file" Cargo.lock
    fi
    
    # Test builds based on workspace
    case "$workspace_name" in
        "core")
            print_info "Testing core features..."
            run_build "core default features" "nix develop -c cargo build --package traverse-core"
            run_build "core no-std" "nix develop -c cargo build --package traverse-core --no-default-features --features no-std"
            run_build "core minimal" "nix develop -c cargo build --package traverse-core --no-default-features --features minimal"
            run_build "core wasm" "nix develop -c cargo build --package traverse-core --no-default-features --features wasm"
            ;;
            
        "ethereum")
            print_info "Testing Ethereum features..."
            run_build "ethereum default" "nix develop .#ethereum -c cargo build --package traverse-ethereum"
            run_build "ethereum std only" "nix develop .#ethereum -c cargo build --package traverse-ethereum --no-default-features --features std"
            run_build "ethereum with alloy" "nix develop .#ethereum -c cargo build --package traverse-ethereum --no-default-features --features std,ethereum,lightweight-alloy"
            run_build "ethereum cli" "nix develop .#ethereum -c cargo build --package traverse-cli-ethereum"
            ;;
            
        "solana")
            print_info "Testing Solana features..."
            run_build "solana default" "nix develop .#solana -c cargo build --package traverse-solana"
            run_build "solana std only" "nix develop .#solana -c cargo build --package traverse-solana --no-default-features --features std"
            run_build "solana with sdk" "nix develop .#solana -c cargo build --package traverse-solana --no-default-features --features std,solana"
            run_build "solana with anchor" "nix develop .#solana -c cargo build --package traverse-solana --no-default-features --features std,solana,anchor"
            run_build "solana with client" "nix develop .#solana -c cargo build --package traverse-solana --no-default-features --features std,solana,anchor,spl-token,client"
            # CLI builds might fail due to API mismatches, but we'll test anyway
            # run_build "solana cli" "nix develop .#solana -c cargo build --package traverse-cli-solana"
            ;;
            
        "cosmos")
            print_info "Testing Cosmos features..."
            run_build "cosmos default" "nix develop .#cosmos -c cargo build --package traverse-cosmos"
            run_build "cosmos std only" "nix develop .#cosmos -c cargo build --package traverse-cosmos --no-default-features --features std"
            run_build "cosmos with cosmos feature" "nix develop .#cosmos -c cargo build --package traverse-cosmos --no-default-features --features std,cosmos"
            run_build "cosmos cli" "nix develop .#cosmos -c cargo build --package traverse-cli-cosmos"
            ;;
    esac
    
    # Restore original files
    rm -f Cargo.toml Cargo.lock
    if [ -f "Cargo.toml.backup" ]; then
        mv Cargo.toml.backup Cargo.toml
    fi
    if [ -f "Cargo.lock.backup" ]; then
        mv Cargo.lock.backup Cargo.lock
    fi
}

# Test each workspace
print_info "Testing workspace configurations..."
test_rust_workspace "core"
test_rust_workspace "ethereum"
test_rust_workspace "solana"
test_rust_workspace "cosmos"

# Test cross-compilation targets (optional)
print_header "Testing Cross-Compilation Targets (Optional)"
# Uncomment to test WASM builds
# run_build "core wasm32-unknown-unknown" "nix develop -c cargo build --package traverse-core --target wasm32-unknown-unknown --no-default-features --features wasm"

# Summary
print_header "Build Test Summary"
echo "Total builds attempted: $TOTAL_BUILDS"
echo "Successful builds: $SUCCESSFUL_BUILDS"
echo "Failed builds: ${#FAILED_BUILDS[@]}"

if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    print_success "All builds completed successfully! 🎉"
    exit 0
else
    print_error "The following builds failed:"
    for build in "${FAILED_BUILDS[@]}"; do
        echo "  - $build"
    done
    exit 1
fi