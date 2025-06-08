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
    
    Ok(())
}

/// Test `traverse ethereum analyze-contract` command
async fn test_analyze_contract(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output_path = fixtures.path("outputs/erc20_analysis.json");
    
    // Test basic analysis
    let output = cli.run_success(&[
        "ethereum", "analyze-contract",
        erc20_abi.to_str().unwrap(),
        "--output", output_path.to_str().unwrap(),
    ])?;
    
    // Verify output contains expected information
    assertions::assert_output_contains(&output, "Analyzing Ethereum contract")?;
    
    // Verify output file was created and has expected structure
    let _analysis = FileValidator::validate_json_structure(&output_path, &[
        "contract_type", "functions", "storage_patterns"
    ])?;
    
    // Test with validation flag
    let output = cli.run_success(&[
        "ethereum", "analyze-contract", 
        erc20_abi.to_str().unwrap(),
        "--validate-storage"
    ])?;
    
    assertions::assert_output_contains(&output, "Storage validation")?;
    
    // Test with complex DeFi contract
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();
    let defi_output_path = fixtures.path("outputs/defi_analysis.json");
    
    let output = cli.run_success(&[
        "ethereum", "analyze-contract",
        defi_abi.to_str().unwrap(),
        "--output", defi_output_path.to_str().unwrap(),
        "--validate-storage"
    ])?;
    
    assertions::assert_output_contains(&output, "complex mappings")?;
    FileValidator::validate_json(&defi_output_path)?;
    
    println!("      ✅ analyze-contract tests passed");
    Ok(())
}

/// Test `traverse ethereum compile-layout` command  
async fn test_compile_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout.json");
    
    // Test basic layout compilation
    let output = cli.run_success(&[
        "ethereum", "compile-layout",
        erc20_abi.to_str().unwrap(), 
        "--output", layout_path.to_str().unwrap(),
        "--format", "traverse"
    ])?;
    
    assertions::assert_output_contains(&output, "Compiling storage layout")?;
    
    // Verify layout file structure
    let _layout = FileValidator::validate_json_structure(&layout_path, &[
        "contract_name", "storage_layout", "commitment"
    ])?;
    
    // Test with validation flag
    let output = cli.run_success(&[
        "ethereum", "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--validate"
    ])?;
    
    assertions::assert_output_contains(&output, "Layout validation")?;
    
    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary"] {
        let format_output_path = fixtures.path(&format!("outputs/erc20_layout.{}", format));
        
        cli.run_success(&[
            "ethereum", "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output", format_output_path.to_str().unwrap(),
            "--format", format
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
        "ethereum", "compile-layout",
        defi_abi.to_str().unwrap(),
        "--output", defi_layout_path.to_str().unwrap(),
        "--validate"
    ])?;
    
    FileValidator::validate_json(&defi_layout_path)?;
    
    println!("      ✅ compile-layout tests passed");
    Ok(())
}

/// Test `traverse ethereum generate-queries` command
async fn test_generate_queries(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // First compile a layout
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");
    
    cli.run_success(&[
        "ethereum", "compile-layout",
        erc20_abi.to_str().unwrap(),
        "--output", layout_path.to_str().unwrap()
    ])?;
    
    // Test query generation
    let queries_path = fixtures.path("outputs/erc20_queries.json");
    
    let output = cli.run_success(&[
        "ethereum", "generate-queries",
        layout_path.to_str().unwrap(),
        "--fields", "totalSupply,name,symbol,balanceOf",
        "--output", queries_path.to_str().unwrap()
    ])?;
    
    assertions::assert_output_contains(&output, "Generated")?;
    
    // Verify queries file structure
    let _queries = FileValidator::validate_json_structure(&queries_path, &[
        "queries", "metadata"
    ])?;
    
    // Test with examples flag
    let queries_with_examples_path = fixtures.path("outputs/erc20_queries_with_examples.json");
    
    cli.run_success(&[
        "ethereum", "generate-queries",
        layout_path.to_str().unwrap(),
        "--fields", "balanceOf",
        "--output", queries_with_examples_path.to_str().unwrap(),
        "--include-examples"
    ])?;
    
    let queries_with_examples = FileValidator::validate_json(&queries_with_examples_path)?;
    
    // Should contain example addresses for mappings
    assertions::assert_output_contains(
        &serde_json::to_string(&queries_with_examples)?,
        "0x"
    )?;
    
    println!("      ✅ generate-queries tests passed");
    Ok(())
}

