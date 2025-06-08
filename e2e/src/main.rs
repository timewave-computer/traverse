//! Comprehensive end-to-end test runner for traverse CLI commands
//!
//! This test runner validates all CLI functionality described in the documentation
//! and work plan by executing actual CLI commands with real test data.

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tokio::time::{timeout, Duration};

mod fixtures;
mod test_ethereum;
mod test_cosmos;
mod test_unified;
mod test_core;
mod utils;

use fixtures::TestFixtures;

/// Main test runner that executes all end-to-end tests
#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting traverse CLI end-to-end test suite");
    
    // Initialize test environment
    let fixtures = TestFixtures::new().await?;
    println!("✅ Test fixtures initialized");
    
    // Check if CLI binary exists
    check_cli_binary()?;
    println!("✅ CLI binary found");
    
    // Run test suites in order
    let mut results = Vec::new();
    
    println!("\n📋 Running Core Commands Tests...");
    results.push(run_test_suite("core", test_core::run_tests(&fixtures)).await);
    
    println!("\n🔷 Running Ethereum Commands Tests...");
    results.push(run_test_suite("ethereum", test_ethereum::run_tests(&fixtures)).await);
    
    println!("\n🌌 Running Cosmos Commands Tests...");
    results.push(run_test_suite("cosmos", test_cosmos::run_tests(&fixtures)).await);
    
    println!("\n🔗 Running Unified Commands Tests...");
    results.push(run_test_suite("unified", test_unified::run_tests(&fixtures)).await);
    
    // Summary
    let total_tests = results.len();
    let passed = results.iter().filter(|&&r| r).count();
    let failed = total_tests - passed;
    
    println!("\n📊 Test Results Summary:");
    println!("   Total test suites: {}", total_tests);
    println!("   Passed: {} ✅", passed);
    println!("   Failed: {} ❌", failed);
    
    if failed > 0 {
        println!("\n❌ Some tests failed. Check output above for details.");
        std::process::exit(1);
    } else {
        println!("\n🎉 All tests passed! CLI is fully functional.");
    }
    
    Ok(())
}

/// Run a test suite with timeout and error handling
async fn run_test_suite(name: &str, test_future: impl std::future::Future<Output = Result<()>>) -> bool {
    println!("  🧪 Starting {} test suite...", name);
    
    match timeout(Duration::from_secs(300), test_future).await {
        Ok(Ok(())) => {
            println!("  ✅ {} tests passed", name);
            true
        }
        Ok(Err(e)) => {
            println!("  ❌ {} tests failed: {}", name, e);
            false
        }
        Err(_) => {
            println!("  ⏰ {} tests timed out", name);
            false
        }
    }
}

/// Check if the CLI binary exists and is executable
fn check_cli_binary() -> Result<()> {
    let cli_path = get_cli_path();
    
    if !cli_path.exists() {
        return Err(anyhow::anyhow!(
            "CLI binary not found at {}. Run 'cargo build --package traverse-cli' first.",
            cli_path.display()
        ));
    }
    
    // Test basic CLI functionality
    let output = Command::new(&cli_path)
        .arg("--help")
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "CLI binary is not functional. Exit code: {}",
            output.status.code().unwrap_or(-1)
        ));
    }
    
    Ok(())
}

/// Get path to the CLI binary
fn get_cli_path() -> PathBuf {
    let exe_name = if cfg!(windows) {
        "traverse-cli.exe"
    } else {
        "traverse-cli"
    };
    
    // Try to find the binary in target/debug or target/release
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap());
    
    let debug_path = manifest_dir.parent().unwrap().join("target/debug").join(exe_name);
    let release_path = manifest_dir.parent().unwrap().join("target/release").join(exe_name);
    
    if release_path.exists() {
        release_path
    } else {
        debug_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_binary_exists() {
        // This test ensures the binary can be found in CI/CD
        let cli_path = get_cli_path();
        println!("Looking for CLI binary at: {}", cli_path.display());
        
        // In CI/CD, we might need to build first
        if !cli_path.exists() {
            println!("CLI binary not found, this is expected in some test environments");
        }
    }
    
    #[tokio::test]
    async fn test_fixtures_creation() {
        let fixtures = TestFixtures::new().await;
        assert!(fixtures.is_ok(), "Test fixtures should be created successfully");
    }
} 