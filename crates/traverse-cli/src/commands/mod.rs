//! Command implementations for the traverse CLI tool
//!
//! This module contains the implementation of all CLI commands, organized by functionality.

pub mod cosmos;
pub mod ethereum;
pub mod layout;
pub mod proof;
pub mod resolve;
pub mod unified;
pub mod codegen;
pub mod minimal;

pub use layout::cmd_compile_layout;
pub use proof::cmd_generate_proof;
pub use resolve::{cmd_batch_resolve, cmd_resolve, cmd_resolve_all};
pub use unified::{
    cmd_unified_auto_generate as cmd_auto_generate,
    cmd_unified_batch_generate as cmd_batch_generate, cmd_unified_watch as cmd_watch,
};
pub use codegen::{cmd_codegen, CodegenCommands};
pub use minimal::cmd_generate_minimal;
