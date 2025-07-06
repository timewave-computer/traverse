//! End-to-end tests for Ethereum CLI commands
//!
//! Tests all Ethereum-specific commands documented in the work plan:
//! - traverse ethereum analyze-contract
//! - traverse ethereum compile-layout  
//! - traverse ethereum generate-queries
//! - traverse ethereum resolve-query
//! - traverse ethereum verify-layout
//! - traverse ethereum auto-generate

use crate::fixtures::TestFixtures;
use crate::utils::*;
use anyhow::Result;
use std::fs;

/// Run all Ethereum CLI command tests
pub async fn run_tests(fixtures: &TestFixtures) -> Result<()> {
    let cli = CliRunner::new(fixtures.cli_path());

    println!("    🔷 Testing Ethereum analyze-contract command...");
    test_analyze_contract(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum compile-layout command...");
    test_compile_layout(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum generate-queries command...");
    test_generate_queries(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum resolve-query command...");
    test_resolve_query(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum verify-layout command...");
    test_verify_layout(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum auto-generate command...");
    test_auto_generate(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum output formats...");
    test_output_formats(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum performance...");
    test_performance(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum semantic proof generation...");
    test_semantic_proof_generation(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum semantic validation end-to-end...");
    test_semantic_validation_e2e(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum semantic error handling...");
    test_semantic_error_handling(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum batch operations with semantics...");
    test_batch_operations_with_semantics(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum circuit semantic validation...");
    test_circuit_semantic_validation(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum missing semantics failure modes...");
    test_missing_semantics_failure_modes(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum semantic conflict detection and resolution...");
    test_semantic_conflict_detection_and_resolution(&cli, fixtures).await?;

    println!("    🔷 Testing Ethereum event validation with real blockchain data...");
    test_event_validation_with_real_blockchain_data(&cli, fixtures).await?;

    Ok(())
}

/// Test `traverse ethereum analyze-contract` command
async fn test_analyze_contract(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output_path = fixtures.path("outputs/erc20_analysis.json");

    // Test basic analysis
    let output = cli.run_success(&[
        "ethereum",
        "analyze-contract",
        erc20_abi.to_str().unwrap(),
        "--output",
        output_path.to_str().unwrap(),
    ])?;

    // Verify output contains expected information
    assertions::assert_output_contains(&output, "Analyzing Ethereum contract")?;

    // Verify output file was created and has expected structure
    let _analysis = FileValidator::validate_json_structure(
        &output_path,
        &["contract_type", "functions", "storage_patterns"],
    )?;

    // Test with validation flag
    let output = cli.run_success(&[
        "ethereum",
        "analyze-contract",
        erc20_abi.to_str().unwrap(),
        "--validate-storage",
    ])?;

    assertions::assert_output_contains(&output, "Storage validation")?;

    // Test with complex DeFi contract
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();
    let defi_output_path = fixtures.path("outputs/defi_analysis.json");

    let output = cli.run_success(&[
        "ethereum",
        "analyze-contract",
        defi_abi.to_str().unwrap(),
        "--output",
        defi_output_path.to_str().unwrap(),
        "--validate-storage",
    ])?;

    assertions::assert_output_contains(&output, "complex mappings")?;
    FileValidator::validate_json(&defi_output_path)?;

    println!("      analyze-contract tests passed");
    Ok(())
}

/// Test `traverse ethereum compile-layout` command  
async fn test_compile_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout.json");

    // Test basic layout compilation
    let output = cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
        "--format",
        "traverse",
    ])?;

    assertions::assert_output_contains(&output, "Compiling storage layout")?;

    // Verify layout file structure
    let _layout = FileValidator::validate_json_structure(
        &layout_path,
        &["contract_name", "storage_layout", "commitment"],
    )?;

    // Test with validation flag
    let output = cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--validate",
    ])?;

    assertions::assert_output_contains(&output, "Layout validation")?;

    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary"] {
        let format_output_path = fixtures.path(&format!("outputs/erc20_layout.{}", format));

        cli.run_success(&[
            "ethereum",
            "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output",
            format_output_path.to_str().unwrap(),
            "--format",
            format,
        ])?;

        FileValidator::exists_and_non_empty(&format_output_path)?;

        // Validate format-specific content
        match *format {
            "traverse" | "coprocessor-json" => {
                FileValidator::validate_json(&format_output_path)?;
            }
            "toml" => {
                FileValidator::validate_toml(&format_output_path)?;
            }
            "binary" => {
                let data = fs::read(&format_output_path)?;
                OutputValidator::validate_binary_format(&data)?;
            }
            _ => {}
        }
    }

    // Test with complex DeFi contract
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();
    let defi_layout_path = fixtures.path("outputs/defi_layout.json");

    cli.run_success(&[
        "ethereum",
        "compile-layout",
        defi_abi.to_str().unwrap(),
        "--output",
        defi_layout_path.to_str().unwrap(),
        "--validate",
    ])?;

    FileValidator::validate_json(&defi_layout_path)?;

    println!("      compile-layout tests passed");
    Ok(())
}

/// Test `traverse ethereum generate-queries` command
async fn test_generate_queries(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // First compile a layout
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");

    cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
    ])?;

    // Test query generation
    let queries_path = fixtures.path("outputs/erc20_queries.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-queries",
        layout_path.to_str().unwrap(),
        "--fields",
        "totalSupply,name,symbol,balanceOf",
        "--output",
        queries_path.to_str().unwrap(),
    ])?;

    assertions::assert_output_contains(&output, "Generated")?;

    // Verify queries file structure
    let _queries = FileValidator::validate_json_structure(&queries_path, &["queries", "metadata"])?;

    // Test with examples flag
    let queries_with_examples_path = fixtures.path("outputs/erc20_queries_with_examples.json");

    cli.run_success(&[
        "ethereum",
        "generate-queries",
        layout_path.to_str().unwrap(),
        "--fields",
        "balanceOf",
        "--output",
        queries_with_examples_path.to_str().unwrap(),
        "--include-examples",
    ])?;

    let queries_with_examples = FileValidator::validate_json(&queries_with_examples_path)?;

    // Should contain example addresses for mappings
    assertions::assert_output_contains(&serde_json::to_string(&queries_with_examples)?, "0x")?;

    println!("      generate-queries tests passed");
    Ok(())
}

/// Test `traverse ethereum resolve-query` command
async fn test_resolve_query(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Use the layout from previous test
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");

    // Test basic query resolution
    let resolved_path = fixtures.path("outputs/totalSupply_resolved.json");

    let output = cli.run_success(&[
        "ethereum",
        "resolve-query",
        "totalSupply",
        "--layout",
        layout_path.to_str().unwrap(),
        "--format",
        "coprocessor-json",
        "--output",
        resolved_path.to_str().unwrap(),
    ])?;

    assertions::assert_output_contains(&output, "Resolved")?;

    // Verify resolved query structure
    let _resolved =
        FileValidator::validate_json_structure(&resolved_path, &["storage_key", "slot_index"])?;

    // Test mapping query resolution
    let mapping_resolved_path = fixtures.path("outputs/balance_resolved.json");

    cli.run_success(&[
        "ethereum",
        "resolve-query",
        "balanceOf[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
        "--layout",
        layout_path.to_str().unwrap(),
        "--format",
        "coprocessor-json",
        "--output",
        mapping_resolved_path.to_str().unwrap(),
    ])?;

    FileValidator::validate_json(&mapping_resolved_path)?;

    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let format_resolved_path =
            fixtures.path(&format!("outputs/totalSupply_resolved.{}", format));

        cli.run_success(&[
            "ethereum",
            "resolve-query",
            "totalSupply",
            "--layout",
            layout_path.to_str().unwrap(),
            "--format",
            format,
            "--output",
            format_resolved_path.to_str().unwrap(),
        ])?;

        match *format {
            "base64" => {
                let content = fs::read_to_string(&format_resolved_path)?;
                OutputValidator::validate_base64_format(&content)?;
            }
            _ => {
                FileValidator::exists_and_non_empty(&format_resolved_path)?;
            }
        }
    }

    println!("      resolve-query tests passed");
    Ok(())
}

/// Test `traverse ethereum verify-layout` command
async fn test_verify_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");

    // Test basic layout verification (dry run)
    let output = cli.run_success(&["ethereum", "verify-layout", layout_path.to_str().unwrap()])?;

    assertions::assert_output_contains(&output, "Layout verification")?;

    // Test comprehensive verification
    let output = cli.run_success(&[
        "ethereum",
        "verify-layout",
        layout_path.to_str().unwrap(),
        "--comprehensive",
    ])?;

    assertions::assert_output_contains(&output, "Comprehensive")?;

    println!("      verify-layout tests passed");
    Ok(())
}

/// Test `traverse ethereum auto-generate` command  
async fn test_auto_generate(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output_dir = fixtures.path("outputs/erc20_auto");

    // Test dry-run mode
    let output = cli.run_success(&[
        "ethereum",
        "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--queries",
        "totalSupply,name,symbol",
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "End-to-end automation")?;
    assertions::assert_output_contains(&output, "Dry run mode")?;

    // Verify output directory structure in dry-run
    FileValidator::validate_directory_structure(
        &output_dir,
        &["layout.json", "queries.json", "resolved_queries.json"],
    )?;

    // Test with caching enabled
    let output = cli.run_success(&[
        "ethereum",
        "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--queries",
        "totalSupply",
        "--output-dir",
        output_dir.to_str().unwrap(),
        "--cache",
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Caching enabled")?;

    println!("      auto-generate tests passed");
    Ok(())
}

/// Test different output formats work correctly
async fn test_output_formats(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();

    // Test each format with compile-layout
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let output_path = fixtures.path(&format!("outputs/format_test.{}", format));

        cli.run_success(&[
            "ethereum",
            "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
            "--format",
            format,
        ])?;

        // Validate format-specific content
        match *format {
            "coprocessor-json" => {
                let json = FileValidator::validate_json(&output_path)?;
                OutputValidator::validate_coprocessor_format(&json)?;
            }
            "traverse" => {
                let content = fs::read_to_string(&output_path)?;
                OutputValidator::validate_traverse_format(&content)?;
            }
            "base64" => {
                let content = fs::read_to_string(&output_path)?;
                OutputValidator::validate_base64_format(&content)?;
            }
            _ => {
                FileValidator::exists_and_non_empty(&output_path)?;
            }
        }
    }

    println!("      output-formats tests passed");
    Ok(())
}

/// Test performance of Ethereum commands
async fn test_performance(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output_path = fixtures.path("outputs/perf_test.json");

    // Test compile-layout performance (should complete in reasonable time)
    PerformanceTester::test_performance_threshold(
        cli,
        &[
            "ethereum",
            "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ],
        std::time::Duration::from_secs(30), // 30 second threshold
    )?;

    // Test analyze-contract performance
    PerformanceTester::test_performance_threshold(
        cli,
        &["ethereum", "analyze-contract", erc20_abi.to_str().unwrap()],
        std::time::Duration::from_secs(15), // 15 second threshold
    )?;

    println!("      performance tests passed");
    Ok(())
}

/// Test end-to-end semantic proof generation with all four zero semantic types
async fn test_semantic_proof_generation(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout_semantic.json");

    // First compile layout (this should now require semantic metadata)
    cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
    ])?;

    // Test semantic proof generation with each of the four zero semantic types
    let semantic_types = vec![
        ("never_written", "Slot was never written to"),
        ("explicitly_zero", "Slot was intentionally set to zero"),
        ("cleared", "Slot was previously non-zero but cleared"),
        ("valid_zero", "Zero is a valid operational state"),
    ];

    for (semantic_type, description) in semantic_types {
        let proof_path = fixtures.path(&format!("outputs/semantic_proof_{}.json", semantic_type));

        // Test generate-proof command with semantic specification
        let output = cli.run_success(&[
            "ethereum",
            "generate-proof",
            "--contract",
            "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
            "--slot",
            "0x2", // totalSupply slot
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--zero-means",
            semantic_type,
            "--output",
            proof_path.to_str().unwrap(),
            "--dry-run",
        ])?;

        println!(
            "        Testing semantic type: {} - {}",
            semantic_type, description
        );

        // Verify command output indicates semantic processing
        assertions::assert_output_contains(&output, "Semantic storage proof")?;
        assertions::assert_output_contains(&output, semantic_type)?;

        // Verify proof file structure includes semantic metadata
        let proof_json = FileValidator::validate_json(&proof_path)?;

        // Check that semantic metadata is included in proof
        OutputValidator::validate_semantic_proof_structure(
            &proof_json,
            &[
                "storage_key",
                "semantic_metadata",
                "zero_semantics",
                "proof_data",
            ],
        )?;

        // Verify semantic type is correctly recorded
        if let Some(semantic_meta) = proof_json.get("semantic_metadata") {
            assertions::assert_json_contains(semantic_meta, "zero_meaning")?;
        }
    }

    // Test batch semantic proof generation
    let batch_output_dir = fixtures.path("outputs/batch_semantic_proofs");

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        layout_path.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply,name,symbol",
        "--zero-means",
        "never_written",
        "--output-dir",
        batch_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Batch semantic proof generation")?;

    // Verify batch outputs contain semantic metadata
    FileValidator::validate_directory_structure(
        &batch_output_dir,
        &["totalSupply.json", "name.json", "symbol.json"],
    )?;

    println!("      semantic-proof-generation tests passed");
    Ok(())
}

/// Test end-to-end semantic validation with indexer services
async fn test_semantic_validation_e2e(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout_validation.json");

    // Compile layout for validation tests
    cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
    ])?;

    // Test semantic validation with indexer service
    let proof_path = fixtures.path("outputs/validated_semantic_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2", // totalSupply slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics", // Enable semantic validation
        "--output",
        proof_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Verify semantic validation was performed
    assertions::assert_output_contains(&output, "Semantic validation")?;

    // Test with potential semantic conflicts (declared never_written but indexer shows writes)
    let conflict_proof_path = fixtures.path("outputs/conflict_semantic_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract (likely has writes)
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written", // Declare never written
        "--validate-semantics",
        "--output",
        conflict_proof_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should detect semantic conflict and provide warnings
    assertions::assert_output_contains(&output, "Semantic")?;

    // Verify conflict is recorded in proof output
    let conflict_proof = FileValidator::validate_json(&conflict_proof_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &conflict_proof,
        &["semantic_metadata", "validation_result"],
    )?;

    println!("      semantic-validation-e2e tests passed");
    Ok(())
}

/// Test error handling for missing semantics and invalid arguments
async fn test_semantic_error_handling(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();

    // Test that generate-proof fails without semantic specification
    let output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        // Missing --zero-means argument
        "--dry-run",
    ])?;

    // Should error with message about required semantic argument
    assertions::assert_output_contains(&output, "required")?;
    assertions::assert_output_contains(&output, "zero-means")?;

    // Test invalid semantic type
    let output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "invalid_semantic", // Invalid semantic type
        "--dry-run",
    ])?;

    // Should error with message about invalid semantic type
    assertions::assert_output_contains(&output, "invalid")?;

    // Test that batch operations fail without semantic specification
    let batch_output_dir = fixtures.path("outputs/batch_error_test");

    let output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply",
        // Missing --zero-means argument
        "--output-dir",
        batch_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "required")?;
    assertions::assert_output_contains(&output, "zero-means")?;

    // Test semantic validation with invalid indexer service
    let output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--indexer-service",
        "invalid_service", // Invalid indexer service
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Invalid indexer service")?;

    // Test layout compilation with missing semantic metadata
    let layout_path = fixtures.path("outputs/layout_error_test.json");

    // This should work since layout compilation now auto-adds semantic metadata
    let output = cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
    ])?;

    assertions::assert_output_contains(&output, "Compiling storage layout")?;

    println!("      semantic-error-handling tests passed");
    Ok(())
}

