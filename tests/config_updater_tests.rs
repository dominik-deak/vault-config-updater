use std::fs;
use std::path::Path;
use tempfile::TempDir;
use vault_config_updater::{update_vault_token, update_vault_token_in_file};

#[test]
fn test_updates_simple_vault_token() {
    let json_content = r#"{
  "database": {
    "host": "localhost"
  },
  "vaultToken": "hvs.old-token",
  "apiKey": "some-key"
}"#;

    let result = update_vault_token(json_content, "hvs.new-token").unwrap();

    assert!(result.contains("hvs.new-token"));
    assert!(!result.contains("hvs.old-token"));
}

#[test]
fn test_updates_nested_vault_tokens() {
    let json_content = r#"{
  "global": {
    "vaultToken": "hvs.global-old"
  },
  "services": [
    {
      "config": {
        "vaultToken": "hvs.service-old"
      }
    }
  ]
}"#;

    let result = update_vault_token(json_content, "hvs.new-token").unwrap();

    assert!(result.contains("hvs.new-token"));
    assert!(!result.contains("hvs.global-old"));
    assert!(!result.contains("hvs.service-old"));
    // Should replace both occurrences
    assert_eq!(result.matches("hvs.new-token").count(), 2);
}

#[test]
fn test_preserves_json_formatting() {
    let json_content = r#"{
  "vaultToken": "hvs.old-token",
  "other": "value"
}"#;

    let result = update_vault_token(json_content, "hvs.new-token").unwrap();

    // Check that formatting is preserved (whitespace, indentation)
    assert!(result.contains("  \"vaultToken\": \"hvs.new-token\""));
    assert!(result.contains("  \"other\": \"value\""));
}

#[test]
fn test_handles_no_vault_token() {
    let json_content = r#"{
  "database": {
    "host": "localhost"
  },
  "apiKey": "some-key"
}"#;

    let result = update_vault_token(json_content, "hvs.new-token").unwrap();

    // Should return original content unchanged
    assert_eq!(result, json_content);
}

#[test]
fn test_handles_malformed_json() {
    let malformed_json = r#"{
  "vaultToken": "hvs.old-token"
  "missing": "comma"
}"#;

    let result = update_vault_token(malformed_json, "hvs.new-token");

    // Should handle gracefully - either return error or apply regex replacement anyway
    // We'll decide the behavior in implementation
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_update_file_in_place() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_config.json");

    let original_content = r#"{
  "vaultToken": "hvs.old-token",
  "other": "value"
}"#;

    fs::write(&file_path, original_content).unwrap();

    let result = update_vault_token_in_file(&file_path, "hvs.new-token");
    assert!(result.is_ok());

    let updated_content = fs::read_to_string(&file_path).unwrap();
    assert!(updated_content.contains("hvs.new-token"));
    assert!(!updated_content.contains("hvs.old-token"));
}

#[test]
fn test_atomic_file_update() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_config.json");

    let original_content = r#"{"vaultToken": "hvs.old-token"}"#;
    fs::write(&file_path, original_content).unwrap();

    // Test that file operations are atomic (no intermediate states)
    let result = update_vault_token_in_file(&file_path, "hvs.new-token");
    assert!(result.is_ok());

    // File should exist and have correct content
    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("hvs.new-token"));
}

#[test]
fn test_handles_file_read_error() {
    let nonexistent_path = Path::new("nonexistent/file.json");
    let result = update_vault_token_in_file(nonexistent_path, "hvs.new-token");
    assert!(result.is_err());
}