# Traverse Solana

Solana-specific implementation for ZK storage path generation, including IDL parsing, PDA/ATA derivation, and account proof fetching.

## Dependency Conflicts

This crate includes optional dependencies that conflict with the Alloy ecosystem used by `traverse-ethereum`. You cannot use both Solana and Ethereum features in the same binary.

### The Problem

- **Solana SDK** requires `k256 ^0.13` and other crypto dependencies
- **Alloy ecosystem** requires `k256 ^0.14` and different crypto dependencies
- These create irreconcilable dependency conflicts in Cargo

### The Solution: Feature Flags

Use conditional compilation to enable only the blockchain you need:

#### Option 1: Ethereum Only (Default)
```toml
[dependencies]
traverse = { version = "0.1", features = ["ethereum"] }
# OR
traverse-ethereum = { version = "0.1", features = ["lightweight-alloy"] }
```

#### Option 2: Solana Only
```toml
[dependencies]
traverse = { version = "0.1", features = ["solana"], default-features = false }
# OR
traverse-solana = { version = "0.1", features = ["solana"] }
```

#### Option 3: Separate Binaries
Create separate binaries for each blockchain:

```
my-project/
├── ethereum-processor/     # Uses traverse-ethereum
│   └── Cargo.toml         # features = ["ethereum"]
├── solana-processor/      # Uses traverse-solana  
│   └── Cargo.toml         # features = ["solana"]
└── shared-lib/            # Common logic without blockchain deps
    └── Cargo.toml         # no blockchain features
```

## Feature Flags

### Core Features

- `std` - Standard library support (default)
- `no-std` - No standard library (embedded/wasm compatible)

### Solana Features

- `solana` - Enable Solana SDK integration (conflicts with Alloy)
- `anchor` - Enable Anchor framework support (requires `solana`)
- `client` - Enable HTTP client for live RPC queries

### Without Solana Features

When Solana features are disabled, the crate provides:
- Basic data structures (`AccountLayout`, `FieldLayout`, etc.)
- Fallback implementations that return configuration errors
- Type-safe API that guides users to enable the right features

## Usage

### With Solana Features Enabled

```rust
use traverse_solana::{SolanaLayoutCompiler, SolanaKeyResolver, SolanaProofFetcher};

// Parse IDL and compile layout
let compiler = SolanaLayoutCompiler::new();
let layout = compiler.compile_from_idl(&idl_data)?;

// Resolve account addresses
let resolver = SolanaKeyResolver::new();
let pda = resolver.derive_pda(&seeds, Some(&program_id))?;

// Generate account proofs
let fetcher = SolanaProofFetcher::new(rpc_url);
let proof = fetcher.fetch_account_proof(&account_address).await?;
```

### Without Solana Features

```rust
use traverse_solana::{SolanaLayoutCompiler};

let compiler = SolanaLayoutCompiler::new();
let result = compiler.compile_from_idl(&idl_data);
// Returns: Err("Solana SDK feature not enabled. Enable the 'solana' feature flag.")
```

## CLI Integration

When using the traverse CLI:

### Ethereum Mode
```bash
cargo run --features ethereum -- ethereum generate-proof \
  --contract 0x1234... \
  --slot 0x5678... \
  --rpc https://eth-mainnet.g.alchemy.com/v2/...
```

### Solana Mode
```bash
cargo run --features solana --no-default-features -- solana generate-proof \
  --account 1234...ABC \
  --rpc https://api.mainnet-beta.solana.com
```

## Examples

See the `examples/` directory for:
- Basic Solana account proof generation
- IDL parsing and layout compilation
- PDA and ATA derivation
- Integration with traverse-valence

## Testing

```bash
# Test without any features (should compile)
cargo test --no-default-features

# Test with Solana features (may conflict in workspace)
cargo test --features solana --no-default-features

# Test in isolation (avoid workspace conflicts)
cd crates/traverse-solana
cargo test --features solana
``` 