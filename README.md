# Traverse

Chain-independent ZK storage proof system for blockchain state verification.

## Overview

Traverse generates cryptographic proofs of blockchain storage state for use in zero-knowledge circuits and cross-chain applications. It provides semantic understanding of storage values to eliminate ambiguity in ZK proofs.

### Key Features

- **Multi-chain support**: Ethereum, Solana, and Cosmos
- **Semantic storage proofs**: Distinguishes between different meanings of zero values
- **ZK-circuit ready**: Optimized builds for proof generation
- **Isolated builds**: Each blockchain ecosystem builds independently

## Installation

### Using Nix (Recommended)

```bash
# Core packages
nix build .#traverse-core
nix build .#traverse-ethereum 
nix build .#traverse-solana
nix build .#traverse-cosmos

# CLI tools
nix build .#traverse-ethereum-cli
nix build .#traverse-solana-cli  
nix build .#traverse-cosmos-cli

# Run all tests
nix flake check
```

### Using Cargo

Choose one blockchain ecosystem due to dependency conflicts:

```toml
# Ethereum only
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }

# Solana only  
[dependencies]
traverse = { version = "0.1", features = ["solana"], default-features = false }

# Cosmos only
[dependencies]
traverse = { version = "0.1", features = ["cosmos"], default-features = false }
```

See [Feature Flags documentation](docs/feature_flags.md) for details on dependency conflicts.

## Architecture

```
traverse/
├── traverse-core/          # Chain-agnostic types and traits
├── traverse-ethereum/      # Ethereum/EVM implementation  
├── traverse-solana/        # Solana implementation
├── traverse-cosmos/        # Cosmos implementation
├── traverse-valence/       # ZK circuit integration
├── traverse-cli-*/         # Ecosystem-specific CLIs
└── workspace-configs/      # Per-ecosystem Cargo workspaces
```

### Build System

Traverse uses isolated Nix builds with separate Cargo workspaces per ecosystem to handle incompatible dependencies. See [Feature Flags documentation](docs/feature_flags.md) for technical details.

## Usage

### CLI Examples

#### Ethereum Storage Proof
```bash
traverse-ethereum generate-proof \
  --contract 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --slot 0x0000000000000000000000000000000000000000000000000000000000000001 \
  --rpc https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY \
  --zero-means valid-zero
```

#### Solana Account Proof
```bash
traverse-solana generate-proof \
  --account TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  --rpc https://api.mainnet-beta.solana.com
```

#### Cosmos Storage Proof
```bash
traverse-cosmos generate-proof \
  --contract osmo1qzsxd3t0p2ek0y2ysycfq3gm5qmfhsh32w6s9xrw4x2cmwlz3rjs5rrsnw \
  --key "balances" \
  --rpc https://rpc.osmosis.zone
```

### Library Usage

#### Ethereum
```rust
use traverse_ethereum::{EthereumLayoutCompiler, EthereumProofFetcher};

// Compile storage layout from ABI
let layout = EthereumLayoutCompiler::compile_from_abi(&abi_json)?;

// Generate storage proof
let fetcher = EthereumProofFetcher::new(rpc_url);
let proof = fetcher.fetch_storage_proof(
    &contract_address, 
    &storage_slot,
    ZeroSemantics::ValidZero
).await?;
```

#### Solana
```rust
use traverse_solana::{SolanaLayoutCompiler, SolanaProofFetcher};

// Compile layout from IDL
let layout = SolanaLayoutCompiler::compile_from_idl(&idl_json)?;

// Generate account proof
let fetcher = SolanaProofFetcher::new(rpc_url);
let proof = fetcher.fetch_account_proof(&account_address).await?;
```

#### ZK Circuit Integration
```rust
use traverse_valence::{create_witness_from_request, StorageVerificationRequest};

// Create witness for ZK circuit
let witness = create_witness_from_request(&verification_request)?;
```

## Semantic Zero Values

Traverse eliminates ambiguity in storage proofs by requiring explicit declaration of zero meanings:

```rust
use traverse_core::ZeroSemantics;

ZeroSemantics::NeverWritten    // Slot has never been written to
ZeroSemantics::ExplicitlyZero  // Slot was intentionally set to zero  
ZeroSemantics::Cleared         // Slot was previously non-zero but cleared
ZeroSemantics::ValidZero       // Zero is a valid operational state
```

## Feature Flags

### Core Features
- `std` - Standard library support (default)
- `no-std` - No standard library
- `minimal` - Essential functionality only
- `constrained` - Maximum optimization for ZK circuits

### Blockchain Features (Mutually Exclusive)
- `ethereum` - Ethereum/EVM support
- `solana` - Solana support
- `cosmos` - Cosmos/CosmWasm support

### Additional Features
- `client` - HTTP clients for blockchain data
- `lightweight-alloy` - Minimal Ethereum dependencies
- `codegen` - Layout code generation

## Documentation

- **[Valence Integration Guide](docs/valence_integration_guide.md)** - ZK coprocessor integration
- **[Semantic Storage Proofs](docs/semantic_storage_proofs.md)** - Understanding zero semantics
- **[Feature Flags Reference](docs/feature_flags.md)** - Complete feature documentation

## License

Apache-2.0