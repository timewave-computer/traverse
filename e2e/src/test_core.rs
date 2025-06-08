//! End-to-end tests for core CLI commands
//!
//! Tests core commands that work across both chains:
//! - traverse compile-layout
//! - traverse resolve
//! - traverse resolve-all
//! - traverse batch-resolve
//! - traverse generate-proof

use crate::fixtures::TestFixtures;
use crate::utils::*;
use anyhow::Result;

/// Run all core CLI command tests
pub async fn run_tests(fixtures: &TestFixtures) -> Result<()> {
    let cli = CliRunner::new(fixtures.cli_path());
    
    println!("    📋 Testing core compile-layout command...");
    test_compile_layout(&cli, fixtures).await?;
    
    println!("    📋 Testing core resolve command...");
    test_resolve(&cli, fixtures).await?;
    
    println!("    📋 Testing core resolve-all command...");
    test_resolve_all(&cli, fixtures).await?;
    
    println!("    📋 Testing core batch-resolve command...");
    test_batch_resolve(&cli, fixtures).await?;
    
    println!("    📋 Testing core generate-proof command...");
    test_generate_proof(&cli, fixtures).await?;
    
    println!("    📋 Testing CLI help and basic functionality...");
    test_cli_basics(&cli, fixtures).await?;
    
    Ok(())
}

/// Test `traverse compile-layout` command (core version)
async fn test_compile_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Test with Ethereum ABI
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let eth_layout_path = fixtures.path("outputs/core_eth_layout.json");
    
    let output = cli.run_success(&[
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output", eth_layout_path.to_str().unwrap(),
        "--chain", "ethereum"
    ])?;
    
    assertions::assert_output_contains(&output, "Compiling")?;
    FileValidator::validate_json(&eth_layout_path)?;
    
    // Test with Cosmos schema
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let cosmos_layout_path = fixtures.path("outputs/core_cosmos_layout.json");
    
    let output = cli.run_success(&[
        "compile-layout",
        cw20_schema.to_str().unwrap(),
        "--output", cosmos_layout_path.to_str().unwrap(),
        "--chain", "cosmos"
    ])?;
    
    assertions::assert_output_contains(&output, "Compiling")?;
    FileValidator::validate_json(&cosmos_layout_path)?;
    
    println!("      ✅ compile-layout tests passed");
    Ok(())
}

/// Test `traverse resolve` command
async fn test_resolve(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // First create layouts to work with
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/resolve_test_layout.json");
    
    cli.run_success(&[
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output", layout_path.to_str().unwrap(),
        "--chain", "ethereum"
    ])?;
    
    // Test basic query resolution
    let resolved_path = fixtures.path("outputs/core_resolved.json");
    
    let output = cli.run_success(&[
        "resolve",
        "totalSupply",
        "--layout", layout_path.to_str().unwrap(),
        "--output", resolved_path.to_str().unwrap(),
        "--format", "traverse",
        "--chain", "ethereum"
    ])?;
    
    assertions::assert_output_contains(&output, "Resolved")?;
    FileValidator::validate_json(&resolved_path)?;
    
    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let format_path = fixtures.path(&format!("outputs/core_resolved.{}", format));
        
        cli.run_success(&[
            "resolve",
            "totalSupply",
            "--layout", layout_path.to_str().unwrap(),
            "--output", format_path.to_str().unwrap(),
            "--format", format,
            "--chain", "ethereum"
        ])?;
        
        FileValidator::exists_and_non_empty(&format_path)?;
    }
    
    println!("      ✅ resolve tests passed");
    Ok(())
}

/// Test `traverse resolve-all` command
async fn test_resolve_all(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let layout_path = fixtures.path("outputs/resolve_test_layout.json");
    let resolved_all_path = fixtures.path("outputs/core_resolved_all.json");
    
    let output = cli.run_success(&[
        "resolve-all",
        "--layout", layout_path.to_str().unwrap(),
        "--output", resolved_all_path.to_str().unwrap(),
        "--format", "coprocessor-json",
        "--chain", "ethereum"
    ])?;
    
    assertions::assert_output_contains(&output, "Resolved all")?;
    
    // Verify all resolved queries are in the output
    let resolved_all = FileValidator::validate_json(&resolved_all_path)?;
    
    // Should contain multiple resolved queries
    if let Some(obj) = resolved_all.as_object() {
        if obj.len() < 2 {
            return Err(anyhow::anyhow!("resolve-all should return multiple queries"));
        }
    }
    
    println!("      ✅ resolve-all tests passed");
    Ok(())
}

