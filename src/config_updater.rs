use std::fs;
use std::path::Path;
use anyhow::Result;
use regex::Regex;

/// Updates all vaultToken values in a JSON string with the new token
pub fn update_vault_token(json_content: &str, new_token: &str) -> Result<String> {
    let re = Regex::new(r#""vaultToken"\s*:\s*"[^"]*""#)?;
    let replacement = format!(r#""vaultToken": "{}""#, new_token);

    let updated_content = re.replace_all(json_content, replacement.as_str());
    Ok(updated_content.to_string())
}

/// Updates vaultToken values in a file atomically
pub fn update_vault_token_in_file<P: AsRef<Path>>(file_path: P, new_token: &str) -> Result<()> {
    let path = file_path.as_ref();
    let original_content = fs::read_to_string(path)?;
    let updated_content = update_vault_token(&original_content, new_token)?;
    if updated_content != original_content {
        // Use a temporary file for atomic updates
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, &updated_content)?;
        fs::rename(&temp_path, path)?;
    }

    Ok(())
}

/// Statistics about the update operation
#[derive(Debug, PartialEq, Default)]
pub struct UpdateStats {
    pub files_processed: usize,
    pub files_updated: usize,
    pub tokens_replaced: usize,
    pub errors: Vec<String>,
}

impl UpdateStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
}

/// Updates vault tokens in multiple files and returns statistics
pub fn update_vault_tokens_in_files<P: AsRef<Path>>(
    file_paths: &[P],
    new_token: &str
) -> UpdateStats {
    let mut stats = UpdateStats::new();

    for path in file_paths {
        stats.files_processed += 1;

        match update_vault_token_in_file(path, new_token) {
            Ok(_) => {
                if let Ok(content) = fs::read_to_string(path) {
                    let token_count = content.matches(&format!(r#""vaultToken": "{}""#, new_token)).count();
                    if token_count > 0 {
                        stats.files_updated += 1;
                        stats.tokens_replaced += token_count;
                    }
                }
            }
            Err(e) => {
                stats.add_error(format!("Error processing {:?}: {}", path.as_ref(), e));
            }
        }
    }

    stats
}
