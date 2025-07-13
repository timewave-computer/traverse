//! End-to-end test suite for the traverse CLI
//!
//! This module runs comprehensive integration tests against the traverse CLI
//! to ensure all commands work correctly in real-world scenarios.

use anyhow::Result;
use std::process::Command;

mod fixtures;
mod test_core;
mod test_cosmos;
mod test_ethereum;
mod test_solana;
mod test_unified;
mod utils;

use fixtures::TestFixtures;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting traverse CLI end-to-end test suite");
    println!("==========================================");

    // Initialize test fixtures
    let fixtures = TestFixtures::new().await?;
    println!("Test fixtures initialized");

    // Check if CLI binary exists
    check_cli_binary(&fixtures)?;
    println!("CLI binary found");

    // Run all test suites
    println!("\nRunning Core Commands Tests...");
    let mut passed = 0;
    let mut failed = 0;

    // Test core commands
    run_test_suite(
        "Core Commands",
        test_core::run_tests(&fixtures),
        &mut passed,
        &mut failed,
    )
    .await;

    // Test Ethereum commands
    run_test_suite(
        "Ethereum Commands",
        test_ethereum::run_tests(&fixtures),
        &mut passed,
        &mut failed,
    )
    .await;

    // Test Cosmos commands
    run_test_suite(
        "Cosmos Commands",
        test_cosmos::run_tests(&fixtures),
        &mut passed,
        &mut failed,
    )
    .await;

    // Test Solana commands (if Solana tools are available)
    run_test_suite(
        "Solana Commands",
        test_solana::run_tests(&fixtures),
        &mut passed,
        &mut failed,
    )
    .await;

    // Test unified commands
    run_test_suite(
        "Unified Commands",
        test_unified::run_tests(&fixtures),
        &mut passed,
        &mut failed,
    )
    .await;

    // Print summary
    println!("\nTest Results Summary:");
    println!("====================");
    println!("   Passed: {}", passed);
    println!("   Failed: {}", failed);

    if failed > 0 {
        println!("\nSome tests failed. Check output above for details.");
        std::process::exit(1);
    } else {
        println!("\nAll tests passed! CLI is fully functional.");
        Ok(())
    }
}

async fn run_test_suite<F>(name: &str, test_future: F, passed: &mut u32, failed: &mut u32)
where
    F: std::future::Future<Output = Result<()>>,
{
    match test_future.await {
        Ok(()) => {
            println!("  {} tests passed", name);
            *passed += 1;
        }
        Err(e) => {
            println!("  {} tests failed: {}", name, e);
            *failed += 1;
        }
    }
}

/// Check if the CLI binary exists and is executable
fn check_cli_binary(fixtures: &TestFixtures) -> Result<()> {
    let cli_path = fixtures.cli_path();

    if !cli_path.exists() {
        return Err(anyhow::anyhow!(
            "CLI binary not found at {}. Run 'cargo build --package traverse-cli' first.",
            cli_path.display()
        ));
    }

    // Test basic CLI functionality
    let output = Command::new(&cli_path).arg("--help").output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "CLI binary is not functional. Exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli_binary_exists() {
        // This test ensures the binary can be found in CI/CD
        let fixtures = TestFixtures::new()
            .await
            .expect("Failed to create fixtures");
        let cli_path = fixtures.cli_path();
        println!("Looking for CLI binary at: {}", cli_path.display());

        // In CI/CD, we might need to build first
        if !cli_path.exists() {
            println!("CLI binary not found, this is expected in some test environments");
        }
    }

    #[tokio::test]
    async fn test_fixtures_creation() {
        let fixtures = TestFixtures::new().await;
        assert!(
            fixtures.is_ok(),
            "Test fixtures should be created successfully"
        );
    }
}
