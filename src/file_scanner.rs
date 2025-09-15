use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;

/// Finds all config.json and globalConfig.json files recursively in the given directory
pub fn find_config_files<P: AsRef<Path>>(search_path: P) -> Result<Vec<PathBuf>> {
    let mut config_files = Vec::new();

    let walker = WalkDir::new(search_path.as_ref())
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok()); // Skip entries we can't read

    for entry in walker {
        let path = entry.path();

        if path.is_file()
            && let Some(file_name) = path.file_name()
            && let Some(name_str) = file_name.to_str()
            && (name_str == "config.json" || name_str == "globalConfig.json")
        {
            config_files.push(path.to_path_buf());
        }
    }

    if !search_path.as_ref().exists() {
        return Err(anyhow::anyhow!("Search path does not exist: {:?}", search_path.as_ref()));
    }

    Ok(config_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_find_config_files_basic() {
        let test_dir = Path::new("tests/fixtures");
        if test_dir.exists() {
            let result = find_config_files(test_dir);
            assert!(result.is_ok());
        }
    }
}