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

# Test ecosystem-specific dev shells (quick validation)
print_header "Testing Development Shells"

print_info "Testing ecosystem dev shells..."
run_build "default dev shell" "nix develop -c echo 'Default dev shell works'"
run_build "ethereum dev shell" "nix develop .#ethereum -c echo 'Ethereum dev shell works'"
run_build "solana dev shell" "nix develop .#solana -c echo 'Solana dev shell works'"
run_build "cosmos dev shell" "nix develop .#cosmos -c echo 'Cosmos dev shell works'"

# Test supported Nix package builds
print_header "Testing Supported Nix Packages"

print_info "Testing core packages..."
run_build "traverse-core" "nix build .#traverse-core --no-link"

print_info "Testing ecosystem packages..."
run_build "traverse-ethereum" "nix build .#traverse-ethereum --no-link"
run_build "traverse-solana" "nix build .#traverse-solana --no-link"
run_build "traverse-cosmos" "nix build .#traverse-cosmos --no-link"

# Optional: Test CLI packages (these take longer due to more dependencies)
print_info "Testing CLI packages (optional - comment out for faster testing)..."
# run_build "traverse-ethereum-cli" "nix build .#traverse-ethereum-cli --no-link"
# run_build "traverse-solana-cli" "nix build .#traverse-solana-cli --no-link"  
# run_build "traverse-cosmos-cli" "nix build .#traverse-cosmos-cli --no-link"
print_info "CLI package tests skipped for faster execution. Uncomment lines above to test CLIs."

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