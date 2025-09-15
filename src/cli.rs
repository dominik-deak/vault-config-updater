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

    /// Dry run mode - show what files would be changed without modifying them
    #[arg(long)]
    pub dry_run: bool,
}

impl CliArgs {
    /// Get the search path, defaulting to current directory
    pub fn get_search_path(&self) -> &Path {
        match &self.path {
            Some(p) => Path::new(p),
            None => Path::new("."),
        }
    }

    /// Check if interactive input is needed (no token provided and not in dry-run mode)
    pub fn needs_interactive_input(&self) -> bool {
        self.token.is_none() && !self.dry_run
    }

    /// Get the token, prompting for input if not provided
    pub fn get_token(&mut self) -> Result<String> {
        if self.dry_run {
            return Err(anyhow::anyhow!("Token not required in dry-run mode"));
        }

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

    /// Get the token if available, returns None in dry-run mode
    pub fn get_token_if_needed(&mut self) -> Result<Option<String>> {
        if self.dry_run {
            Ok(None)
        } else {
            Ok(Some(self.get_token()?))
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
