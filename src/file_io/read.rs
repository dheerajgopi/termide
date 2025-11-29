//! File reading functionality

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Reads the contents of a file and returns it as a UTF-8 encoded string.
///
/// This function enforces UTF-8 encoding and provides user-friendly error messages
/// for common file system errors.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// * `Ok(String)` - File contents as a UTF-8 string
/// * `Err(anyhow::Error)` - User-friendly error if reading fails
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist
/// - Permission is denied
/// - The file contains invalid UTF-8
/// - An I/O error occurs
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use termide::file_io::read_file;
///
/// # fn main() -> Result<(), anyhow::Error> {
/// let content = read_file(Path::new("example.txt"))?;
/// println!("File content: {}", content);
/// # Ok(())
/// # }
/// ```
pub fn read_file(path: &Path) -> Result<String> {
    // Read the file to bytes first
    let bytes = fs::read(path).context(format!(
        "Failed to read file '{}'. Check that the file exists and you have permission to read it.",
        path.display()
    ))?;

    // Convert to UTF-8 string
    String::from_utf8(bytes).context(format!(
        "File '{}' contains invalid UTF-8. Only UTF-8 encoded text files are supported.",
        path.display()
    ))
}
