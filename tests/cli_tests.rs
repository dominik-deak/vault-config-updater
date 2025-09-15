use clap::Parser;
use vault_config_updater::{CliArgs, parse_args};

#[test]
fn test_parses_token_argument() {
    let args = vec!["vault-config-updater", "hvs.test-token"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.token, Some("hvs.test-token".to_string()));
}

#[test]
fn test_parses_path_argument() {
    let args = vec!["vault-config-updater", "hvs.test-token", "/some/path"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.path, Some("/some/path".to_string()));
}

#[test]
fn test_default_current_directory() {
    let args = vec!["vault-config-updater", "hvs.test-token"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.get_search_path(), std::path::Path::new("."));
}

#[test]
fn test_custom_directory() {
    let args = vec!["vault-config-updater", "hvs.test-token", "/custom/path"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.get_search_path(), std::path::Path::new("/custom/path"));
}

#[test]
fn test_help_flag() {
    let args = vec!["vault-config-updater", "--help"];
    let result = CliArgs::try_parse_from(args);

    // Should fail with help message
    assert!(result.is_err());
}

#[test]
fn test_version_flag() {
    let args = vec!["vault-config-updater", "--version"];
    let result = CliArgs::try_parse_from(args);

    // Should fail with version message
    assert!(result.is_err());
}

#[test]
fn test_no_token_provided() {
    let args = vec!["vault-config-updater"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.token, None);
}

#[test]
fn test_parse_args_with_token() {
    let args = vec!["vault-config-updater", "hvs.my-token"];
    let result = parse_args(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert_eq!(cli.token, Some("hvs.my-token".to_string()));
}

#[test]
fn test_validate_hvs_token_format() {
    let valid_tokens = vec![
        "hvs.CAESIHGBcOKn9LDs8JpZkZ",
        "hvs.1234567890abcdef",
        "hvs.simple"
    ];

    for token in valid_tokens {
        let args = vec!["vault-config-updater", token];
        let result = CliArgs::try_parse_from(args);
        assert!(result.is_ok(), "Failed to parse valid token: {}", token);
    }
}

#[test]
fn test_interactive_mode_detection() {
    // Test when no token is provided, should prompt for interactive input
    let args = vec!["vault-config-updater"];
    let result = CliArgs::try_parse_from(args);

    assert!(result.is_ok());
    let cli = result.unwrap();
    assert!(cli.needs_interactive_input());
}