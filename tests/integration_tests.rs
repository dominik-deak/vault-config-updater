use std::fs;
use tempfile::TempDir;
use vault_config_updater::{find_config_files, update_vault_tokens_in_files};

#[test]
fn test_end_to_end_workflow() {
    // Create a temporary directory with test files
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create test config files
    let config1_path = temp_path.join("config.json");
    let config1_content = r#"{
  "database": "localhost",
  "vaultToken": "hvs.old-token-1",
  "other": "value"
}"#;
    fs::write(&config1_path, config1_content).unwrap();

    let global_config_path = temp_path.join("globalConfig.json");
    let global_config_content = r#"{
  "global": {
    "vaultToken": "hvs.old-token-2"
  },
  "service": {
    "vaultToken": "hvs.old-token-3"
  }
}"#;
    fs::write(&global_config_path, global_config_content).unwrap();

    // Create a nested directory with another config
    let nested_dir = temp_path.join("nested");
    fs::create_dir(&nested_dir).unwrap();
    let nested_config_path = nested_dir.join("config.json");
    let nested_config_content = r#"{"vaultToken": "hvs.old-token-4"}"#;
    fs::write(&nested_config_path, nested_config_content).unwrap();

    // Create a non-config file that should be ignored
    let other_file_path = temp_path.join("other.json");
    let other_content = r#"{"vaultToken": "hvs.should-not-change"}"#;
    fs::write(&other_file_path, other_content).unwrap();

    // Test the complete workflow
    let new_token = "hvs.new-super-secret-token";

    // 1. Find config files
    let config_files = find_config_files(temp_path).unwrap();
    assert_eq!(config_files.len(), 3); // Should find 3 config files, not the other.json

    // 2. Update tokens in all files
    let stats = update_vault_tokens_in_files(&config_files, new_token);

    // 3. Verify results
    assert_eq!(stats.files_processed, 3);
    assert_eq!(stats.files_updated, 3);
    assert_eq!(stats.tokens_replaced, 4); // 1 + 2 + 1 tokens
    assert!(stats.errors.is_empty());

    // 4. Verify file contents were updated correctly
    let updated_config1 = fs::read_to_string(&config1_path).unwrap();
    assert!(updated_config1.contains(new_token));
    assert!(!updated_config1.contains("hvs.old-token-1"));

    let updated_global = fs::read_to_string(&global_config_path).unwrap();
    assert!(updated_global.contains(new_token));
    assert!(!updated_global.contains("hvs.old-token-2"));
    assert!(!updated_global.contains("hvs.old-token-3"));

    let updated_nested = fs::read_to_string(&nested_config_path).unwrap();
    assert!(updated_nested.contains(new_token));
    assert!(!updated_nested.contains("hvs.old-token-4"));

    // 5. Verify non-config file was not touched
    let unchanged_other = fs::read_to_string(&other_file_path).unwrap();
    assert!(unchanged_other.contains("hvs.should-not-change"));
}

#[test]
fn test_no_config_files_found() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create only non-config files
    fs::write(temp_path.join("other.json"), r#"{"key": "value"}"#).unwrap();
    fs::write(temp_path.join("data.txt"), "some text").unwrap();

    let config_files = find_config_files(temp_path).unwrap();
    assert_eq!(config_files.len(), 0);
}

#[test]
fn test_files_without_vault_tokens() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create config files without vaultToken fields
    let config_path = temp_path.join("config.json");
    let config_content = r#"{
  "database": "localhost",
  "apiKey": "some-key"
}"#;
    fs::write(&config_path, config_content).unwrap();

    let config_files = find_config_files(temp_path).unwrap();
    assert_eq!(config_files.len(), 1);

    let stats = update_vault_tokens_in_files(&config_files, "hvs.new-token");
    assert_eq!(stats.files_processed, 1);
    assert_eq!(stats.files_updated, 0); // No tokens to update
    assert_eq!(stats.tokens_replaced, 0);
    assert!(stats.errors.is_empty());
}

#[test]
fn test_mixed_file_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a valid config file
    let good_config_path = temp_path.join("config.json");
    fs::write(&good_config_path, r#"{"vaultToken": "hvs.old"}"#).unwrap();

    // Create a directory that looks like a config file (should cause an error)
    let bad_path = temp_path.join("globalConfig.json");
    fs::create_dir(&bad_path).unwrap();

    let config_files = vec![good_config_path, bad_path];
    let stats = update_vault_tokens_in_files(&config_files, "hvs.new");

    assert_eq!(stats.files_processed, 2);
    assert_eq!(stats.files_updated, 1); // Only the good file
    assert_eq!(stats.tokens_replaced, 1);
    assert_eq!(stats.errors.len(), 1); // One error from the directory
}

#[test]
fn test_concurrent_file_updates() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create multiple config files to test parallel processing
    let mut file_paths = Vec::new();
    for i in 0..10 {
        let file_path = temp_path.join(format!("config{}.json", i));
        let content = format!(r#"{{"service": "service{}", "vaultToken": "hvs.old-{i}"}}"#, i, i = i);
        fs::write(&file_path, content).unwrap();
        file_paths.push(file_path);
    }

    // Also create globalConfig files
    for i in 0..5 {
        let file_path = temp_path.join(format!("globalConfig{}.json", i));
        let content = format!(r#"{{"vaultToken": "hvs.global-old-{i}"}}"#, i = i);
        fs::write(&file_path, content).unwrap();
        file_paths.push(file_path);
    }

    let new_token = "hvs.concurrent-test-token";
    let stats = update_vault_tokens_in_files(&file_paths, new_token);

    assert_eq!(stats.files_processed, 15);
    assert_eq!(stats.files_updated, 15);
    assert_eq!(stats.tokens_replaced, 15);
    assert!(stats.errors.is_empty());

    // Verify all files were updated correctly
    for (i, file_path) in file_paths.iter().enumerate() {
        let content = fs::read_to_string(file_path).unwrap();
        assert!(content.contains(new_token), "File {} not updated: {}", i, content);
    }
}

#[test]
fn test_preserves_json_structure_and_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let original_content = r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "credentials": {
      "username": "admin",
      "vaultToken": "hvs.db-token"
    }
  },
  "api": {
    "vaultToken": "hvs.api-token",
    "timeout": 30
  },
  "metadata": {
    "version": "1.0.0",
    "environment": "production"
  }
}"#;

    fs::write(&config_path, original_content).unwrap();

    let stats = update_vault_tokens_in_files(&[&config_path], "hvs.new-token");
    assert_eq!(stats.tokens_replaced, 2);

    let updated_content = fs::read_to_string(&config_path).unwrap();

    // Verify structure is preserved
    assert!(updated_content.contains("hvs.new-token"));
    assert!(updated_content.contains(r#""host": "localhost""#));
    assert!(updated_content.contains(r#""port": 5432"#));
    assert!(updated_content.contains(r#""username": "admin""#));
    assert!(updated_content.contains(r#""timeout": 30"#));
    assert!(updated_content.contains(r#""version": "1.0.0""#));

    // Verify old tokens are gone
    assert!(!updated_content.contains("hvs.db-token"));
    assert!(!updated_content.contains("hvs.api-token"));

    // Verify basic formatting is preserved (indentation)
    assert!(updated_content.contains("  \"database\""));
    assert!(updated_content.contains("    \"host\""));
}