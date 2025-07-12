# Traverse

Generate ZK-compatible storage proofs for blockchain contract verification. Traverse provides a pipeline for generating deterministic, circuit-friendly storage paths from blockchain contract layouts. It abstracts away chain-specific storage encoding to support cross-chain ZK applications.

![](./labyrinth.png)

 Traverse implements a three-layer architecture:

1. **Layout Compilation**: Convert chain-specific ABIs to canonical layout format
2. **Path Resolution**: Translate queries into deterministic storage keys  
3. **Proof Generation**: Fetch runtime proofs for ZK coprocessors

#### Key Features

**Circuit-Ready**: Core library is `no_std` compatible for RISC-V circuits  
**Fast & Deterministic**: Layout commitments ensure reproducible circuit behavior  
**Chain-Independent**: Extensible architecture supports multiple blockchains  
**Developer-Friendly**: Simple CLI and library API for easy integration  

#### Development Setup

```bash
# Enter nix shell
nix develop

# Build all crates with default features
cargo build

# Build with specific features
cargo build --features ethereum,client
cargo build --features minimal --no-default-features
cargo build --features wasm --no-default-features

# Run all tests including integration tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Build and run the CLI
cargo run -- --help
```

#### Feature Flags

Traverse supports comprehensive feature flags for different use cases:

- **`std`** (default): Full standard library support
- **`no-std`**: Minimal build for embedded/circuit environments  
- **`minimal`**: Lightweight build with essential functionality
- **`wasm`**: WebAssembly-compatible build
- **`ethereum`** (default): Ethereum blockchain support
- **`cosmos`**: Cosmos/CosmWasm blockchain support
- **`client`**: Live blockchain integration
- **`examples`**: Example code and demonstrations

See [Feature Flags Guide](docs/feature_flags.md) for detailed usage instructions.

#### CLI Examples

```bash
# Show available commands and help
cargo run -p traverse-cli -- --help

# Compile a contract layout (using existing test data)
cargo run -p traverse-cli -- compile-layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Resolve a simple storage field
cargo run -p traverse-cli -- resolve "_totalSupply" --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Resolve a mapping query with coprocessor JSON format
cargo run -p traverse-cli -- resolve "_balances[0x742d35cc6ab8b23c0532c65c6b555f09f9d40894]" \
  --layout crates/traverse-ethereum/tests/data/erc20_layout.json \
  --format coprocessor-json

# Resolve all possible paths from a layout
cargo run -p traverse-cli -- resolve-all --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Process multiple queries from a file (batch operation)
echo "_balances[0x742d35cc6ab8b23c0532c65c6b555f09f9d40894]" > queries.txt
echo "_totalSupply" >> queries.txt
cargo run -p traverse-cli -- batch-resolve queries.txt --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Generate storage proofs (uses alloy-based fallback by default)
cargo run -p traverse-cli -- generate-proof \
  --slot 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
  --rpc https://mainnet.infura.io/v3/YOUR_API_KEY \
  --contract 0xA0b86a33E6417c7eDFeb7c14eDe3e5C8b7db1234

# Generate real storage proof with valence-domain-clients
cargo run -p traverse-cli --features client -- generate-proof \
  --slot 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
  --rpc https://mainnet.infura.io/v3/YOUR_API_KEY \
  --contract 0xA0b86a33E6417c7eDFeb7c14eDe3e5C8b7db1234 \
  --output proof.json

# Real contract ABI fetching and storage layout generation
# Example: Valence One Way Vault storage queries
export ETHERSCAN_API_KEY=your_etherscan_api_key
export ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your_infura_key
cargo run --example valence_vault_storage

# Example with client feature for live RPC calls via valence-domain-clients
cargo run --example valence_vault_storage --features client
```

#### Examples

Traverse includes several examples demonstrating real-world usage:

```bash
# Circuit Usage Example
# Demonstrates no_std circuit patterns and pre-computed storage paths
cargo run --features examples --example circuit_usage

# USDT Live Proof Example  
# Shows complete integration with real Ethereum storage proofs
cargo run --features examples --example usdt_live_proof

# Valence Integration Example
# Demonstrates traverse-valence coprocessor integration
cargo run --features examples --example valence_integration
```

The examples showcase:
- **Circuit Usage**: No-std compatible circuit patterns for ZK applications
- **USDT Live Proof**: Real-time storage proof processing from actual eth_getProof responses
- **Valence Integration**: Complete three-tier integration with valence coprocessor framework

The **Valence Vault Storage** example showcases:
- Real-time ABI fetching from Etherscan API
- Automatic storage layout generation from contract ABIs  
- Live storage queries via Ethereum RPC calls using valence-domain-clients
- Coprocessor-compatible JSON output generation
- Environment variable configuration for API keys

