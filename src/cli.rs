use std::path::Path;
use std::io::{self, Write};
use clap::Parser;
use anyhow::Result;

/// High-performance concurrent HashiCorp Vault token updater for configuration files
#[derive(Parser, Debug)]
#[command(name = "vault-config-updater")]
#[command(version = "0.1.0")]
#[command(about = "Updates HashiCorp Vault tokens in config.json and globalConfig.json files")]
#[command(long_about = "Recursively finds config.json and globalConfig.json files and updates their vaultToken fields concurrently using all available CPU cores.")]
pub struct CliArgs {
    /// HashiCorp Vault token (hvs.xxx format). If not provided, will prompt for input.
    #[arg(value_name = "TOKEN")]
    pub token: Option<String>,

    /// Directory to search for config files (default: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl CliArgs {
    /// Get the search path, defaulting to current directory
    pub fn get_search_path(&self) -> &Path {
        match &self.path {
            Some(p) => Path::new(p),
            None => Path::new("."),
        }
    }

    /// Check if interactive input is needed (no token provided)
    pub fn needs_interactive_input(&self) -> bool {
        self.token.is_none()
    }

    /// Get the token, prompting for input if not provided
    pub fn get_token(&mut self) -> Result<String> {
        match &self.token {
            Some(token) => Ok(token.clone()),
            None => {
                print!("Enter Vault token: ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                let token = input.trim().to_string();
                if token.is_empty() {
                    return Err(anyhow::anyhow!("Token cannot be empty"));
                }

                // Store the token for future use
                self.token = Some(token.clone());
                Ok(token)
            }
        }
    }
}

/// Parse command line arguments
pub fn parse_args<I, T>(args: I) -> Result<CliArgs>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = CliArgs::try_parse_from(args)?;
    Ok(cli)
}

/// Parse command line arguments from env::args()
pub fn parse_env_args() -> Result<CliArgs> {
    let cli = CliArgs::parse();
    Ok(cli)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_search_path_default() {
        let cli = CliArgs {
            token: Some("hvs.test".to_string()),
            path: None,
            verbose: false,
        };
        assert_eq!(cli.get_search_path(), Path::new("."));
    }

    #[test]
    fn test_get_search_path_custom() {
        let cli = CliArgs {
            token: Some("hvs.test".to_string()),
            path: Some("/custom/path".to_string()),
            verbose: false,
        };
        assert_eq!(cli.get_search_path(), Path::new("/custom/path"));
    }

    #[test]
    fn test_needs_interactive_input() {
        let cli_no_token = CliArgs {
            token: None,
            path: None,
            verbose: false,
        };
        assert!(cli_no_token.needs_interactive_input());

        let cli_with_token = CliArgs {
            token: Some("hvs.test".to_string()),
            path: None,
            verbose: false,
        };
        assert!(!cli_with_token.needs_interactive_input());
    }
}