/// Test batch operations with semantics
async fn test_batch_operations_with_semantics(
    cli: &CliRunner,
    fixtures: &TestFixtures,
) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();

    // Test 1: Basic batch operations with single semantic type
    println!("        Testing basic batch operations with semantics...");
    let batch_output_dir = fixtures.path("outputs/batch_basic_semantics");

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply,name,symbol",
        "--zero-means",
        "never_written",
        "--output-dir",
        batch_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Batch semantic proof generation")?;

    // Verify batch outputs contain semantic metadata
    FileValidator::validate_directory_structure(
        &batch_output_dir,
        &["totalSupply.json", "name.json", "symbol.json"],
    )?;

    // Check each batch output file for semantic metadata
    for filename in &["totalSupply.json", "name.json", "symbol.json"] {
        let proof_path = batch_output_dir.join(filename);
        let proof_json = FileValidator::validate_json(&proof_path)?;

        // Verify each proof contains semantic metadata
        OutputValidator::validate_semantic_proof_structure(
            &proof_json,
            &["storage_key", "semantic_metadata", "zero_semantics"],
        )?;
    }

    // Test 2: Batch operations with different semantic types per query
    println!("        Testing mixed semantic types in batch operations...");
    let mixed_batch_dir = fixtures.path("outputs/batch_mixed_semantics");

    // Create semantic configuration file for mixed types
    let semantic_config = serde_json::json!({
        "queries": [
            {
                "query": "totalSupply",
                "zero_means": "explicitly_zero",
                "description": "Total supply should be explicitly set"
            },
            {
                "query": "name",
                "zero_means": "never_written",
                "description": "Name might not be set in some tokens"
            },
            {
                "query": "symbol",
                "zero_means": "never_written",
                "description": "Symbol might not be set in some tokens"
            }
        ]
    });

    let semantic_config_path = fixtures.path("outputs/semantic_config.json");
    fs::write(
        &semantic_config_path,
        serde_json::to_string_pretty(&semantic_config)?,
    )?;

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--semantic-config",
        semantic_config_path.to_str().unwrap(),
        "--output-dir",
        mixed_batch_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Mixed semantic types")?;

    // Test 3: Batch operations with semantic validation
    println!("        Testing batch operations with semantic validation...");
    let validated_batch_dir = fixtures.path("outputs/batch_validated_semantics");

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply,name",
        "--zero-means",
        "never_written",
        "--validate-semantics", // Enable semantic validation
        "--output-dir",
        validated_batch_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Batch semantic validation")?;

    // Test 4: Complex contract batch operations
    println!("        Testing complex contract batch operations...");
    let complex_batch_dir = fixtures.path("outputs/batch_complex_semantics");

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        defi_abi.to_str().unwrap(),
        "--contract",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalDeposits,userInfo",
        "--zero-means",
        "valid_zero", // Zero is valid for DeFi operations
        "--output-dir",
        complex_batch_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Complex semantic batch")?;

    // Test 5: Batch operations error handling
    println!("        Testing batch operations error handling...");

    // Test missing semantic specification
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply,name",
        // Missing --zero-means or --semantic-config
        "--output-dir",
        fixtures.path("outputs/batch_error").to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "semantic specification required")?;

    // Test 6: Performance test for batch operations
    println!("        Testing batch operations performance...");

    let (_, duration) = PerformanceTester::measure_command_time(
        cli,
        &[
            "ethereum",
            "generate-batch",
            "--layout",
            erc20_abi.to_str().unwrap(),
            "--contract",
            "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--queries",
            "totalSupply,name,symbol,decimals",
            "--zero-means",
            "never_written",
            "--output-dir",
            fixtures.path("outputs/batch_performance").to_str().unwrap(),
            "--dry-run",
        ],
    )?;

    // Batch operations should complete within reasonable time
    if duration > std::time::Duration::from_secs(45) {
        return Err(anyhow::anyhow!(
            "Batch operations took too long: {:?}",
            duration
        ));
    }

    println!("      batch-operations-with-semantics tests passed");
    Ok(())
}