#### Live Proof Generation

Traverse uses a hybrid approach for proof generation:

- **Default mode**: Uses Alloy-based proof fetcher for basic functionality
- **Client mode** (--features client): Uses valence-domain-clients for enhanced integration

The client feature provides deeper integration with the valence coprocessor ecosystem and supports advanced signing workflows, while the default mode offers a lightweight option for basic proof generation.

#### Architecture

Traverse converts from chain-specific contract ABIs to deterministic storage keys:

1. **ABI Fetching**: Retrieve contract ABIs from Etherscan or other sources
2. **Layout Compilation**: Generate canonical storage layouts from ABIs
3. **Key Resolution**: Convert queries like `_balances[0x...]` to storage slots
4. **Proof Generation**: Fetch storage proofs via RPC using Alloy or valence-domain-clients

All core operations are deterministic and `no_std` compatible for use in ZK circuits.

## Overview

Traverse provides tools to query blockchain storage, generate cryptographic proofs, and create ZK circuits for cross-chain verification. The library now includes **semantic storage proofs** that disambiguate the meaning of zero values in contract storage.

### Semantic Storage Proofs

Traverse addresses a fundamental ambiguity in blockchain storage: when a storage slot contains zero, it could mean:
- **Never Written**: The slot has never been written to (default state)
- **Explicitly Zero**: The slot was intentionally set to zero
- **Cleared**: The slot previously held a non-zero value but was cleared
- **Valid Zero**: Zero is a valid operational state for this field

This ambiguity can lead to incorrect business logic in ZK applications. Traverse solves this with semantic storage proofs that include explicit semantic metadata.

## Key Features

- **Storage Layout Generation**: Automatically extract storage layouts from contract ABIs
- **Live Blockchain Queries**: Query storage values from Ethereum and CosmWasm chains
- **Semantic Specifications**: Declare the intended meaning of zero values in storage
- **Conflict Detection**: Validate semantic declarations against blockchain events
- **ZK Circuit Integration**: Generate proofs compatible with SP1 and other ZK frameworks
- **Cross-Chain Support**: Unified interface for Ethereum and Cosmos ecosystems

## Examples

### Semantic Storage Proof Examples

The `examples/` directory contains comprehensive demonstrations of semantic storage proofs:

#### 1. Semantic Conflict Resolution (`examples/semantic_conflict_resolution.rs`)
Demonstrates how to detect and resolve conflicts between declared and actual semantic meanings:

```bash
cargo run --example semantic_conflict_resolution
```

**Key Features:**
- Mock contracts with different semantic scenarios
- Automatic conflict detection using event analysis
- Resolution strategies preferring validated over declared semantics
- Business logic integration based on resolved semantics

#### 2. Semantic Business Logic Integration (`examples/semantic_business_logic.rs`)
Shows how semantic meanings affect DeFi protocol authorization and validation:

```bash
cargo run --example semantic_business_logic
```

**Key Features:**
- Semantic-aware authorization systems
- Binary semantic validation (valid/invalid)
- Protocol health monitoring with semantic conflict detection
- Business logic based on semantic correctness

#### 3. Semantic CLI Integration (`examples/semantic_cli_integration.rs`)
Complete guide to using the traverse CLI with semantic storage proofs:

```bash
cargo run --example semantic_cli_integration
```

**Key Features:**
- CLI command examples for all semantic types
- Batch processing with mixed semantic specifications
- Configuration file templates
- Integration with multiple indexer services

#### 4. Live USDT Example with Semantics (`examples/usdt_live_proof.rs`)
Real-world USDT contract storage proof with semantic specifications:

```bash
cargo run --example usdt_live_proof --features client
```

**Requirements:**
- `ETHEREUM_RPC_URL` environment variable
- `ETHERSCAN_API_KEY` for ABI fetching

#### 5. Valence Vault Semantic Integration (`examples/valence_vault_storage.rs`)
Complete CosmWasm coprocessor workflow with semantic storage proofs:

```bash
cargo run --example valence_vault_storage --features client
```

**Features:**
- End-to-end semantic storage proof workflow
- Valence protocol integration
- Semantic-aware business logic
- Cross-chain state verification

### Circuit Usage Example

```rust
use traverse_valence::circuit;

// Verify semantic storage proofs in ZK circuit
let verification_results = circuit::verify_semantic_storage_proofs_and_extract(&witnesses)?;

// All proofs must pass verification (returns 0x01 for valid)
let all_valid = verification_results.iter().all(|&result| result == 0x01);
```

## CLI Usage

### Basic Semantic Storage Proof

```bash
traverse generate-proof \
  --contract 0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123 \
  --slot 0x0000000000000000000000000000000000000000000000000000000000000000 \
  --zero-means never_written \
  --rpc-url $ETHEREUM_RPC_URL
```

