//! File I/O module - file system operations
//!
//! This module handles reading and writing text files with atomic writes
//! and proper error handling.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::Path;
//! use termide::file_io::{read_file, write_file};
//!
//! # fn main() -> Result<(), anyhow::Error> {
//! let path = Path::new("example.txt");
//!
//! // Read a file
//! let content = read_file(path)?;
//!
//! // Write content to a file
//! write_file(path, "Hello, world!")?;
//! # Ok(())
//! # }
//! ```

mod read;
mod write;

#[cfg(test)]
mod tests;

pub use read::read_file;
pub use write::write_file;
