//! Utility functions for end-to-end testing
//!
//! This module provides helper functions for running CLI commands,
//! validating outputs, and common test operations.

use anyhow::Result;
use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Output;

/// CLI command runner with validation capabilities
pub struct CliRunner {
    binary_path: std::path::PathBuf,
}

impl CliRunner {
    /// Create a new CLI runner
    pub fn new(binary_path: std::path::PathBuf) -> Self {
        Self { binary_path }
    }

    /// Run a CLI command and return the output
    pub fn run(&self, args: &[&str]) -> Result<Output> {
        let output = std::process::Command::new(&self.binary_path)
            .args(args)
            .output()?;
        Ok(output)
    }

    /// Run a CLI command using assert_cmd for better testing
    #[allow(dead_code)]
    pub fn cmd(&self) -> Command {
        Command::new(&self.binary_path)
    }

    /// Run command and expect success
    pub fn run_success(&self, args: &[&str]) -> Result<String> {
        let output = self.run(args)?;
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Command failed with exit code {}: {}",
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    /// Run command and expect failure
    pub fn run_failure(&self, args: &[&str]) -> Result<String> {
        let output = self.run(args)?;
        if output.status.success() {
            return Err(anyhow::anyhow!(
                "Command unexpectedly succeeded when failure was expected"
            ));
        }
        Ok(String::from_utf8(output.stderr)?)
    }

    /// Run command and expect specific exit code
    #[allow(dead_code)]
    pub fn run_with_exit_code(
        &self,
        args: &[&str],
        expected_code: i32,
    ) -> Result<(String, String)> {
        let output = self.run(args)?;
        let actual_code = output.status.code().unwrap_or(-1);
        if actual_code != expected_code {
            return Err(anyhow::anyhow!(
                "Expected exit code {}, got {}. Stderr: {}",
                expected_code,
                actual_code,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok((
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?,
        ))
    }
}

/// File validation utilities
pub struct FileValidator;

impl FileValidator {
    /// Check if file exists and is not empty
    pub fn exists_and_non_empty(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(anyhow::anyhow!("File is empty: {}", path.display()));
        }

        Ok(())
    }

    /// Validate JSON file and return parsed content
    pub fn validate_json(path: &Path) -> Result<Value> {
        Self::exists_and_non_empty(path)?;
        let content = fs::read_to_string(path)?;
        let json: Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid JSON in {}: {}", path.display(), e))?;
        Ok(json)
    }

    /// Validate that JSON file has expected structure
    pub fn validate_json_structure(path: &Path, expected_keys: &[&str]) -> Result<Value> {
        let json = Self::validate_json(path)?;

        if let Some(obj) = json.as_object() {
            for key in expected_keys {
                if !obj.contains_key(*key) {
                    return Err(anyhow::anyhow!(
                        "JSON file {} missing expected key: {}",
                        path.display(),
                        key
                    ));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "JSON file {} is not an object",
                path.display()
            ));
        }

        Ok(json)
    }

    /// Validate TOML file
    pub fn validate_toml(path: &Path) -> Result<toml::Value> {
        Self::exists_and_non_empty(path)?;
        let content = fs::read_to_string(path)?;
        let toml: toml::Value = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid TOML in {}: {}", path.display(), e))?;
        Ok(toml)
    }

    /// Check if directory exists and has expected files
    pub fn validate_directory_structure(dir: &Path, expected_files: &[&str]) -> Result<()> {
        if !dir.exists() {
            return Err(anyhow::anyhow!(
                "Directory does not exist: {}",
                dir.display()
            ));
        }

        for file in expected_files {
            let file_path = dir.join(file);
            if !file_path.exists() {
                return Err(anyhow::anyhow!(
                    "Expected file not found: {}",
                    file_path.display()
                ));
            }
        }

        Ok(())
    }
}

/// Output format validation
pub struct OutputValidator;

impl OutputValidator {
    /// Validate traverse native format
    pub fn validate_traverse_format(content: &str) -> Result<()> {
        // Basic validation for traverse format
        if content.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty traverse format output"));
        }

        // Check for expected patterns in traverse output
        if !content.contains("storage_key") && !content.contains("slot") {
            return Err(anyhow::anyhow!(
                "Traverse format missing storage key information"
            ));
        }

        Ok(())
    }

    /// Validate coprocessor JSON format
    pub fn validate_coprocessor_format(json: &Value) -> Result<()> {
        // For now, just check it's valid JSON with some expected structure
        if let Some(obj) = json.as_object() {
            // Basic validation - should have some storage-related fields
            if !obj
                .keys()
                .any(|k| k.contains("storage") || k.contains("layout") || k.contains("query"))
            {
                return Err(anyhow::anyhow!(
                    "Coprocessor format missing storage-related fields"
                ));
            }
        } else {
            return Err(anyhow::anyhow!("Coprocessor format is not a JSON object"));
        }

        Ok(())
    }

    /// Validate binary format (check for binary data)
    pub fn validate_binary_format(data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(anyhow::anyhow!("Empty binary output"));
        }

        // Basic check - binary data should have some non-printable characters or be reasonably long
        if data.len() < 4 {
            return Err(anyhow::anyhow!("Binary data too short"));
        }

        Ok(())
    }

    /// Validate base64 format
    pub fn validate_base64_format(content: &str) -> Result<Vec<u8>> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let content = content.trim();
        if content.is_empty() {
            return Err(anyhow::anyhow!("Empty base64 output"));
        }

        let decoded = STANDARD
            .decode(content)
            .map_err(|e| anyhow::anyhow!("Invalid base64 format: {}", e))?;

        if decoded.is_empty() {
            return Err(anyhow::anyhow!("Base64 decoded to empty data"));
        }

        Ok(decoded)
    }

