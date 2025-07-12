# Feature Flags Guide

This document explains the feature flags system in Traverse, which allows you to build the library with only the components you need, optimizing for size, performance, and target environment compatibility.

## Overview

Traverse uses a comprehensive feature flag system that allows you to:
- Build minimal versions for resource-constrained environments
- Support no-std and WASM targets
- Include only the blockchain networks you need
- Optimize dependency footprint

## Core Feature Flags

### `std` (default)
Enables standard library support with full functionality.

**Includes:**
- Full file system access
- Network operations (tokio, reqwest)
- Complete error handling
- All CLI functionality

**Use when:** Building for desktop, server, or development environments.

```toml
[dependencies]
traverse = { version = "0.1", features = ["std"] }
```

### `no-std`
Minimal no-std build for embedded and circuit environments.

**Includes:**
- Core types only
- Basic storage path resolution
- No network operations
- No file system access
- Alloc-based memory management

**Use when:** Building for ZK circuits, embedded systems, or resource-constrained environments.

```toml
[dependencies]
traverse = { version = "0.1", features = ["no-std"] }
```

### `constrained`
Enhanced no-std build optimized for memory-constrained environments.

**Includes:**
- Compact data structures
- Memory pool management
- Stack-based operations
- Constrained circuit processing
- Fixed-size witness handling

**Use when:** Building for embedded systems, WASM runtimes, or environments with strict memory limits.

```toml
[dependencies]
traverse = { version = "0.1", features = ["constrained"] }
```

### `embedded`
Specialized build for embedded systems and microcontrollers.

**Includes:**
- Constrained environment optimizations
- Minimal memory footprint
- Hardware-friendly data structures
- Interrupt-safe operations

**Use when:** Building for microcontrollers, IoT devices, or bare-metal systems.

```toml
[dependencies]
traverse = { version = "0.1", features = ["embedded"] }
```

### `minimal`
Lightweight build with essential functionality only.

**Includes:**
- Core types and traits
- Basic serialization
- No heavy dependencies

**Use when:** You need basic functionality without the full feature set.

```toml
[dependencies]
traverse = { version = "0.1", features = ["minimal"] }
```

### `wasm`
WebAssembly-compatible build with browser support.

**Includes:**
- WASM-compatible dependencies
- JSON serialization
- Web-friendly APIs

**Use when:** Building for web browsers or WASM runtimes.

```toml
[dependencies]
traverse = { version = "0.1", features = ["wasm"] }
```

## Chain-Specific Features

### `lightweight`
Enables lightweight alloy integration with selective imports through the `lightweight-alloy` feature in traverse-ethereum.

**Includes:**
- Essential alloy primitives only (Address, B256, U256, Bytes)
- ABI encoding/decoding functionality
- Essential RPC types for storage proofs
- Basic provider functionality
- HTTP transport layer

**Selective Imports:**
- `alloy-primitives`: Core primitive types
- `alloy-sol-types`: ABI encoding/decoding
- `alloy-rpc-types-eth`: RPC types for storage proofs
- `alloy-provider`: Basic provider functionality
- `alloy-transport-http`: HTTP transport

**Benefits:**
- 50-70% faster compilation
- 40-60% smaller binary size
- Full type compatibility with alloy ecosystem
- Easy to upgrade when needed

**Use when:** You want Ethereum support with minimal overhead and fast compilation.

```toml
[dependencies]
traverse = { version = "0.1", features = ["lightweight", "ethereum"] }
```

### `ethereum` (default)
Enables Ethereum blockchain support with lightweight alloy integration.

**Includes:**
- Solidity ABI parsing
- Storage layout compilation
- Keccak256 hashing
- Lightweight alloy selective imports

**Use when:** Working with Ethereum contracts (recommended approach).

```toml
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }
```

### Full Alloy Support
For full alloy ecosystem support, you can enable all alloy crates directly in your dependency:

**Note:** traverse-ethereum uses lightweight-alloy by default for better performance. If you need the full alloy ecosystem, add alloy as a direct dependency in your project:

```toml
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }
alloy = { version = "0.9", features = ["full"] }
```

This gives you access to the complete alloy feature set while still using traverse for storage path generation.

### `cosmos`
Enables Cosmos/CosmWasm blockchain support.

**Includes:**
- CosmWasm schema parsing
- Cosmos storage patterns
- IBC proof verification
- Cosmos RPC client (with `client` feature)

**Use when:** Working with Cosmos or CosmWasm contracts.

```toml
[dependencies]
traverse = { version = "0.1", features = ["cosmos"] }
```

## Integration Features

### `client`
Enables live blockchain integration.

**Includes:**
- HTTP clients for RPC calls
- Live proof generation
- Real-time data fetching

**Use when:** You need to fetch live data from blockchains.

```toml
[dependencies]
traverse = { version = "0.1", features = ["client", "ethereum"] }
```

### `examples`
Enables example code and demonstrations.

**Includes:**
- Example applications
- Demo contracts
- Integration guides

**Use when:** Learning or developing with traverse.

```toml
[dependencies]
traverse = { version = "0.1", features = ["examples", "ethereum"] }
```

## Common Use Cases

### ZK Circuit Integration
For use in zero-knowledge circuits:

```toml
[dependencies]
traverse-core = { version = "0.1", features = ["no-std"] }
traverse-valence = { version = "0.1", features = ["no-std"] }
```

### CLI Tool
For building the full CLI tool:

```toml
[dependencies]
traverse-cli = { version = "0.1", features = ["std", "ethereum", "cosmos", "client"] }
```

### Web Application
For browser-based applications:

```toml
[dependencies]
traverse = { version = "0.1", features = ["wasm", "ethereum"] }
```

