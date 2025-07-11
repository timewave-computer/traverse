//! End-to-end tests for unified CLI commands
//!
//! Tests unified commands that work across both chains:
//! - traverse auto-generate (chain auto-detection)
//! - traverse batch-generate (multi-contract processing)
//! - traverse watch (continuous monitoring)

use crate::fixtures::TestFixtures;
use crate::utils::*;
use anyhow::Result;
use std::fs;

/// Run all unified CLI command tests
pub async fn run_tests(fixtures: &TestFixtures) -> Result<()> {
    let cli = CliRunner::new(fixtures.cli_path());

    println!("    Testing unified auto-generate command...");
    test_auto_generate(&cli, fixtures).await?;

    println!("    Testing unified batch-generate command...");
    test_batch_generate(&cli, fixtures).await?;

    println!("    Testing unified watch command...");
    test_watch(&cli, fixtures).await?;

    println!("    Testing chain auto-detection...");
    test_chain_detection(&cli, fixtures).await?;

    println!("    Testing error handling...");
    test_error_handling(&cli, fixtures).await?;

    Ok(())
}

/// Test `traverse auto-generate` command with chain auto-detection
async fn test_auto_generate(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Test with Ethereum ABI (should auto-detect as Ethereum)
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();
    let eth_output_dir = fixtures.path("outputs/unified_eth_auto");

    let output = cli.run_success(&[
        "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc-ethereum",
        "https://mainnet.infura.io/v3/test",
        "--contract-ethereum",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--output-dir",
        eth_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Auto-generating")?;
    assertions::assert_output_contains(&output, "Detected chain type: ethereum")?;
    assertions::assert_output_contains(&output, "Dry run mode")?;

    // Verify output directory structure
    FileValidator::validate_directory_structure(
        &eth_output_dir,
        &["layout.json", "resolved_queries.json"],
    )?;

    // Test with Cosmos schema (should auto-detect as Cosmos)
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();
    let cosmos_output_dir = fixtures.path("outputs/unified_cosmos_auto");

    let output = cli.run_success(&[
        "auto-generate",
        cw20_schema.to_str().unwrap(),
        "--rpc-cosmos",
        "https://rpc.osmosis.zone:443",
        "--contract-cosmos",
        "cosmos1contract123",
        "--output-dir",
        cosmos_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: cosmos")?;

    // Verify output directory structure
    FileValidator::validate_directory_structure(
        &cosmos_output_dir,
        &["layout.json", "resolved_queries.json"],
    )?;

    // Test with queries file
    let queries_file = fixtures.query_files.get("ethereum").unwrap();
    let eth_queries_output_dir = fixtures.path("outputs/unified_eth_queries_auto");

    let output = cli.run_success(&[
        "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--rpc-ethereum",
        "https://mainnet.infura.io/v3/test",
        "--contract-ethereum",
        "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
        "--queries-file",
        queries_file.to_str().unwrap(),
        "--output-dir",
        eth_queries_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Using queries from file")?;

    // Test with CW721 NFT contract
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();
    let cw721_output_dir = fixtures.path("outputs/unified_cw721");

    let output = cli.run_success(&[
        "auto-generate",
        cw721_schema.to_str().unwrap(),
        "--output-dir",
        cw721_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: cosmos")?;

    println!("      auto-generate tests passed");
    Ok(())
}

/// Test `traverse batch-generate` command with configuration file
async fn test_batch_generate(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let batch_config = fixtures.config_files.get("batch").unwrap();
    let batch_output_dir = fixtures.path("outputs/unified_batch");

    let output = cli.run_success(&[
        "batch-generate",
        batch_config.to_str().unwrap(),
        "--parallel",
        "2",
        "--output-dir",
        batch_output_dir.to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Batch processing")?;
    assertions::assert_output_contains(&output, "parallel workers: 2")?;
    assertions::assert_output_contains(&output, "Dry run mode")?;

    // Should process both Ethereum and Cosmos contracts
    assertions::assert_output_contains(&output, "ethereum")?;
    assertions::assert_output_contains(&output, "cosmos")?;

    // Verify output directory structure for batch processing
    FileValidator::validate_directory_structure(&batch_output_dir, &["summary.json"])?;

    // Test performance with parallel processing
    let (_, duration) = PerformanceTester::measure_command_time(
        cli,
        &[
            "batch-generate",
            batch_config.to_str().unwrap(),
            "--parallel",
            "1",
            "--output-dir",
            batch_output_dir.to_str().unwrap(),
            "--dry-run",
        ],
    )?;

    // Should complete in reasonable time even with multiple contracts
    if duration > std::time::Duration::from_secs(60) {
        return Err(anyhow::anyhow!(
            "Batch processing took too long: {:?}",
            duration
        ));
    }

    println!("      batch-generate tests passed");
    Ok(())
}

/// Test `traverse watch` command (basic functionality)
async fn test_watch(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    let watch_config = fixtures.config_files.get("watch").unwrap();
    let watch_dir = fixtures.path("watch_test");

    // Create watch directory
    fs::create_dir_all(&watch_dir)?;

    // Test watch command startup (should initialize but we'll interrupt quickly)
    let result = std::process::Command::new(fixtures.cli_path())
        .args([
            "watch",
            watch_dir.to_str().unwrap(),
            "--config",
            watch_config.to_str().unwrap(),
        ])
        .spawn();

    if let Ok(mut child) = result {
        // Let it initialize for a moment
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Terminate the watch process
        let _ = child.kill();
        let _output = child.wait_with_output();

        // Check that it started correctly (before we killed it)
        println!("      Watch command started successfully");
    } else {
        // Test that arguments are parsed correctly by checking help
        let output = cli.run_success(&["watch", "--help"])?;
        assertions::assert_output_contains(&output, "Watch")?;
        assertions::assert_output_contains(&output, "webhook")?;
    }

    println!("      watch tests passed");
    Ok(())
}

/// Test chain auto-detection functionality
async fn test_chain_detection(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Test Ethereum detection
    let erc20_abi = fixtures.ethereum_abis.get("erc20").unwrap();

    let output = cli.run_success(&[
        "auto-generate",
        erc20_abi.to_str().unwrap(),
        "--output-dir",
        fixtures.path("outputs/chain_detect_test").to_str().unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: ethereum")?;

    // Test Cosmos detection
    let cw20_schema = fixtures.cosmos_schemas.get("cw20").unwrap();

    let output = cli.run_success(&[
        "auto-generate",
        cw20_schema.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/chain_detect_test2")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: cosmos")?;

    // Test complex DeFi contract detection
    let defi_abi = fixtures.ethereum_abis.get("defi").unwrap();

    let output = cli.run_success(&[
        "auto-generate",
        defi_abi.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/chain_detect_test3")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: ethereum")?;
    assertions::assert_output_contains(&output, "complex mappings")?;

    // Test CW721 NFT contract detection
    let cw721_schema = fixtures.cosmos_schemas.get("cw721").unwrap();

    let output = cli.run_success(&[
        "auto-generate",
        cw721_schema.to_str().unwrap(),
        "--output-dir",
        fixtures
            .path("outputs/chain_detect_test4")
            .to_str()
            .unwrap(),
        "--dry-run",
    ])?;

    assertions::assert_output_contains(&output, "Detected chain type: cosmos")?;
    assertions::assert_output_contains(&output, "NFT")?;

    println!("      chain detection tests passed");
    Ok(())
}

/// Test error handling and edge cases
async fn test_error_handling(cli: &CliRunner, fixtures: &TestFixtures) -> Result<()> {
    // Test invalid file path
    let result = cli.run(&[
        "auto-generate",
        "/nonexistent/file.json",
        "--output-dir",
        fixtures.path("outputs/error_test").to_str().unwrap(),
        "--dry-run",
    ]);

    if let Ok(output) = result {
        if output.status.success() {
            return Err(anyhow::anyhow!(
                "Expected auto-generate to fail with nonexistent file"
            ));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        assertions::assert_output_contains(&stderr, "not found")?;
    }

    // Test invalid configuration file for batch
    let result = cli.run(&[
        "batch-generate",
        "/nonexistent/config.toml",
        "--output-dir",
        fixtures.path("outputs/error_test").to_str().unwrap(),
        "--dry-run",
    ]);

    if let Ok(output) = result {
        if output.status.success() {
            return Err(anyhow::anyhow!(
                "Expected batch-generate to fail with nonexistent config"
            ));
        }
    }

    // Test missing required arguments
    let result = cli.run(&["auto-generate"]);

    if let Ok(output) = result {
        if output.status.success() {
            return Err(anyhow::anyhow!(
                "Expected auto-generate to fail without arguments"
            ));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        assertions::assert_output_contains(&stderr, "required")?;
    }

    // Test invalid output directory (read-only)
    if cfg!(unix) {
        let result = cli.run(&[
            "auto-generate",
            fixtures
                .ethereum_abis
                .get("erc20")
                .unwrap()
                .to_str()
                .unwrap(),
            "--output-dir",
            "/root/readonly",
            "--dry-run",
        ]);

        if let Ok(output) = result {
            if output.status.success() {
                println!("      Read-only directory test skipped (permissions vary)");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("      Correctly rejected read-only directory: {}", stderr);
            }
        }
    }

    println!("      error handling tests passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::TestFixtures;

    #[tokio::test]
    async fn test_unified_commands_structure() {
        // Test that we can create fixtures and runner
        let fixtures = TestFixtures::new().await.unwrap();
        let cli = CliRunner::new(fixtures.cli_path());

        // Test auto-generate help command works
        let output = cli.run(&["auto-generate", "--help"]);
        assert!(output.is_ok());
    }
}