/// Test `traverse ethereum resolve-query` command
async fn test_resolve_query(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Use the layout from previous test
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");
    
    // Test basic query resolution
    let resolved_path = fixtures.path("outputs/totalSupply_resolved.json");
    
    let output = cli.run_success(&[
        "ethereum", "resolve-query",
        "totalSupply",
        "--layout", layout_path.to_str().unwrap(),
        "--format", "coprocessor-json",
        "--output", resolved_path.to_str().unwrap()
    ])?;
    
    assertions::assert_output_contains(&output, "Resolved")?;
    
    // Verify resolved query structure
    let _resolved = FileValidator::validate_json_structure(&resolved_path, &[
        "storage_key", "slot_index"
    ])?;
    
    // Test mapping query resolution
    let mapping_resolved_path = fixtures.path("outputs/balance_resolved.json");
    
    cli.run_success(&[
        "ethereum", "resolve-query",
        "balanceOf[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
        "--layout", layout_path.to_str().unwrap(),
        "--format", "coprocessor-json",
        "--output", mapping_resolved_path.to_str().unwrap()
    ])?;
    
    FileValidator::validate_json(&mapping_resolved_path)?;
    
    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let format_resolved_path = fixtures.path(&format!("outputs/totalSupply_resolved.{}", format));
        
        cli.run_success(&[
            "ethereum", "resolve-query",
            "totalSupply",
            "--layout", layout_path.to_str().unwrap(),
            "--format", format,
            "--output", format_resolved_path.to_str().unwrap()
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
    
    println!("      ✅ resolve-query tests passed");
    Ok(())
}

/// Test `traverse ethereum verify-layout` command
async fn test_verify_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let layout_path = fixtures.path("outputs/erc20_layout_for_queries.json");
    
    // Test basic layout verification (dry run)
    let output = cli.run_success(&[
        "ethereum", "verify-layout",
        layout_path.to_str().unwrap()
    ])?;
    
    assertions::assert_output_contains(&output, "Layout verification")?;
    
    // Test comprehensive verification
    let output = cli.run_success(&[
        "ethereum", "verify-layout", 
        layout_path.to_str().unwrap(),
        "--comprehensive"
    ])?;
    
    assertions::assert_output_contains(&output, "Comprehensive")?;
    
    println!("      ✅ verify-layout tests passed");
    Ok(())
}

/// Test `traverse ethereum auto-generate` command  
async fn test_auto_generate(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let output_dir = fixtures.path("outputs/erc20_auto");
    
    // Test dry-run mode
    let output = cli.run_success(&[
        "ethereum", "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc", "https://mainnet.infura.io/v3/test",
        "--contract", "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--queries", "totalSupply,name,symbol",
        "--output-dir", output_dir.to_str().unwrap(),
        "--dry-run"
    ])?;
    
    assertions::assert_output_contains(&output, "End-to-end automation")?;
    assertions::assert_output_contains(&output, "Dry run mode")?;
    
    // Verify output directory structure in dry-run
    FileValidator::validate_directory_structure(&output_dir, &[
        "layout.json", "queries.json", "resolved_queries.json"
    ])?;
    
    // Test with caching enabled
    let output = cli.run_success(&[
        "ethereum", "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc", "https://mainnet.infura.io/v3/test", 
        "--contract", "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--queries", "totalSupply",
        "--output-dir", output_dir.to_str().unwrap(),
        "--cache",
        "--dry-run"
    ])?;
    
    assertions::assert_output_contains(&output, "Caching enabled")?;
    
    println!("      ✅ auto-generate tests passed");
    Ok(())
}

/// Test different output formats work correctly
async fn test_output_formats(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    
    // Test each format with compile-layout
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let output_path = fixtures.path(&format!("outputs/format_test.{}", format));
        
        cli.run_success(&[
            "ethereum", "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output", output_path.to_str().unwrap(),
            "--format", format
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
    
    println!("      ✅ output-formats tests passed");
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
            "ethereum", "compile-layout",
            erc20_abi.to_str().unwrap(),
            "--output", output_path.to_str().unwrap()
        ],
        std::time::Duration::from_secs(30) // 30 second threshold
    )?;
    
    // Test analyze-contract performance
    PerformanceTester::test_performance_threshold(
        cli,
        &[
            "ethereum", "analyze-contract",
            erc20_abi.to_str().unwrap()
        ],
        std::time::Duration::from_secs(15) // 15 second threshold
    )?;
    
    println!("      ✅ performance tests passed");
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