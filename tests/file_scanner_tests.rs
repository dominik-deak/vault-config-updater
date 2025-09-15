use std::path::Path;
use vault_config_updater::find_config_files;

#[test]
fn test_finds_config_json_files() {
    let test_dir = Path::new("tests/fixtures");
    let files = find_config_files(test_dir).unwrap();

    let file_names: Vec<_> = files.iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(file_names.contains(&"config.json"));
}

#[test]
fn test_finds_global_config_json_files() {
    let test_dir = Path::new("tests/fixtures");
    let files = find_config_files(test_dir).unwrap();

    let file_names: Vec<_> = files.iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(file_names.contains(&"globalConfig.json"));
}

#[test]
fn test_finds_nested_config_files() {
    let test_dir = Path::new("tests/fixtures");
    let files = find_config_files(test_dir).unwrap();

    let nested_files: Vec<_> = files.iter()
        .filter(|p| p.to_str().unwrap().contains("nested"))
        .collect();

    assert!(!nested_files.is_empty());
}

#[test]
fn test_ignores_other_json_files() {
    let test_dir = Path::new("tests/fixtures");
    let files = find_config_files(test_dir).unwrap();

    let file_names: Vec<_> = files.iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(!file_names.contains(&"other.json"));
    assert!(!file_names.contains(&"malformed.json"));
}

#[test]
fn test_handles_nonexistent_directory() {
    let result = find_config_files(Path::new("nonexistent/directory"));
    assert!(result.is_err());
}