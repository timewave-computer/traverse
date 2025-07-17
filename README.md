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
traverse-ethereum = { git = "https://github.com/timewave-computer/traverse" }

# Solana only  
[dependencies]
traverse-solana = { git = "https://github.com/timewave-computer/traverse" }

# Cosmos only
[dependencies]
traverse-cosmos = { git = "https://github.com/timewave-computer/traverse" }
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

#### Ethereum Storage Analysis
```bash
# Compile storage layout from ABI
traverse-ethereum compile-layout contract.abi.json --output layout.json

# Resolve storage query
traverse-ethereum resolve-query "_balances[0x742d35Cc...]" \
  --layout layout.json

# Generate storage proof
traverse-ethereum generate-proof \
  --address 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --query "_balances[0x742d35Cc...]" \
  --rpc https://eth-mainnet.g.alchemy.com/v2/YOUR-API-KEY
```

#### Solana Account Analysis
```bash
# Compile layout from IDL
traverse-solana compile-layout program.idl.json --output layout.json

# Generate account proof
traverse-solana generate-proof \
  --account TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  --rpc https://api.mainnet-beta.solana.com
```

#### Cosmos Storage Analysis
```bash
# Compile layout from schema
traverse-cosmos compile-layout contract.schema.json --output layout.json

# Generate storage proof
traverse-cosmos generate-proof \
  --contract osmo1qzsxd3t0p2ek0y2ysycfq3gm5qmfhsh32w6s9xrw4x2cmwlz3rjs5rrsnw \
  --query "balances" \
  --rpc https://rpc.osmosis.zone
```

### Library Usage

#### Ethereum
```rust
use traverse_ethereum::{EthereumLayoutCompiler, EthereumKeyResolver};
use traverse_core::{LayoutCompiler, KeyResolver, ZeroSemantics};

// Compile storage layout from ABI
let compiler = EthereumLayoutCompiler;
let layout = compiler.compile_layout(abi_file_path)?;

// Resolve storage queries
let resolver = EthereumKeyResolver;
let path = resolver.resolve(&layout, "_balances[0x742d35Cc...]")?;
```

#### Solana
```rust
use traverse_solana::{SolanaLayoutCompiler, SolanaKeyResolver};
use traverse_core::{LayoutCompiler, KeyResolver};

// Compile layout from IDL
let compiler = SolanaLayoutCompiler::new();
let layout = compiler.compile_from_idl(&idl_content)?;

// Resolve account queries
let resolver = SolanaKeyResolver::new();
let path = resolver.resolve_account_address(&query)?;
```

#### ZK Circuit Integration
```rust
use traverse_valence::{controller, circuit};

// Create witnesses for ZK circuit
let witnesses = controller::create_storage_witnesses(&args)?;

// Verify in circuit
let results = circuit::verify_storage_proofs_and_extract(witnesses);
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