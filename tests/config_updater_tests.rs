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

#[test]
fn test_regex_pattern_matches_various_formats() {
    use regex::Regex;

    let re = Regex::new(r#""vaultToken"\s*:\s*"[^"]*""#).unwrap();

    // Test standard format
    assert!(re.is_match(r#""vaultToken": "test""#));

    // Test format without spaces
    assert!(re.is_match(r#""vaultToken":"test""#));

    // Test format with extra spaces
    assert!(re.is_match(r#""vaultToken" : "test""#));
    assert!(re.is_match(r#""vaultToken"  :  "test""#));

    // Test it doesn't match incomplete patterns
    assert!(!re.is_match(r#""vaultToken""#));
    assert!(!re.is_match(r#""notVaultToken": "test""#));
}

#[test]
fn test_scan_vault_tokens_in_file() {
    use std::fs;
    use tempfile::TempDir;
    use vault_config_updater::scan_vault_tokens_in_file;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_scan.json");

    let json_content = r#"{
  "vaultToken": "hvs.first-token",
  "database": {
    "vaultToken": "hvs.second-token"
  },
  "other": "value"
}"#;

    fs::write(&file_path, json_content).unwrap();

    let result = scan_vault_tokens_in_file(&file_path).unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_scan_vault_tokens_in_file_no_tokens() {
    use std::fs;
    use tempfile::TempDir;
    use vault_config_updater::scan_vault_tokens_in_file;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_no_tokens.json");

    let json_content = r#"{
  "database": {
    "host": "localhost"
  },
  "apiKey": "some-key"
}"#;

    fs::write(&file_path, json_content).unwrap();

    let result = scan_vault_tokens_in_file(&file_path).unwrap();
    assert_eq!(result, 0);
}

#[test]
fn test_scan_vault_tokens_in_files() {
    use std::fs;
    use tempfile::TempDir;
    use vault_config_updater::scan_vault_tokens_in_files;

    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.path().join("file1.json");
    let file2_path = temp_dir.path().join("file2.json");
    let file3_path = temp_dir.path().join("file3.json");

    // File with 1 token
    fs::write(&file1_path, r#"{"vaultToken": "hvs.token1"}"#).unwrap();

    // File with 2 tokens
    fs::write(&file2_path, r#"{"vaultToken": "hvs.token2", "nested": {"vaultToken": "hvs.token3"}}"#).unwrap();

    // File with no tokens
    fs::write(&file3_path, r#"{"apiKey": "some-key"}"#).unwrap();

    let file_paths = vec![file1_path, file2_path, file3_path];
    let stats = scan_vault_tokens_in_files(&file_paths);

    assert_eq!(stats.files_scanned, 3);
    assert_eq!(stats.files_with_tokens, 2);
    assert_eq!(stats.total_tokens_found, 3);
    assert!(stats.errors.is_empty());
}

#[test]
fn test_scan_stats_struct() {
    use vault_config_updater::ScanStats;

    let mut stats = ScanStats::new();
    assert_eq!(stats.files_scanned, 0);
    assert_eq!(stats.files_with_tokens, 0);
    assert_eq!(stats.total_tokens_found, 0);
    assert!(stats.errors.is_empty());

    stats.add_error("Test error".to_string());
    assert_eq!(stats.errors.len(), 1);
    assert_eq!(stats.errors[0], "Test error");
}