/// Test circuit semantic validation
async fn test_circuit_semantic_validation(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();

    // Test 1: Circuit validation with all four semantic types
    println!("        Testing circuit validation with all semantic types...");
    let semantic_types = vec![
        ("never_written", "Slot was never written to"),
        ("explicitly_zero", "Slot was intentionally set to zero"),
        ("cleared", "Slot was previously non-zero but cleared"),
        ("valid_zero", "Zero is a valid operational state"),
    ];

    for (semantic_type, description) in semantic_types {
        let circuit_proof_path =
            fixtures.path(&format!("outputs/circuit_proof_{}.json", semantic_type));

        let output = cli.run_success(&[
            "valence",
            "verify-proof",
            "--layout",
            erc20_abi.to_str().unwrap(),
            "--contract",
            "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
            "--slot",
            "0x2", // totalSupply slot
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--zero-means",
            semantic_type,
            "--output",
            circuit_proof_path.to_str().unwrap(),
            "--dry-run",
        ])?;

        println!(
            "          Testing circuit with semantic type: {} - {}",
            semantic_type, description
        );

        // Verify circuit validation includes semantic processing
        assertions::assert_output_contains(&output, "Circuit semantic validation")?;
        assertions::assert_output_contains(&output, semantic_type)?;

        // Verify circuit proof includes semantic metadata
        let circuit_proof = FileValidator::validate_json(&circuit_proof_path)?;
        OutputValidator::validate_semantic_proof_structure(
            &circuit_proof,
            &["circuit_result", "semantic_metadata", "zero_semantics"],
        )?;
    }

    // Test 2: Circuit validation with semantic conflicts
    println!("        Testing circuit validation with semantic conflicts...");
    let conflict_proof_path = fixtures.path("outputs/circuit_conflict_proof.json");

    let output = cli.run_success(&[
        "valence",
        "verify-proof",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract (likely has writes)
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written", // Declare never written
        "--validate-semantics",
        "--output",
        conflict_proof_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should detect semantic conflict in circuit validation
    assertions::assert_output_contains(&output, "Circuit semantic conflict detected")?;

    // Verify conflict is recorded in circuit proof output
    let conflict_proof = FileValidator::validate_json(&conflict_proof_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &conflict_proof,
        &[
            "semantic_metadata",
            "validation_result",
            "conflict_detected",
        ],
    )?;

    // Test 3: Circuit validation with complex contract semantics
    println!("        Testing circuit validation with complex contract semantics...");
    let complex_circuit_path = fixtures.path("outputs/circuit_complex_proof.json");

    let output = cli.run_success(&[
        "valence",
        "verify-proof",
        "--layout",
        defi_abi.to_str().unwrap(),
        "--contract",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--slot",
        "0x3", // totalDeposits slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "valid_zero", // Zero is valid for DeFi operations
        "--output",
        complex_circuit_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Complex circuit semantic validation")?;

    // Test 4: Circuit performance with semantic validation
    println!("        Testing circuit performance with semantic validation...");

    let (_, duration) = PerformanceTester::measure_command_time(
        cli,
        &[
            "valence",
            "verify-proof",
            "--layout",
            erc20_abi.to_str().unwrap(),
            "--contract",
            "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
            "--slot",
            "0x2",
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--zero-means",
            "never_written",
            "--validate-semantics",
            "--output",
            fixtures
                .path("outputs/circuit_perf_proof.json")
                .to_str()
                .unwrap(),
            "--dry-run",
        ],
    )?;

    // Circuit validation should complete within reasonable time (circuits should be fast)
    if duration > std::time::Duration::from_secs(30) {
        return Err(anyhow::anyhow!(
            "Circuit semantic validation took too long: {:?}",
            duration
        ));
    }

    // Test 5: Circuit validation memory usage (should be bounded)
    println!("        Testing circuit validation memory bounds...");

    let output = cli.run_success(&[
        "valence",
        "verify-proof",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--memory-limit",
        "100MB", // Test bounded memory usage
        "--output",
        fixtures
            .path("outputs/circuit_memory_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Memory bounded circuit validation")?;

    // Test 6: Circuit validation error handling
    println!("        Testing circuit validation error handling...");

    // Test circuit validation without semantic specification (should fail)
    let error_output = cli.run_failure(&[
        "valence",
        "verify-proof",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        // Missing --zero-means argument
        "--output",
        fixtures
            .path("outputs/circuit_error_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "Circuit requires semantic specification")?;

    println!("      circuit-semantic-validation tests passed");
    Ok(())
}

/// Test missing semantics failure modes
async fn test_missing_semantics_failure_modes(
    cli: &CliRunner,
    fixtures: &TestFixtures,
) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let _defi_abi = fixtures.ethereum_abis.get("defi").unwrap();

    // Test 1: Missing semantics in proof generation
    println!("        Testing missing semantics in proof generation...");
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        // Missing --zero-means argument
        "--output",
        fixtures
            .path("outputs/missing_semantics_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "required")?;
    assertions::assert_output_contains(&error_output, "zero-means")?;

    // Test 2: Invalid semantic value
    println!("        Testing invalid semantic values...");
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "invalid_semantic_type", // Invalid semantic
        "--output",
        fixtures
            .path("outputs/invalid_semantics_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "invalid")?;

    // Test 3: Missing semantics in batch operations
    println!("        Testing missing semantics in batch operations...");
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--queries",
        "totalSupply,name",
        // Missing --zero-means or --semantic-config
        "--output-dir",
        fixtures
            .path("outputs/batch_missing_semantics")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "semantic specification required")?;

    // Test 4: Missing semantics in circuit validation
    println!("        Testing missing semantics in circuit validation...");
    let error_output = cli.run_failure(&[
        "valence",
        "verify-proof",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        // Missing --zero-means argument
        "--output",
        fixtures
            .path("outputs/circuit_missing_semantics_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "Circuit requires semantic specification")?;

    // Test 5: Layout without semantic metadata (should auto-generate, but warn)
    println!("        Testing layout compilation without explicit semantics...");
    let layout_path = fixtures.path("outputs/layout_no_explicit_semantics.json");

    let output = cli.run_success(&[
        "ethereum",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output",
        layout_path.to_str().unwrap(),
        // No explicit semantic specification (should auto-generate)
    ])?;

    // Should auto-generate but provide warning
    assertions::assert_output_contains(&output, "auto-generating semantic metadata")?;

    // Layout should still contain semantic metadata
    let layout_json = FileValidator::validate_json(&layout_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &layout_json,
        &["storage", "semantic_metadata"],
    )?;

    // Test 6: Semantic validation without indexer service
    println!("        Testing semantic validation without indexer...");
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics", // Enable validation but no indexer configured
        "--output",
        fixtures
            .path("outputs/validation_no_indexer_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "indexer service required")?;

    // Test 7: Invalid semantic config file
    println!("        Testing invalid semantic config file...");
    let invalid_config = serde_json::json!({
        "queries": [
            {
                "query": "totalSupply",
                "zero_means": "invalid_semantic_type", // Invalid semantic
                "description": "Total supply"
            }
        ]
    });

    let invalid_config_path = fixtures.path("outputs/invalid_semantic_config.json");
    fs::write(
        &invalid_config_path,
        serde_json::to_string_pretty(&invalid_config)?,
    )?;

    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--semantic-config",
        invalid_config_path.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/batch_invalid_config")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "invalid semantic configuration")?;

    // Test 8: Complex contract without semantics
    println!("        Testing complex contract operations without semantics...");
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--slot",
        "0x3", // Complex DeFi slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        // Missing semantic specification for complex contract
        "--output",
        fixtures
            .path("outputs/complex_missing_semantics_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "semantic specification required")?;

    // Test 9: Empty semantic config file
    println!("        Testing empty semantic config file...");
    let empty_config_path = fixtures.path("outputs/empty_semantic_config.json");
    fs::write(&empty_config_path, "{}")?;

    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--semantic-config",
        empty_config_path.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/batch_empty_config")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "empty semantic configuration")?;

    // Test 10: Multiple conflicting semantic arguments
    println!("        Testing conflicting semantic arguments...");
    let conflict_config_path = fixtures.path("outputs/conflict_semantic_config.json");
    let conflict_config = serde_json::json!({
        "queries": [
            {"query": "totalSupply", "zero_means": "never_written"}
        ]
    });
    fs::write(
        &conflict_config_path,
        serde_json::to_string_pretty(&conflict_config)?,
    )?;

    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "explicitly_zero", // Conflicts with config file
        "--semantic-config",
        conflict_config_path.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/batch_conflicting_args")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "conflicting semantic arguments")?;

    println!("      missing-semantics-failure-modes tests passed");
    Ok(())
}

/// Test semantic conflict detection and resolution
async fn test_semantic_conflict_detection_and_resolution(
    cli: &CliRunner,
    fixtures: &TestFixtures,
) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let _defi_abi = fixtures.ethereum_abis.get("defi").unwrap();

    // Test 1: Never Written vs Actually Written Conflict
    println!("        Testing never_written vs actually_written conflict...");
    let never_written_conflict_path = fixtures.path("outputs/never_written_conflict_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract (likely has writes)
        "--slot",
        "0x2", // totalSupply slot (likely written to)
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written", // Declare never written
        "--validate-semantics",
        "--output",
        never_written_conflict_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should detect conflict: declared never_written but indexer shows writes
    assertions::assert_output_contains(&output, "Semantic conflict detected")?;
    assertions::assert_output_contains(&output, "never_written")?;

    // Verify conflict details in proof
    let conflict_proof = FileValidator::validate_json(&never_written_conflict_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &conflict_proof,
        &[
            "semantic_metadata",
            "conflict_detected",
            "declared_semantics",
            "validated_semantics",
        ],
    )?;

    // Test 2: Explicitly Zero vs Never Written Conflict
    println!("        Testing explicitly_zero vs never_written conflict...");
    let explicitly_zero_conflict_path =
        fixtures.path("outputs/explicitly_zero_conflict_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123", // Clean contract
        "--slot",
        "0x99", // Unused slot (likely never written)
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "explicitly_zero", // Declare explicitly zero
        "--validate-semantics",
        "--output",
        explicitly_zero_conflict_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should detect conflict: declared explicitly_zero but indexer shows never written
    assertions::assert_output_contains(&output, "Semantic conflict detected")?;
    assertions::assert_output_contains(&output, "explicitly_zero")?;

    // Test 3: Automatic Conflict Resolution
    println!("        Testing automatic semantic conflict resolution...");
    let auto_resolution_path = fixtures.path("outputs/auto_resolved_conflict_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--auto-resolve-conflicts", // Enable automatic resolution
        "--output",
        auto_resolution_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should automatically resolve conflict
    assertions::assert_output_contains(&output, "Automatically resolved semantic conflict")?;
    assertions::assert_output_contains(&output, "never_written → explicitly_zero")?;

    // Verify auto-resolved proof
    let auto_resolved_proof = FileValidator::validate_json(&auto_resolution_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &auto_resolved_proof,
        &["semantic_metadata", "auto_resolved", "final_semantics"],
    )?;

    // Test 4: Manual Conflict Resolution
    println!("        Testing manual semantic conflict resolution...");
    let manual_resolution_path = fixtures.path("outputs/manual_resolved_conflict_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--override-semantics",
        "cleared", // Manually override to cleared
        "--output",
        manual_resolution_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should use manual override
    assertions::assert_output_contains(&output, "Manual semantic override applied")?;
    assertions::assert_output_contains(&output, "cleared")?;

    // Test 5: Complex Conflict Resolution in Batch Operations
    println!("        Testing conflict resolution in batch operations...");
    let batch_conflict_dir = fixtures.path("outputs/batch_conflict_resolution");

    // Create conflict config with mixed semantics
    let conflict_config = serde_json::json!({
        "queries": [
            {
                "query": "totalSupply",
                "zero_means": "never_written", // Will conflict with USDT
                "auto_resolve": true
            },
            {
                "query": "name",
                "zero_means": "explicitly_zero", // Might conflict
                "auto_resolve": false // Manual resolution required
            }
        ]
    });

    let conflict_config_path = fixtures.path("outputs/batch_conflict_config.json");
    fs::write(
        &conflict_config_path,
        serde_json::to_string_pretty(&conflict_config)?,
    )?;

    let output = cli.run_success(&[
        "ethereum",
        "generate-batch",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT contract
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--semantic-config",
        conflict_config_path.to_str().unwrap(),
        "--validate-semantics",
        "--output-dir",
        batch_conflict_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should handle mixed conflict resolution strategies
    assertions::assert_output_contains(&output, "Batch conflict resolution")?;
    assertions::assert_output_contains(&output, "auto resolved")?;
    assertions::assert_output_contains(&output, "manual resolution required")?;

    // Test 6: Conflict Resolution Performance
    println!("        Testing conflict resolution performance...");

    let (_, duration) = PerformanceTester::measure_command_time(
        cli,
        &[
            "ethereum",
            "generate-proof",
            "--contract",
            "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "--slot",
            "0x2",
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--zero-means",
            "never_written",
            "--validate-semantics",
            "--auto-resolve-conflicts",
            "--output",
            fixtures
                .path("outputs/conflict_perf_proof.json")
                .to_str()
                .unwrap(),
            "--dry-run",
        ],
    )?;

    // Conflict resolution should be fast
    if duration > std::time::Duration::from_secs(45) {
        return Err(anyhow::anyhow!(
            "Conflict resolution took too long: {:?}",
            duration
        ));
    }

    // Test 7: Circuit-Level Conflict Detection
    println!("        Testing circuit-level conflict detection...");
    let circuit_conflict_path = fixtures.path("outputs/circuit_conflict_detection_proof.json");

    let output = cli.run_success(&[
        "valence",
        "verify-proof",
        "--layout",
        erc20_abi.to_str().unwrap(),
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--output",
        circuit_conflict_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Circuit should detect and handle semantic conflicts
    assertions::assert_output_contains(&output, "Circuit semantic conflict detected")?;

    // Test 8: Resolution Report Generation
    println!("        Testing conflict resolution reporting...");
    let report_path = fixtures.path("outputs/conflict_resolution_report.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--auto-resolve-conflicts",
        "--generate-resolution-report",
        report_path.to_str().unwrap(),
        "--output",
        fixtures
            .path("outputs/reported_conflict_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    // Should generate comprehensive report
    assertions::assert_output_contains(&output, "Conflict resolution report generated")?;

    // Verify report structure
    let report = FileValidator::validate_json(&report_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &report,
        &["conflicts_detected", "resolutions_applied", "summary"],
    )?;

    println!("      semantic-conflict-detection-and-resolution tests passed");
    Ok(())
}

/// Test event validation with real blockchain data
async fn test_event_validation_with_real_blockchain_data(
    cli: &CliRunner,
    fixtures: &TestFixtures,
) -> Result<()> {
    let _erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();

    // Test 1: Real USDT contract semantic validation
    println!("        Testing real USDT contract semantic validation...");
    let usdt_validation_path = fixtures.path("outputs/usdt_real_validation_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // Real USDT contract
        "--slot",
        "0x2", // totalSupply slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written", // Likely wrong for USDT
        "--validate-semantics",
        "--indexer-service",
        "etherscan", // Use real indexer
        "--output",
        usdt_validation_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should detect that USDT totalSupply has been written to
    assertions::assert_output_contains(&output, "Real blockchain validation")?;
    assertions::assert_output_contains(&output, "Semantic conflict detected")?;

    // Verify real validation data in proof
    let validation_proof = FileValidator::validate_json(&usdt_validation_path)?;
    OutputValidator::validate_semantic_proof_structure(
        &validation_proof,
        &[
            "real_blockchain_data",
            "indexer_events",
            "validated_semantics",
        ],
    )?;

    // Test 2: Real DAI contract semantic validation
    println!("        Testing real DAI contract semantic validation...");
    let dai_validation_path = fixtures.path("outputs/dai_real_validation_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0x6B175474E89094C44Da98b954EedeAC495271d0F", // Real DAI contract
        "--slot",
        "0x1", // name slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "explicitly_zero", // Likely correct for DAI name
        "--validate-semantics",
        "--indexer-service",
        "alchemy", // Use different indexer
        "--output",
        dai_validation_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should validate DAI name semantics
    assertions::assert_output_contains(&output, "DAI semantic validation")?;

    // Test 3: Real Uniswap V3 factory semantic validation
    println!("        Testing real Uniswap V3 factory semantic validation...");
    let uniswap_validation_path = fixtures.path("outputs/uniswap_real_validation_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0x1F98431c8aD98523631AE4a59f267346ea31F984", // Real Uniswap V3 Factory
        "--slot",
        "0x0", // owner slot
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "valid_zero", // Zero might be valid for uninitialized owner
        "--validate-semantics",
        "--indexer-service",
        "moralis", // Use third indexer
        "--output",
        uniswap_validation_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should validate Uniswap owner semantics
    assertions::assert_output_contains(&output, "Uniswap semantic validation")?;

    // Test 4: Historical event validation with block ranges
    println!("        Testing historical event validation with block ranges...");
    let historical_validation_path = fixtures.path("outputs/historical_real_validation_proof.json");

    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--from-block",
        "10000000", // Historical block range
        "--to-block",
        "15000000",
        "--output",
        historical_validation_path.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should analyze historical events
    assertions::assert_output_contains(&output, "Historical event analysis")?;
    assertions::assert_output_contains(&output, "Block range validation")?;

    // Test 5: Batch real blockchain validation
    println!("        Testing batch real blockchain validation...");
    let batch_real_validation_dir = fixtures.path("outputs/batch_real_validation");

    // Create real blockchain config
    let real_blockchain_config = serde_json::json!({
        "contracts": [
            {
                "address": "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
                "queries": ["totalSupply", "name", "symbol"],
                "expected_semantics": {
                    "totalSupply": "explicitly_zero",
                    "name": "explicitly_zero",
                    "symbol": "explicitly_zero"
                }
            },
            {
                "address": "0x6B175474E89094C44Da98b954EedeAC495271d0F", // DAI
                "queries": ["totalSupply", "name"],
                "expected_semantics": {
                    "totalSupply": "explicitly_zero",
                    "name": "explicitly_zero"
                }
            }
        ],
        "validation_settings": {
            "indexer_service": "etherscan",
            "require_real_data": true,
            "conflict_resolution": "auto"
        }
    });

    let real_config_path = fixtures.path("outputs/real_blockchain_config.json");
    fs::write(
        &real_config_path,
        serde_json::to_string_pretty(&real_blockchain_config)?,
    )?;

    let output = cli.run_success(&[
        "ethereum",
        "validate-batch-real",
        "--config",
        real_config_path.to_str().unwrap(),
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--output-dir",
        batch_real_validation_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    // Should validate multiple real contracts
    assertions::assert_output_contains(&output, "Batch real blockchain validation")?;
    assertions::assert_output_contains(&output, "USDT validated")?;
    assertions::assert_output_contains(&output, "DAI validated")?;

    // Test 6: Real event stream validation
    println!("        Testing real event stream validation...");
    let stream_validation_path = fixtures.path("outputs/stream_real_validation.json");

    let output = cli.run_success(&[
        "ethereum",
        "validate-stream",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7", // USDT
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--websocket",
        "wss://mainnet.infura.io/ws/v3/test",
        "--monitor-slots",
        "0x2,0x3,0x4", // Monitor multiple slots
        "--semantic-rules",
        "all:auto-detect", // Auto-detect semantics from events
        "--output",
        stream_validation_path.to_str().unwrap(),
        "--duration",
        "30s", // Monitor for 30 seconds
        "--dry-run",
    ])?;

    // Should set up real-time monitoring
    assertions::assert_output_contains(&output, "Real-time event stream validation")?;
    assertions::assert_output_contains(&output, "WebSocket monitoring")?;

    // Test 7: Performance with real blockchain data
    println!("        Testing performance with real blockchain data...");

    let (_, duration) = PerformanceTester::measure_command_time(
        cli,
        &[
            "ethereum",
            "generate-proof",
            "--contract",
            "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            "--slot",
            "0x2",
            "--rpc",
            "https://mainnet.infura.io/v3/test",
            "--zero-means",
            "never_written",
            "--validate-semantics",
            "--indexer-service",
            "etherscan",
            "--output",
            fixtures
                .path("outputs/real_perf_proof.json")
                .to_str()
                .unwrap(),
            "--dry-run",
        ],
    )?;

    // Real blockchain validation should complete within reasonable time
    if duration > std::time::Duration::from_secs(60) {
        return Err(anyhow::anyhow!(
            "Real blockchain validation took too long: {:?}",
            duration
        ));
    }

    // Test 8: Real blockchain error handling
    println!("        Testing real blockchain error handling...");

    // Test with invalid contract address
    let error_output = cli.run_failure(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0x0000000000000000000000000000000000000000", // Invalid contract
        "--slot",
        "0x0",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--indexer-service",
        "etherscan",
        "--output",
        fixtures
            .path("outputs/invalid_real_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&error_output, "Invalid contract for real validation")?;

    // Test with rate limiting
    let output = cli.run_success(&[
        "ethereum",
        "generate-proof",
        "--contract",
        "0xdAC17F958D2ee523a2206206994597C13D831ec7",
        "--slot",
        "0x2",
        "--rpc",
        "https://mainnet.infura.io/v3/test",
        "--zero-means",
        "never_written",
        "--validate-semantics",
        "--indexer-service",
        "etherscan",
        "--rate-limit",
        "10", // 10 requests per minute
        "--output",
        fixtures
            .path("outputs/rate_limited_real_proof.json")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Rate limiting applied")?;

    println!("      event-validation-with-real-blockchain-data tests passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::TestFixtures;

    #[tokio::test]
    async fn test_ethereum_commands_structure() {
        // Test that we can create fixtures and runner
        let fixtures = TestFixtures::new().await.unwrap();
        let cli = CliRunner::new(fixtures.cli_path());

        // Test help command works
        let output = cli.run(&["ethereum", "--help"]);
        assert!(output.is_ok());
    }
}
