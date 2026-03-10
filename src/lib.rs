//! Coral - Proto dependency visualizer for gRPC/Connect projects.

pub mod analyzer;
pub mod decoder;
pub mod diff;
pub mod domain;
pub mod error;
pub mod reporter;
pub mod server;

pub use analyzer::Analyzer;
pub use diff::DiffReport;
pub use domain::{Edge, GraphModel, Node, NodeDetails, NodeType, Package};
pub use error::{CoralError, Result};
pub use reporter::MarkdownReporter;
pub use server::serve;

use prost_types::FileDescriptorSet;
use std::io::Read;

const STDIN_BUFFER_CAPACITY: usize = 64 * 1024;
const MAX_STDIN_BYTES: usize = 256 * 1024 * 1024; // 256 MiB

pub fn read_stdin() -> Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(STDIN_BUFFER_CAPACITY);
    std::io::stdin()
        .take(MAX_STDIN_BYTES as u64 + 1)
        .read_to_end(&mut buffer)?;
    if buffer.len() > MAX_STDIN_BYTES {
        return Err(CoralError::InputTooLarge {
            max_bytes: MAX_STDIN_BYTES,
        });
    }
    Ok(buffer)
}

pub fn debug_output(fds: &FileDescriptorSet) {
    println!("=== FileDescriptorSet Debug ===");
    println!("Total files: {}", fds.file.len());
    println!();

    for file in &fds.file {
        let name = file.name.as_deref().unwrap_or("<unknown>");
        let package = file.package.as_deref().unwrap_or("<unknown>");
        let msg = file.message_type.len();
        let srv = file.service.len();
        println!("📄 File: {name}");
        println!("   Package: {package}");
        println!("   Messages: {msg}");
        println!("   Services: {srv}");
        println!();
    }
}
