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

# Build all crates
cargo build

# Run all tests including integration tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Build and run the CLI
cargo run -- --help
```

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

# Generate storage proofs (mock implementation by default)
cargo run -p traverse-cli -- generate-proof \
  --slot 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
  --rpc https://mainnet.infura.io/v3/YOUR_API_KEY \
  --contract 0xA0b86a33E6417c7eDFeb7c14eDe3e5C8b7db1234

# Generate real storage proof with traverse-cli
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

# Example with client feature for live RPC calls
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
- Live storage queries via Ethereum RPC calls
- Coprocessor-compatible JSON output generation
- Environment variable configuration for API keys

#### Live Proof Generation

By default, the `