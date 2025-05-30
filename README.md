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
cargo run -- --help

# Compile a contract layout (using existing test data)
cargo run -- compile-layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Resolve a simple storage field
cargo run -- resolve "_totalSupply" --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Resolve a mapping query with coprocessor JSON format
cargo run -- resolve "_balances[0x742d35cc6ab8b23c0532c65c6b555f09f9d40894]" \
  --layout crates/traverse-ethereum/tests/data/erc20_layout.json \
  --format coprocessor-json

# Resolve all possible paths from a layout
cargo run -- resolve-all --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Process multiple queries from a file (batch operation)
echo "_balances[0x742d35cc6ab8b23c0532c65c6b555f09f9d40894]" > queries.txt
echo "_totalSupply" >> queries.txt
cargo run -- batch-resolve queries.txt --layout crates/traverse-ethereum/tests/data/erc20_layout.json

# Generate storage proofs (mock implementation by default)
cargo run -- generate-proof \
  --slot 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
  --rpc https://mainnet.infura.io/v3/YOUR_API_KEY \
  --contract 0xA0b86a33E6417c7eDFeb7c14eDe3e5C8b7db1234
```

#### Live Proof Generation

By default, the `generate-proof` command uses a mock implementation. To enable live proof generation using [valence-domain-clients](https://github.com/timewave-computer/valence-domain-clients), build with the `client` feature:

```bash
# Build with client feature for live proof generation
cargo build --features client

# Generate real storage proofs from Ethereum
cargo run --features client -- generate-proof \
  --slot 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
  --rpc https://mainnet.infura.io/v3/YOUR_API_KEY \
  --contract 0xA0b86a33E6417c7eDFeb7c14eDe3e5C8b7db1234 \
  --output proof.json
```

The client feature integrates with valence-domain-clients to:
- Fetch real storage proofs via `eth_getProof` RPC calls
- Convert proof data to `CoprocessorQueryPayload` format
- Support retry logic and error handling for production use

#### Testing & Validation

```bash
# Run all tests
cargo test --all

# Test no_std compatibility
cargo test -p traverse-core --no-default-features

# Run integration tests with realistic data
cargo test -p traverse-ethereum --test integration

# Run integration tests with verbose output
cargo test -p traverse-ethereum --test integration -- --nocapture
```

#### Credit

Cover: Steinberg, Saul, The Labyrinth (New York: Harper & Brothers, 1960).