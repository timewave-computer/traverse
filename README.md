# Traverse

Chain-independent ZK storage proof system for blockchain state verification.

## Overview

Traverse generates cryptographic proofs of blockchain storage state for use in zero-knowledge circuits and cross-chain applications. It provides semantic understanding of storage values to eliminate ambiguity in ZK proofs, enabling deterministic and verifiable storage path resolution across multiple blockchain ecosystems.

### Key Features

- **Chain-independent**: Supports Ethereum, Solana, and Cosmos
- **Semantic storage proofs**: Distinguishes between different meanings of zero values
- **ZK-circuit ready**: Minimal, constrained builds for proof generation
- **Modular architecture**: Use only the components you need

## Dependency Conflicts (Important)

**Solana and Ethereum ecosystems have incompatible dependencies and cannot be used in the same binary.**

### The Problem

The Rust blockchain ecosystem has fragmented around different cryptographic libraries:

| Ecosystem | Library | k256 Version | secp256k1 Version |
|-----------|---------|--------------|-------------------|
| **Ethereum (Alloy)** | `alloy-*` | `^0.14` | `^0.29` |
| **Solana** | `solana-sdk` | `^0.13` | `^0.28` |

These create irreconcilable dependency conflicts in Cargo's dependency resolver.

### The Solution: Feature Flags

We use conditional compilation to ensure you can only enable one blockchain ecosystem at a time:

#### Ethereum Only (Default)
```toml
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }
```

#### Solana Only
```toml
[dependencies]
traverse = { version = "0.1", features = ["solana"], default-features = false }
```

#### Separate Binaries
```toml
# ethereum-processor/Cargo.toml
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }

# solana-processor/Cargo.toml  
[dependencies]
traverse = { version = "0.1", features = ["solana"], default-features = false }
```

## Quick Start

### Ethereum Storage Proofs

```rust
use traverse_ethereum::{EthereumLayoutCompiler, EthereumProofFetcher};

// Compile storage layout from ABI
let layout = EthereumLayoutCompiler::compile_from_abi(&abi_json)?;

// Generate storage proof
let fetcher = EthereumProofFetcher::new(rpc_url);
let proof = fetcher.fetch_storage_proof(
    &contract_address, 
    &storage_slot,
    ZeroSemantics::ValidZero  // Semantic meaning of zero values
).await?;
```

### Solana Account Proofs

```rust
use traverse_solana::{SolanaLayoutCompiler, SolanaProofFetcher};

// Compile layout from IDL
let layout = SolanaLayoutCompiler::compile_from_idl(&idl_json)?;

// Generate account proof
let fetcher = SolanaProofFetcher::new(rpc_url);
let proof = fetcher.fetch_account_proof(&account_address).await?;
```

### ZK Circuit Integration

```rust
use traverse_valence::{create_witness_from_request, StorageVerificationRequest};

// Create witness for ZK circuit
let witness = create_witness_from_request(&verification_request)?;

// Use in your circuit
let result = my_circuit(vec![witness]);
```

## Semantic Zero Values

Traverse eliminates ambiguity in ZK proofs by requiring explicit declaration of what zero values mean:

```rust
use traverse_core::ZeroSemantics;

// Different semantic meanings of storage slot = 0x00...00
ZeroSemantics::NeverWritten    // Slot has never been written to
ZeroSemantics::ExplicitlyZero  // Slot was intentionally set to zero  
ZeroSemantics::Cleared         // Slot was previously non-zero but cleared
ZeroSemantics::ValidZero       // Zero is a valid operational state
```

This prevents semantic confusion attacks and makes proofs more reliable.

## Feature Flags

### Core Features
- `std` - Standard library support (default)
- `no-std` - No standard library (embedded/constrained environments)
- `minimal` - Lightweight build with essential functionality only
- `constrained` - Maximum optimization for ZK circuits

### Blockchain Features (Mutually Exclusive)
- `ethereum` - Ethereum/EVM support with lightweight Alloy integration
- `solana` - Solana support with SDK integration **Conflicts with Alloy**
- `cosmos` - Cosmos/CosmWasm support

### Integration Features
- `client` - HTTP clients for live blockchain data
- `lightweight` - Optimized Ethereum support with selective Alloy imports
- `codegen` - Generate custom crates for specific layouts

## Architecture

```
traverse/
├── traverse-core/          # Chain-agnostic types and traits
├── traverse-ethereum/      # Ethereum/EVM implementation  
├── traverse-solana/        # Solana implementation
├── traverse-cosmos/        # Cosmos implementation
├── traverse-valence/       # ZK circuit integration
└── traverse-cli/           # Command-line interface
```

## Examples

### Generate Ethereum Storage Proof
```bash
cargo run --features ethereum -- ethereum generate-proof \
  --contract 0xA0b86a33E6842E8D4EACB7EB3Bf4a8B0B6A0A20D \
  --slot 0x0000000000000000000000000000000000000000000000000000000000000001 \
  --rpc https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY \
  --zero-means valid_zero
```

### Generate Solana Account Proof
```bash
cargo run --features solana --no-default-features -- solana generate-proof \
  --account 4fYNw3dojWmQ4dXtSGE9epjRGy9VpgX1BAmH6qAoMZY4 \
  --rpc https://api.mainnet-beta.solana.com
```

### Compile Storage Layout
```bash
# Ethereum ABI to layout
cargo run --features ethereum -- ethereum compile-layout \
  ./contracts/MyToken.abi.json \
  --output layout.json

# Solana IDL to layout  
cargo run --features solana --no-default-features -- solana compile-layout \
  ./programs/my_program.json \
  --output layout.json
```

## Integration Guides

- **[Valence Integration](docs/valence_integration_guide.md)** - ZK coprocessor integration
- **[Semantic Storage Proofs](docs/semantic_storage_proofs.md)** - Understanding zero semantics
- **[Feature Flags](docs/feature_flags.md)** - Complete feature flag reference

## Contributing

When contributing:

1. **Choose your blockchain**: Don't mix Ethereum and Solana dependencies
2. **Use feature flags**: Always gate blockchain-specific code with `#[cfg(feature = "...")]`
3. **Provide fallbacks**: Implement stubs when features are disabled
4. **Test thoroughly**: Test with and without features enabled
5. **Document conflicts**: Update documentation for any new dependency conflicts

## License

Apache-2.0
