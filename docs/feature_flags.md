# Feature Flags & Build System Guide

This document explains Traverse's feature flag system and the isolated build approach used to handle incompatible blockchain dependencies.

## Dependency Conflict Resolution

### The Problem

The Rust blockchain ecosystem has incompatible dependencies between different chains:

| Ecosystem | Library | k256 Version | secp256k1 Version |
|-----------|---------|--------------|-------------------|
| **Ethereum (Alloy)** | `alloy-*` | `^0.14` | `^0.29` |
| **Solana** | `solana-sdk` | `^0.13` | `^0.28` |

These version conflicts make it impossible to include both Ethereum and Solana support in the same binary.

### The Solution: Isolated Builds

Traverse uses separate Cargo workspaces for each blockchain ecosystem:

```
workspace-configs/
├── Cargo.toml.core      # Core functionality only
├── Cargo.toml.ethereum   # Ethereum + core
├── Cargo.toml.solana     # Solana + core  
├── Cargo.toml.cosmos     # Cosmos + core
└── Cargo.lock.*         # Locked dependencies per workspace
```

The Nix build system automatically:
- Selects the appropriate workspace configuration
- Resolves dependencies independently per ecosystem
- Ensures reproducible builds with checked-in lock files
- Prevents accidental mixing of incompatible dependencies

## Core Feature Flags

### `std` (default)
Standard library support with full functionality.

**Use when:** Building for desktop, server, or development environments.

### `no-std`
Minimal build for embedded and circuit environments.

**Use when:** Building for ZK circuits or resource-constrained environments.

### `minimal`
Essential functionality without heavy dependencies.

**Use when:** You need basic functionality with fast compilation.

### `constrained`
Optimized for memory-constrained environments with fixed-size data structures.

**Use when:** Building for embedded systems or environments with strict memory limits.

### `wasm`
WebAssembly-compatible build with browser support.

**Use when:** Building for web browsers or WASM runtimes.

## Blockchain Features (Mutually Exclusive)

### `ethereum`
Ethereum/EVM support with lightweight Alloy integration.

**Includes:**
- Solidity ABI parsing
- Storage layout compilation
- MPT proof verification
- Keccak256 hashing

### `solana`
Solana blockchain support.

**Includes:**
- Anchor IDL parsing
- Account layout compilation
- Borsh serialization
- SPL token support

### `cosmos`
Cosmos/CosmWasm support.

**Includes:**
- CosmWasm schema parsing
- ICS23 proof verification
- Cosmos storage patterns

## Additional Features

### `client`
HTTP clients for live blockchain data (requires `std`).

### `lightweight-alloy`
Minimal Alloy dependencies for faster compilation (Ethereum only).

### `codegen`
Generate custom crates for specific storage layouts.

## Common Configurations

### ZK Circuit Integration
```toml
[dependencies]
traverse-core = { git = "https://github.com/timewave-computer/traverse", features = ["no-std"] }
traverse-valence = { git = "https://github.com/timewave-computer/traverse", features = ["constrained"] }
```

### Ethereum Application
```toml
[dependencies]
traverse-ethereum = { git = "https://github.com/timewave-computer/traverse", features = ["std"] }
traverse-core = { git = "https://github.com/timewave-computer/traverse" }
```

### Solana Application
```toml
[dependencies]
traverse-solana = { git = "https://github.com/timewave-computer/traverse", features = ["std"] }
traverse-core = { git = "https://github.com/timewave-computer/traverse" }
```

## Building with Nix

### Individual Packages
```bash
nix build .#traverse-ethereum
nix build .#traverse-solana
nix build .#traverse-cosmos
```

### Running Tests
```bash
nix flake check                    # All ecosystems
nix build .#checks.$SYSTEM.traverse-ethereum-tests  # Specific ecosystem
```

### Development Shells
```bash
nix develop .#ethereum  # Ethereum development
nix develop .#solana    # Solana development  
nix develop .#cosmos    # Cosmos development
```

## Building with Cargo

When using Cargo directly, you must manually ensure only one blockchain ecosystem is enabled:

```bash
# Ethereum only
cargo build --features ethereum

# Solana only (disable defaults)
cargo build --features solana --no-default-features

# Test specific ecosystem
cargo test --features ethereum --package traverse-ethereum
```

### Using Nix for Multi-Ecosystem Projects

Create a flake.nix that builds separate binaries:

```nix
{
  packages = {
    ethereum-processor = traverse.packages.traverse-ethereum-cli;
    solana-processor = traverse.packages.traverse-solana-cli;
  };
}
```
