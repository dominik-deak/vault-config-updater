# Vault Config Updater

A high-performance, concurrent HashiCorp Vault token updater for configuration files. This Rust CLI tool recursively finds `config.json` and `globalConfig.json` files and updates their `vaultToken` fields using all available CPU cores for blazing-fast performance.

## Features

- ⚡ **High Performance**: Uses Rayon for parallel processing across all CPU cores
- 🔍 **Smart Discovery**: Recursively finds `config.json` and `globalConfig.json` files
- 🔒 **Safe Updates**: Atomic file operations prevent corruption
- 📄 **Format Preservation**: Maintains original JSON formatting and structure
- ⌨️ **Flexible Input**: Accept tokens via command line or interactive prompt
- ✅ **Comprehensive Testing**: 35+ unit and integration tests with 90%+ coverage
- ✨ **Quality Assurance**: Pre-push hooks ensure code quality and security

## Installation

### From Source

Clone the repository and install using Cargo:

```bash
git clone https://github.com/dominik-deak/vault-config-updater.git
cd vault-config-updater
cargo install --path .
```

This will install the `vault-config-updater` command globally, making it available from any directory.

### Prerequisites

- Rust 1.89.0 or later (using Rust Edition 2024)
- Git (for cloning the repository)

## Usage

### Basic Usage

Update vault tokens in the current directory:

```bash
vault-config-updater hvs.YOUR_NEW_TOKEN_HERE
```

### Interactive Mode

If you don't provide a token, you'll be prompted to enter it securely:

```bash
vault-config-updater
# Enter Vault token: hvs.YOUR_TOKEN_HERE
```

### Custom Directory

Specify a different directory to search:

```bash
vault-config-updater hvs.YOUR_TOKEN /path/to/config/directory
```

### Verbose Output

See detailed information about the update process:

```bash
vault-config-updater hvs.YOUR_TOKEN --verbose
```

Example verbose output:
```
🚀 Vault Config Updater v0.1.0
🔍 Searching in: "."
🔑 Token obtained (length: 18 chars)
📁 Found 3 config files:
   • ./app/config.json
   • ./services/globalConfig.json
   • ./nested/service/config.json
⚙️ Processing 3 files in parallel...
    ✓ Updated ./app/config.json (replaced 1 tokens)
    ✓ Updated ./services/globalConfig.json (replaced 2 tokens)
    ✓ Updated ./nested/service/config.json (replaced 1 tokens)

⏱️ Update completed in 0.01s
📊 Results:
   • Files processed: 3
   • Files updated: 3
   • Tokens replaced: 4

✅ Successfully updated vault tokens in 3 files!
```

### Help

View all available options:

```bash
vault-config-updater --help
```

## How It Works

1. **Discovery**: Uses `walkdir` to recursively scan directories for `config.json` and `globalConfig.json` files
2. **Pattern Matching**: Employs regex to find and replace `vaultToken` fields while preserving JSON structure
3. **Parallel Processing**: Leverages Rayon to process multiple files concurrently across all CPU cores
4. **Atomic Updates**: Writes to temporary files and atomically renames them to prevent corruption
5. **Verification**: Validates updates and reports comprehensive statistics

## File Patterns

The tool will update `vaultToken` fields in these files:
- `config.json`
- `globalConfig.json`

It supports nested token structures like:
```json
{
  "database": {
    "vaultToken": "hvs.old-token"
  },
  "services": [
    {
      "config": {
        "vaultToken": "hvs.another-old-token"
      }
    }
  ]
}
```

All instances of `"vaultToken": "..."` will be replaced while preserving the original JSON formatting.

## Development

### Building from Source

```bash
git clone https://github.com/dominik-deak/vault-config-updater.git
cd vault-config-updater
cargo build --release
```

### Running Tests

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test categories
cargo test file_scanner
cargo test config_updater
cargo test integration
```

### Pre-push Hooks

The project uses cargo-husky for quality assurance. Before each push, it automatically runs:
- Cargo clippy (linting)
- Cargo check (compilation)
- Cargo test (all tests)
- Cargo audit (security audit)

### Project Structure

```
vault-config-updater/
  src/
     main.rs              # CLI entry point with rayon parallelization
     lib.rs               # Library root
     cli.rs               # Command-line argument parsing
     config_updater.rs    # Core token update logic
     file_scanner.rs      # File discovery using walkdir
  tests/
     cli_tests.rs         # CLI argument parsing tests
     config_updater_tests.rs  # Token update logic tests
     file_scanner_tests.rs    # File discovery tests
     integration_tests.rs     # End-to-end workflow tests
     fixtures/            # Test data files
  .cargo-husky/hooks/      # Git hook configurations
  Cargo.toml               # Project dependencies and metadata
```

## Performance

The Rust implementation provides significant performance improvements over the original bash script:

- **Parallel Processing**: Uses all available CPU cores
- **Efficient I/O**: Minimizes file system operations
- **Memory Efficient**: Streams large files when necessary
- **Fast Pattern Matching**: Compiled regex for optimal performance

## Dependencies

- `walkdir`: Fast recursive directory traversal
- `rayon`: Data parallelism for concurrent file processing
- `clap`: Command-line argument parsing with derive macros
- `regex`: Fast pattern matching for token replacement
- `serde_json`: JSON parsing and validation
- `anyhow`: Ergonomic error handling

## Security

- Tokens are cleared from memory after use
- Atomic file operations prevent partial writes
- No logging or storage of sensitive token data
- Security audit runs automatically via pre-push hooks