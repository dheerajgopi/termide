//! Integration tests for file_io module
//!
//! These tests verify the interaction between read_file and write_file functions.

use crate::file_io::{read_file, write_file};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_write_then_read() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.txt");
    let content = "Hello, world!";

    // Write content
    write_file(&path, content).unwrap();

    // Read it back
    let read_content = read_file(&path).unwrap();

    assert_eq!(read_content, content);
}

#[test]
fn test_write_read_empty_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("empty.txt");

    write_file(&path, "").unwrap();
    let content = read_file(&path).unwrap();

    assert_eq!(content, "");
}

#[test]
fn test_write_read_large_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("large.txt");
    let large_content = "Line\n".repeat(1000); // 1000 lines

    write_file(&path, &large_content).unwrap();
    let read_content = read_file(&path).unwrap();

    assert_eq!(read_content, large_content);
}

#[test]
fn test_write_read_unicode() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("unicode.txt");
    let unicode_content = "世界 🌍 Мир 🚀 Emoji: 😀🎉";

    write_file(&path, unicode_content).unwrap();
    let read_content = read_file(&path).unwrap();

    assert_eq!(read_content, unicode_content);
}

#[test]
fn test_multiple_write_read_cycles() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cycles.txt");

    for i in 0..10 {
        let content = format!("Iteration {}", i);
        write_file(&path, &content).unwrap();
        let read_content = read_file(&path).unwrap();
        assert_eq!(read_content, content);
    }
}

#[test]
fn test_overwrite_and_verify() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("overwrite.txt");

    // Write initial content
    write_file(&path, "Initial content that is quite long").unwrap();
    assert_eq!(
        read_file(&path).unwrap(),
        "Initial content that is quite long"
    );

    // Overwrite with shorter content
    write_file(&path, "Short").unwrap();
    assert_eq!(read_file(&path).unwrap(), "Short");

    // Overwrite with longer content
    write_file(&path, "Much longer content than before").unwrap();
    assert_eq!(
        read_file(&path).unwrap(),
        "Much longer content than before"
    );
}

#[test]
fn test_read_after_external_modification() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("external.txt");

    // Write using our function
    write_file(&path, "Our content").unwrap();

    // Modify using standard library
    fs::write(&path, "External content").unwrap();

    // Read using our function
    let content = read_file(&path).unwrap();
    assert_eq!(content, "External content");
}

#[test]
fn test_write_preserves_content_integrity() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("integrity.txt");

    // Create content with various characters that could cause issues
    let content = "Line 1\nLine 2\r\nTab:\tNull:\0Special: \x1b[31mRed\x1b[0m";

    write_file(&path, content).unwrap();
    let read_content = read_file(&path).unwrap();

    assert_eq!(read_content, content);
    assert_eq!(read_content.len(), content.len());
}

#[test]
fn test_nonexistent_file_error_handling() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.txt");

    let result = read_file(&path);
    assert!(result.is_err());

    // Error message should be user-friendly
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file"));
    assert!(error_msg.contains(path.to_str().unwrap()));
}

#[cfg(unix)]
#[test]
fn test_permission_handling() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("perms.txt");

    // Create file with custom permissions
    write_file(&path, "Test").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o640);
    fs::set_permissions(&path, perms).unwrap();

    // Write and verify permissions are preserved
    write_file(&path, "Updated").unwrap();
    let final_perms = fs::metadata(&path).unwrap().permissions();
    assert_eq!(final_perms.mode() & 0o777, 0o640);

    // Verify content is correct
    assert_eq!(read_file(&path).unwrap(), "Updated");
}

#[test]
fn test_special_filenames() {
    let dir = TempDir::new().unwrap();

    // Test various special filenames
    let filenames = vec![
        "file with spaces.txt",
        "file_with_underscores.txt",
        "file-with-dashes.txt",
        "file.multiple.dots.txt",
        "UPPERCASE.TXT",
    ];

    for filename in filenames {
        let path = dir.path().join(filename);
        let content = format!("Content for {}", filename);

        write_file(&path, &content).unwrap();
        let read_content = read_file(&path).unwrap();

        assert_eq!(read_content, content);
    }
}

#[test]
fn test_concurrent_writes_to_different_files() {
    let dir = TempDir::new().unwrap();

    // Write to multiple files
    for i in 0..5 {
        let path = dir.path().join(format!("file{}.txt", i));
        let content = format!("Content {}", i);
        write_file(&path, &content).unwrap();
    }

    // Verify all files
    for i in 0..5 {
        let path = dir.path().join(format!("file{}.txt", i));
        let content = read_file(&path).unwrap();
        assert_eq!(content, format!("Content {}", i));
    }
}