### Semantic Validation with Conflict Detection

```bash
traverse generate-proof \
  --contract 0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123 \
  --slot 0x0000000000000000000000000000000000000000000000000000000000000000 \
  --zero-means never_written \
  --validate-semantics \
  --resolve-conflicts \
  --indexer etherscan \
  --rpc-url $ETHEREUM_RPC_URL
```

### Batch Processing with Mixed Semantics

```bash
traverse batch-generate-proofs \
  --config semantic_batch_config.json \
  --validate-semantics \
  --parallel 4 \
  --rpc-url $ETHEREUM_RPC_URL
```

## Configuration

### Environment Variables

```bash
# Required
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your_project_id
ETHERSCAN_API_KEY=your_etherscan_api_key

# Optional for semantic validation
ALCHEMY_API_KEY=your_alchemy_api_key
MORALIS_API_KEY=your_moralis_api_key

# Optional for CosmWasm integration
NEUTRON_RPC_URL=https://rpc.neutron.quokkastake.io
NEUTRON_MNEMONIC=your_neutron_mnemonic
```

### Semantic Specification File

```json
{
  "contract_address": "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
  "specifications": [
    {
      "storage_slot": "0x0",
      "field_name": "_balances",
      "zero_semantics": "never_written",
      "description": "User token balances - most addresses never hold tokens"
    },
    {
      "storage_slot": "0x2",
      "field_name": "_totalSupply",
      "zero_semantics": "explicitly_zero",
      "description": "Total supply initialized to zero during deployment"
    }
  ]
}
```

## Zero Semantic Types

| Type | Meaning | Use Case | Business Logic |
|------|---------|----------|----------------|
| `never_written` | Storage slot has never been written to | User balances in tokens they never held | Treat as uninitialized state |
| `explicitly_zero` | Intentionally set to zero | Contract initialization values | Valid operational state |
| `cleared` | Previously non-zero but cleared | Withdrawn balances, processed requests | Previous activity confirmed |
| `valid_zero` | Zero is a valid operational state | Counters, flags, operational values | Valid operational state |

## Integration Guide

### 1. Generate Semantic Layout

```bash
traverse generate-layout \
  --contract 0xYourContract \
  --semantic-specs semantic_specs.json \
  --etherscan-key $ETHERSCAN_API_KEY
```

### 2. Create Semantic Storage Proofs

```bash
traverse generate-proof \
  --contract 0xYourContract \
  --query "balanceOf[0xUserAddress]" \
  --zero-means never_written \
  --validate-semantics \
  --rpc-url $ETHEREUM_RPC_URL
```

### 3. Integrate with ZK Circuits

```rust
use traverse_valence::{controller, circuit};

// Create semantic witnesses
let witnesses = controller::create_semantic_storage_witnesses(&semantic_batch)?;

// Verify in circuit
let results = circuit::verify_semantic_storage_proofs_and_extract(&witnesses)?;
```

### 4. Business Logic Integration

```rust
match (value, resolved_semantics.zero_meaning) {
    (0, ZeroSemantics::NeverWritten) => "User never participated - no initialization required",
    (0, ZeroSemantics::ExplicitlyZero) => "System initialized and ready for operations", 
    (0, ZeroSemantics::Cleared) => "Previous activity cleared - history confirmed",
    (0, ZeroSemantics::ValidZero) => "Valid operational state - proceed normally",
    (n, _) => format!("Active with value {}", n),
}
```

## Architecture

### Core Components

- **traverse-core**: Common types and semantic definitions
- **traverse-ethereum**: Ethereum storage proof generation with semantic validation
- **traverse-cosmos**: CosmWasm storage proof generation
- **traverse-valence**: ZK circuit integration for semantic proofs
- **traverse-cli**: Command-line interface with semantic support

### Semantic Validation Pipeline

1. **Declaration**: Developer specifies semantic meaning in layout
2. **Generation**: CLI generates storage proof with semantic metadata
3. **Validation**: Indexer services validate semantics against blockchain events
4. **Resolution**: Conflicts resolved automatically (validated > declared)
5. **Integration**: Business logic adapts based on resolved semantics

## Testing

Run all tests including semantic validation:

```bash
cargo test
```

Run semantic-specific tests:

```bash
cargo test -- semantic
```

Run integration tests with semantic validation:

```bash
cargo test -p traverse-ethereum --test integration -- semantic
```

## Documentation

- [Architecture Guide](docs/architecture.md)
- [Integration Guide](docs/integration_guide.md) 
- [Semantic Usage Guide](docs/semantic_usage_guide.md)
- [Event-Based Validation](docs/event_based_validation.md)
- [Ethereum Storage False Positives](docs/ethereum_storage_false_positives.md)