### Server Application
For server-side applications with live blockchain access:

```toml
[dependencies]
traverse = { version = "0.1", features = ["std", "ethereum", "client"] }
```

### Embedded System
For resource-constrained environments:

```toml
[dependencies]
traverse-core = { version = "0.1", features = ["embedded"] }
```

### Memory-Constrained Circuit
For ZK circuits with strict memory limits:

```toml
[dependencies]
traverse-core = { version = "0.1", features = ["constrained"] }
traverse-valence = { version = "0.1", features = ["constrained"] }
```

### Microcontroller
For bare-metal embedded systems:

```toml
[dependencies]
traverse-core = { version = "0.1", features = ["embedded"] }
```

## Version Compatibility

All dependencies use flexible version ranges to ensure compatibility:

- **Alloy**: `>=0.9.0,<2.0` - Supports Alloy 0.9+ through 1.x
- **Serde**: `>=1.0.0,<2.0` - Supports all Serde 1.x versions
- **Tokio**: `>=1.0.0,<2.0` - Supports all Tokio 1.x versions

This allows you to upgrade dependencies within major version ranges without breaking compatibility.

## Feature Combinations

### Valid Combinations

```toml
# Full-featured development
traverse = { features = ["std", "ethereum", "cosmos", "client", "examples"] }

# Production server
traverse = { features = ["std", "ethereum", "client"] }

# Browser application
traverse = { features = ["wasm", "ethereum"] }

# ZK circuit
traverse-core = { features = ["no-std"] }
traverse-valence = { features = ["no-std"] }

# Minimal CLI
traverse-cli = { features = ["minimal", "ethereum"] }
```

### Invalid Combinations

```toml
# ❌ Cannot combine std and no-std
traverse = { features = ["std", "no-std"] }

# ❌ Client requires std
traverse = { features = ["no-std", "client"] }

# ❌ Examples require std
traverse = { features = ["minimal", "examples"] }
```

## Migration Guide

### From Previous Versions

If you were using traverse without explicit features:

**Before:**
```toml
[dependencies]
traverse = "0.1"
```

**After:**
```toml
[dependencies]
traverse = { version = "0.1", features = ["std", "ethereum"] }
```

### For Specific Use Cases

**CLI Development:**
```toml
# Before
traverse-cli = "0.1"

# After
traverse-cli = { version = "0.1", features = ["std", "ethereum", "client"] }
```

**Circuit Integration:**
```toml
# Before
traverse-valence = "0.1"

# After
traverse-valence = { version = "0.1", features = ["no-std"] }
```

## Building

### Check Available Features

```bash
# List all available features
cargo metadata --format-version=1 | jq '.packages[] | select(.name == "traverse") | .features'
```

### Build with Specific Features

```bash
# Build with ethereum support only
cargo build --features ethereum

# Build minimal version
cargo build --features minimal --no-default-features

# Build for WASM
cargo build --target wasm32-unknown-unknown --features wasm --no-default-features
```

### Test with Features

```bash
# Test with all features
cargo test --all-features

# Test minimal build
cargo test --features minimal --no-default-features

# Test no-std build
cargo test --features no-std --no-default-features
```

## Troubleshooting

### Common Issues

**Error: "feature not found"**
- Ensure the feature exists in the crate you're using
- Check if the feature requires other features to be enabled

**Error: "dependency not found"**
- Some features make dependencies optional
- Enable the appropriate feature flag

**Error: "std not available"**
- You're trying to use std features in a no-std build
- Use `no-std` or `minimal` features instead

### Getting Help

If you encounter issues with feature flags:

1. Check the feature documentation for your specific crate
2. Look at the examples in the repository
3. File an issue with your use case and target environment

## Performance Notes

### Binary Size Impact

| Feature Set | Approximate Binary Size |
|-------------|------------------------|
| `no-std` | ~50KB |
| `constrained` | ~75KB |
| `embedded` | ~60KB |
| `minimal` | ~200KB |
| `wasm` | ~500KB |
| `std` | ~2MB |
| `lightweight` | ~3MB |
| `ethereum` (lightweight) | ~3.5MB |
| `ethereum-full` | ~6MB |
| `std + client` | ~5MB |
| `std + client + examples` | ~10MB |

### Compile Time Impact

| Feature Set | Approximate Compile Time |
|-------------|-------------------------|
| `no-std` | ~30s |
| `constrained` | ~45s |
| `embedded` | ~35s |
| `minimal` | ~1m |
| `wasm` | ~1m 30s |
| `std` | ~2m |
| `lightweight` | ~2m 30s |
| `ethereum` (lightweight) | ~3m |
| `ethereum-full` | ~5m |
| `std + client` | ~3m |
| `std + client + examples` | ~5m |

### Alloy Integration Comparison

| Integration Level | Dependencies | Compile Time | Binary Size | Use Case |
|------------------|--------------|--------------|-------------|----------|
| **Fallback** | None | Fastest | Smallest | Basic encoding only |
| **Lightweight** (`lightweight-alloy`) | alloy-primitives, alloy-sol-types, alloy-rpc-types-eth, alloy-provider, alloy-transport-http | ~50% faster than full | ~40% smaller than full | Recommended for most apps |
| **Full Alloy** | Complete alloy ecosystem (add as direct dependency) | Slowest | Largest | Advanced alloy features needed |

### Dependency Count Comparison

| Feature Set | Total Dependencies | Alloy Components |
|-------------|-------------------|------------------|
| `minimal` | ~15 | 0 |
| `lightweight` | ~35 | 3 minimal |
| `ethereum-full` | ~80+ | Complete ecosystem |

These are approximate values and will vary based on your hardware and caching. 