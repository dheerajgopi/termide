//! Unit tests for write_file function

use crate::file_io::write_file;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_write_simple_text() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");

    write_file(&path, "Hello, world!").unwrap();

    // Verify file was created and contains correct content
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_write_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.txt");

    write_file(&path, "").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_write_multiline_text() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("multiline.txt");
    let text = "Line 1\nLine 2\nLine 3\n";

    write_file(&path, text).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_write_unicode_text() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("unicode.txt");
    let text = "Hello 世界! 🚀 Привет мир!";

    write_file(&path, text).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_write_overwrite_existing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("overwrite.txt");

    // Write initial content
    write_file(&path, "Initial content").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "Initial content");

    // Overwrite with new content
    write_file(&path, "New content").unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "New content");
}

#[test]
fn test_write_creates_new_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("new_file.txt");

    assert!(!path.exists());

    write_file(&path, "New file content").unwrap();

    assert!(path.exists());
    assert_eq!(fs::read_to_string(&path).unwrap(), "New file content");
}

#[cfg(unix)]
#[test]
fn test_write_preserves_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("perms.txt");

    // Create file with specific permissions
    write_file(&path, "Initial").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o600); // Read/write for owner only
    fs::set_permissions(&path, perms).unwrap();

    // Verify initial permissions
    let initial_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(initial_mode, 0o600);

    // Write new content
    write_file(&path, "Updated content").unwrap();

    // Verify permissions were preserved
    let final_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(final_mode, 0o600);
}

#[test]
fn test_write_large_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("large.txt");
    let large_content = "a".repeat(1024 * 1024); // 1MB

    write_file(&path, &large_content).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content.len(), 1024 * 1024);
    assert_eq!(content, large_content);
}

#[test]
fn test_write_special_characters() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("special.txt");
    let text = "Tab:\tNewline:\nCarriage return:\rNull:\0";

    write_file(&path, text).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_write_no_temp_file_left_on_success() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");

    write_file(&path, "Test content").unwrap();

    // Check that no temporary files are left in the directory
    let entries: Vec<PathBuf> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0], path);
}

#[cfg(unix)]
#[test]
fn test_write_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();

    // Remove write permission from directory
    let mut perms = fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(dir.path(), perms).unwrap();

    let path = dir.path().join("denied.txt");
    let result = write_file(&path, "Test");

    // Restore permissions for cleanup
    let mut perms = fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(dir.path(), perms).unwrap();

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to create temporary file"));
}

#[test]
fn test_write_with_crlf_line_endings() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("crlf.txt");
    let text = "Line 1\r\nLine 2\r\nLine 3\r\n";

    write_file(&path, text).unwrap();

    // Verify CRLF line endings are preserved
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_write_atomic_behavior() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("atomic.txt");

    // Write initial content
    write_file(&path, "Initial").unwrap();

    // The write should be atomic - either complete new content or old content
    // This test verifies the file exists after write
    write_file(&path, "Updated").unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Updated");
}
