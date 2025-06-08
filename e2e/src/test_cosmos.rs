//! End-to-end tests for Cosmos CLI commands
//!
//! Tests all Cosmos-specific commands documented in the work plan:
//! - traverse cosmos analyze-contract
//! - traverse cosmos compile-layout
//! - traverse cosmos generate-queries  
//! - traverse cosmos resolve-query
//! - traverse cosmos auto-generate

use crate::fixtures::TestFixtures;
use crate::utils::*;
use anyhow::Result;
use std::fs;

/// Run all Cosmos CLI command tests
pub async fn run_tests(fixtures: &TestFixtures) -> Result<()> {
    let cli = CliRunner::new(fixtures.cli_path());
    
    println!("    🌌 Testing Cosmos analyze-contract command...");
    test_analyze_contract(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos compile-layout command...");
    test_compile_layout(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos generate-queries command...");
    test_generate_queries(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos resolve-query command...");
    test_resolve_query(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos auto-generate command...");
    test_auto_generate(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos output formats...");
    test_output_formats(&cli, fixtures).await?;
    
    println!("    🌌 Testing Cosmos performance...");
    test_performance(&cli, fixtures).await?;
    
    Ok(())
}

/// Test `traverse cosmos analyze-contract` command
async fn test_analyze_contract(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let output_path = fixtures.path("outputs/cw20_analysis.json");
    
    // Test basic analysis
    let output = cli.run_success(&[
        "cosmos", "analyze-contract",
        cw20_schema.to_str().unwrap(),
        "--output", output_path.to_str().unwrap(),
    ])?;
    
    // Verify output contains expected information
    assertions::assert_output_contains(&output, "Analyzing CosmWasm contract")?;
    
    // Verify output file was created and has expected structure
    let _analysis = FileValidator::validate_json_structure(&output_path, &[
        "contract_type", "messages", "storage_patterns"
    ])?;
    
    assertions::assert_output_contains(&output, "Contract analysis complete")?;
    
    // Test with validation flag
    let output = cli.run_success(&[
        "cosmos", "analyze-contract", 
        cw20_schema.to_str().unwrap(),
        "--validate-schema"
    ])?;
    
    assertions::assert_output_contains(&output, "Schema validation")?;
    
    // Test with CW721 NFT contract
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();
    let cw721_output_path = fixtures.path("outputs/cw721_analysis.json");
    
    let output = cli.run_success(&[
        "cosmos", "analyze-contract",
        cw721_schema.to_str().unwrap(),
        "--output", cw721_output_path.to_str().unwrap(),
        "--validate-schema"
    ])?;
    
    assertions::assert_output_contains(&output, "token_info")?;
    FileValidator::validate_json(&cw721_output_path)?;
    
    println!("      ✅ analyze-contract tests passed");
    Ok(())
}

/// Test `traverse cosmos compile-layout` command
async fn test_compile_layout(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let layout_path = fixtures.path("outputs/cosmos_layout.json");
    
    let output = cli.run_success(&[
        "cosmos", "compile-layout",
        cw20_schema.to_str().unwrap(),
        "--output", layout_path.to_str().unwrap(),
        "--format", "traverse"
    ])?;
    
    assertions::assert_output_contains(&output, "Storage layout compiled")?;
    FileValidator::validate_json(&layout_path)?;
    
    // Verify layout file structure
    let _layout = FileValidator::validate_json_structure(&layout_path, &[
        "contract_name", "storage_layout", "commitment"
    ])?;
    
    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary"] {
        let format_output_path = fixtures.path(&format!("outputs/cw20_layout.{}", format));
        
        cli.run_success(&[
            "cosmos", "compile-layout",
            cw20_schema.to_str().unwrap(),
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
    
    // Test with CW721 NFT contract
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();
    let cw721_layout_path = fixtures.path("outputs/cw721_layout.json");
    
    cli.run_success(&[
        "cosmos", "compile-layout",
        cw721_schema.to_str().unwrap(),
        "--output", cw721_layout_path.to_str().unwrap()
    ])?;
    
    FileValidator::validate_json(&cw721_layout_path)?;
    
    println!("      ✅ compile-layout tests passed");
    Ok(())
}

/// Test `traverse cosmos generate-queries` command
async fn test_generate_queries(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // First compile a layout
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let layout_path = fixtures.path("outputs/cw20_layout_for_queries.json");
    
    cli.run_success(&[
        "cosmos", "compile-layout",
        cw20_schema.to_str().unwrap(),
        "--output", layout_path.to_str().unwrap()
    ])?;
    
    // Test query generation
    let queries_path = fixtures.path("outputs/cw20_queries.json");
    
    let output = cli.run_success(&[
        "cosmos", "generate-queries",
        layout_path.to_str().unwrap(),
        "--state-keys", "token_info,balance,all_accounts",
        "--output", queries_path.to_str().unwrap()
    ])?;
    
    assertions::assert_output_contains(&output, "Generated")?;
    
    // Verify queries file structure
    let _queries = FileValidator::validate_json_structure(&queries_path, &[
        "queries", "metadata"
    ])?;
    
    // Test with different field types
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();
    let nft_queries_path = fixtures.path("outputs/cosmos_nft_queries.json");
    
    cli.run_success(&[
        "cosmos", "generate-queries",
        cw721_schema.to_str().unwrap(),
        "--fields", "token_info,num_tokens,contract_info",
        "--output", nft_queries_path.to_str().unwrap(),
        "--format", "coprocessor-json"
    ])?;
    
    FileValidator::validate_json(&nft_queries_path)?;
    
    println!("      ✅ generate-queries tests passed");
    Ok(())
}

/// Test `traverse cosmos resolve-query` command
async fn test_resolve_query(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // First create a layout to work with
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let layout_path = fixtures.path("outputs/cosmos_resolve_layout.json");
    
    cli.run_success(&[
        "cosmos", "compile-layout",
        cw20_schema.to_str().unwrap(),
        "--output", layout_path.to_str().unwrap(),
        "--format", "traverse"
    ])?;
    
    // Test basic query resolution
    let resolved_path = fixtures.path("outputs/cosmos_resolved.json");
    
    let output = cli.run_success(&[
        "cosmos", "resolve-query",
        "balances.cosmos1zxj6y5h3r8k9v7n2m4l1q8w5e3t6y9u0i7o4p2s5d8f6g3h1j4k7l9n2",
        "--layout", layout_path.to_str().unwrap(),
        "--output", resolved_path.to_str().unwrap(),
        "--format", "coprocessor-json"
    ])?;
    
    assertions::assert_output_contains(&output, "Resolved")?;
    FileValidator::validate_json(&resolved_path)?;
    
    // Verify resolved query structure
    let _resolved = FileValidator::validate_json_structure(&resolved_path, &[
        "storage_key", "namespace"
    ])?;
    
    // Test different output formats
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let format_resolved_path = fixtures.path(&format!("outputs/cosmos_resolved.{}", format));
        
        cli.run_success(&[
            "cosmos", "resolve-query",
            "balances.cosmos1zxj6y5h3r8k9v7n2m4l1q8w5e3t6y9u0i7o4p2s5d8f6g3h1j4k7l9n2",
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

/// Test `traverse cosmos auto-generate` command
async fn test_auto_generate(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let output_dir = fixtures.path("outputs/cw20_auto");
    
    // Test dry-run mode
    let output = cli.run_success(&[
        "cosmos", "auto-generate",
        cw20_schema.to_str().unwrap(),
        "--rpc", "https://rpc.osmosis.zone:443",
        "--contract", "cosmos1contract123",
        "--queries", "token_info,balance.cosmos1abc123",
        "--output-dir", output_dir.to_str().unwrap(),
        "--dry-run"
    ])?;
    
    assertions::assert_output_contains(&output, "End-to-end automation")?;
    assertions::assert_output_contains(&output, "Dry run mode")?;
    
    // Verify output directory structure in dry-run
    FileValidator::validate_directory_structure(&output_dir, &[
        "layout.json", "queries.json", "resolved_queries.json"
    ])?;
    
    // Test with CW721 NFT contract
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();
    let cw721_output_dir = fixtures.path("outputs/cw721_auto");
    
    let output = cli.run_success(&[
        "cosmos", "auto-generate",
        cw721_schema.to_str().unwrap(),
        "--rpc", "https://rpc.neutron.org",
        "--contract", "neutron1contract456",
        "--queries", "config,vault_info,withdraw_requests.neutron1user789",
        "--output-dir", cw721_output_dir.to_str().unwrap(),
        "--dry-run"
    ])?;
    
    assertions::assert_output_contains(&output, "CosmWasm")?;
    FileValidator::validate_directory_structure(&cw721_output_dir, &[
        "layout.json", "queries.json", "resolved_queries.json"
    ])?;
    
    println!("      ✅ auto-generate tests passed");
    Ok(())
}

/// Test different output formats work correctly
async fn test_output_formats(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    
    // Test each format with compile-layout
    for format in &["traverse", "coprocessor-json", "toml", "binary", "base64"] {
        let output_path = fixtures.path(&format!("outputs/cosmos_format_test.{}", format));
        
        cli.run_success(&[
            "cosmos", "compile-layout",
            cw20_schema.to_str().unwrap(),
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

/// Test performance of Cosmos commands
async fn test_performance(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let output_path = fixtures.path("outputs/cosmos_perf_test.json");
    
    // Test compile-layout performance (should complete in reasonable time)
    PerformanceTester::test_performance_threshold(
        cli,
        &[
            "cosmos", "compile-layout",
            cw20_schema.to_str().unwrap(),
            "--output", output_path.to_str().unwrap()
        ],
        std::time::Duration::from_secs(30) // 30 second threshold
    )?;
    
    // Test analyze-contract performance
    PerformanceTester::test_performance_threshold(
        cli,
        &[
            "cosmos", "analyze-contract",
            cw20_schema.to_str().unwrap()
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
    async fn test_cosmos_commands_structure() {
        // Test that we can create fixtures and runner
        let fixtures = TestFixtures::new().await.unwrap();
        let cli = CliRunner::new(fixtures.cli_path());
        
        // Test help command works
        let output = cli.run(&["cosmos", "--help"]);
        assert!(output.is_ok());
    }
} 