    /// Validate semantic proof structure
    pub fn validate_semantic_proof_structure(json: &Value, expected_keys: &[&str]) -> Result<()> {
        if let Some(obj) = json.as_object() {
            for key in expected_keys {
                if !obj.contains_key(*key) {
                    return Err(anyhow::anyhow!(
                        "Semantic proof missing expected key: {}",
                        key
                    ));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Semantic proof is not a JSON object"));
        }
        Ok(())
    }
}

/// Performance testing utilities
pub struct PerformanceTester;

impl PerformanceTester {
    /// Measure command execution time
    pub fn measure_command_time(
        cli: &CliRunner,
        args: &[&str],
    ) -> Result<(String, std::time::Duration)> {
        let start = std::time::Instant::now();
        let output = cli.run_success(args)?;
        let duration = start.elapsed();
        Ok((output, duration))
    }

    /// Test command performance with threshold
    pub fn test_performance_threshold(
        cli: &CliRunner,
        args: &[&str],
        max_duration: std::time::Duration,
    ) -> Result<()> {
        let (_, duration) = Self::measure_command_time(cli, args)?;

        if duration > max_duration {
            return Err(anyhow::anyhow!(
                "Command took too long: {:?} > {:?}",
                duration,
                max_duration
            ));
        }

        Ok(())
    }
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert command succeeds and return output
    #[allow(dead_code)]
    pub fn assert_success(cli: &CliRunner, args: &[&str]) -> Result<String> {
        cli.run_success(args)
    }

    /// Assert command fails with specific exit code
    #[allow(dead_code)]
    pub fn assert_failure(cli: &CliRunner, args: &[&str], expected_code: i32) -> Result<String> {
        let (_, stderr) = cli.run_with_exit_code(args, expected_code)?;
        Ok(stderr)
    }

    /// Assert output contains expected string
    pub fn assert_output_contains(output: &str, expected: &str) -> Result<()> {
        if !output.contains(expected) {
            return Err(anyhow::anyhow!(
                "Output does not contain expected string '{}'. Actual output: {}",
                expected,
                output
            ));
        }
        Ok(())
    }

    /// Assert file was created with specific content
    #[allow(dead_code)]
    pub fn assert_file_created_with_content(path: &Path, expected_content: &str) -> Result<()> {
        FileValidator::exists_and_non_empty(path)?;
        let content = fs::read_to_string(path)?;
        if !content.contains(expected_content) {
            return Err(anyhow::anyhow!(
                "File {} does not contain expected content '{}'",
                path.display(),
                expected_content
            ));
        }
        Ok(())
    }

    /// Assert JSON value contains expected key
    pub fn assert_json_contains(json: &Value, expected_key: &str) -> Result<()> {
        if let Some(obj) = json.as_object() {
            if !obj.contains_key(expected_key) {
                return Err(anyhow::anyhow!(
                    "JSON does not contain expected key: {}",
                    expected_key
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "JSON value is not an object, cannot check for key: {}",
                expected_key
            ));
        }
        Ok(())
    }
}
