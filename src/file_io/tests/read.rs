//! Unit tests for read_file function

use crate::file_io::read_file;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to create a test file with given content
fn create_test_file(dir: &TempDir, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content).unwrap();
    path
}

#[test]
fn test_read_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "empty.txt", b"");

    let content = read_file(&path).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_read_simple_text() {
    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "simple.txt", b"Hello, world!");

    let content = read_file(&path).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_read_multiline_text() {
    let dir = TempDir::new().unwrap();
    let text = "Line 1\nLine 2\nLine 3\n";
    let path = create_test_file(&dir, "multiline.txt", text.as_bytes());

    let content = read_file(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_read_unicode_text() {
    let dir = TempDir::new().unwrap();
    let text = "Hello 世界! 🚀 Привет мир!";
    let path = create_test_file(&dir, "unicode.txt", text.as_bytes());

    let content = read_file(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_read_file_with_special_characters() {
    let dir = TempDir::new().unwrap();
    let text = "Tab:\tNewline:\nCarriage return:\rNull:\0";
    let path = create_test_file(&dir, "special.txt", text.as_bytes());

    let content = read_file(&path).unwrap();
    assert_eq!(content, text);
}

#[test]
fn test_read_nonexistent_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.txt");

    let result = read_file(&path);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file"));
}

#[test]
fn test_read_invalid_utf8() {
    let dir = TempDir::new().unwrap();
    // Invalid UTF-8 sequence
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let path = create_test_file(&dir, "invalid.txt", &invalid_utf8);

    let result = read_file(&path);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("invalid UTF-8"));
}

#[cfg(unix)]
#[test]
fn test_read_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = create_test_file(&dir, "readonly.txt", b"test");

    // Remove all permissions
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&path, perms).unwrap();

    let result = read_file(&path);

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&path, perms).unwrap();

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file"));
}

#[test]
fn test_read_large_file() {
    let dir = TempDir::new().unwrap();
    // Create a 1MB file
    let large_content = "a".repeat(1024 * 1024);
    let path = create_test_file(&dir, "large.txt", large_content.as_bytes());

    let content = read_file(&path).unwrap();
    assert_eq!(content.len(), 1024 * 1024);
    assert_eq!(content, large_content);
}

#[test]
fn test_read_file_with_long_lines() {
    let dir = TempDir::new().unwrap();
    // Create a file with a very long line (10KB)
    let long_line = "x".repeat(10 * 1024);
    let path = create_test_file(&dir, "longline.txt", long_line.as_bytes());

    let content = read_file(&path).unwrap();
    assert_eq!(content, long_line);
}

#[test]
fn test_read_file_with_crlf_line_endings() {
    let dir = TempDir::new().unwrap();
    let text = "Line 1\r\nLine 2\r\nLine 3\r\n";
    let path = create_test_file(&dir, "crlf.txt", text.as_bytes());

    let content = read_file(&path).unwrap();
    // We preserve line endings as-is
    assert_eq!(content, text);
}