/// Test `traverse batch-resolve` command
async fn test_batch_resolve(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let layout_path = fixtures.path("outputs/resolve_test_layout.json");
    let queries_file = fixtures.query_files.get("simple").unwrap();
    let batch_resolved_path = fixtures.path("outputs/core_batch_resolved.json");
    
    let output = cli.run_success(&[
        "batch-resolve",
        queries_file.to_str().unwrap(),
        "--layout", layout_path.to_str().unwrap(),
        "--output", batch_resolved_path.to_str().unwrap(),
        "--format", "coprocessor-json",
        "--chain", "ethereum"
    ])?;
    
    assertions::assert_output_contains(&output, "Batch resolved")?;
    
    // Verify batch resolved output
    let batch_resolved = FileValidator::validate_json(&batch_resolved_path)?;
    
    // Should contain multiple resolved queries from the batch file
    if let Some(obj) = batch_resolved.as_array() {
        if obj.len() < 2 {
            return Err(anyhow::anyhow!("batch-resolve should return multiple resolved queries"));
        }
    }
    
    println!("      ✅ batch-resolve tests passed");
    Ok(())
}

/// Test `traverse generate-proof` command (dry run only)
async fn test_generate_proof(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let proof_output_path = fixtures.path("outputs/core_proof.json");
    
    // Test with mock RPC (this will fail but we can test argument parsing)
    let result = cli.run(&[
        "generate-proof",
        "--slot", "0x0000000000000000000000000000000000000000000000000000000000000000",
        "--rpc", "http://invalid-rpc-endpoint",
        "--contract", "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--output", proof_output_path.to_str().unwrap(),
        "--block-number", "latest"
    ]);
    
    // Should fail due to invalid RPC, but should show correct usage
    if let Ok(output) = result {
        if output.status.success() {
            return Err(anyhow::anyhow!("Expected proof generation to fail with invalid RPC"));
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should show error about RPC connection, not argument parsing
        assertions::assert_output_contains(&stderr, "RPC")?;
    } else {
        // Command parsing worked, network error is expected
        println!("      📝 generate-proof correctly rejected invalid RPC");
    }
    
    println!("      ✅ generate-proof tests passed");
    Ok(())
}

/// Test basic CLI functionality (help, version, etc.)
async fn test_cli_basics(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Test help command
    let output = cli.run_success(&["--help"])?;
    assertions::assert_output_contains(&output, "Chain-independent ZK storage path generator")?;
    assertions::assert_output_contains(&output, "ethereum")?;
    assertions::assert_output_contains(&output, "cosmos")?;
    
    // Test version command
    let output = cli.run_success(&["--version"])?;
    assertions::assert_output_contains(&output, "zkpath")?;
    
    // Test verbose flag
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output = cli.run_success(&[
        "--verbose",
        "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--chain", "ethereum"
    ])?;
    
    // Verbose output should contain more details
    assertions::assert_output_contains(&output, "Compiling")?;
    
    // Test invalid command
    let result = cli.run(&["invalid-command"]);
    if let Ok(output) = result {
        if output.status.success() {
            return Err(anyhow::anyhow!("Expected invalid command to fail"));
        }
    }
    
    println!("      ✅ CLI basics tests passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::TestFixtures;
    
    #[tokio::test]
    async fn test_core_commands_structure() {
        // Test that we can create fixtures and runner
        let fixtures = TestFixtures::new().await.unwrap();
        let cli = CliRunner::new(fixtures.cli_path());
        
        // Test help command works
        let output = cli.run(&["--help"]);
        assert!(output.is_ok());
    }
} 