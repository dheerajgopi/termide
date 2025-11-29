//! File writing functionality with atomic operations

use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Writes content to a file using an atomic write strategy.
///
/// This function uses a temporary file and rename strategy to ensure atomic writes,
/// preventing data corruption if the write operation fails. File permissions are
/// preserved when overwriting existing files.
///
/// The atomic write strategy:
/// 1. Write content to a temporary file in the same directory
/// 2. If write succeeds, rename the temporary file to the target path
/// 3. The rename operation is atomic on POSIX systems
///
/// # Arguments
///
/// * `path` - Path to the file to write
/// * `content` - Content to write to the file (UTF-8)
///
/// # Returns
///
/// * `Ok(())` - File written successfully
/// * `Err(anyhow::Error)` - User-friendly error if writing fails
///
/// # Errors
///
/// Returns an error if:
/// - Permission is denied
/// - The parent directory does not exist
/// - An I/O error occurs during write or rename
/// - Disk space is insufficient
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use termide::file_io::write_file;
///
/// # fn main() -> Result<(), anyhow::Error> {
/// let path = Path::new("output.txt");
/// write_file(path, "Hello, world!")?;
/// # Ok(())
/// # }
/// ```
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    // Get the parent directory for the temp file
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    // Get file metadata to preserve permissions (if file exists)
    let existing_metadata = fs::metadata(path).ok();

    // Create a temporary file in the same directory
    // Format: .<original_name>.tmp.<process_id>
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let temp_name = format!(".{}.tmp.{}", file_name, std::process::id());
    let temp_path = parent.join(temp_name);

    // Write content to temporary file
    {
        let mut temp_file = fs::File::create(&temp_path).context(format!(
            "Failed to create temporary file for '{}'. Check that you have write permission in the directory.",
            path.display()
        ))?;

        temp_file.write_all(content.as_bytes()).context(format!(
            "Failed to write content to '{}'. Check available disk space.",
            path.display()
        ))?;

        // Ensure all data is written to disk
        temp_file.sync_all().context(format!(
            "Failed to sync file data to disk for '{}'.",
            path.display()
        ))?;
    }

    // Preserve file permissions if the original file existed
    #[cfg(unix)]
    if let Some(metadata) = existing_metadata {
        let permissions = metadata.permissions();
        fs::set_permissions(&temp_path, permissions).context(format!(
            "Failed to set permissions on temporary file for '{}'.",
            path.display()
        ))?;
    }

    // Atomically rename temporary file to target path
    fs::rename(&temp_path, path).context(format!(
        "Failed to save file '{}'. The temporary file has been left at '{}'.",
        path.display(),
        temp_path.display()
    ))?;

    Ok(())
}
