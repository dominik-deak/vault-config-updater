use std::process;
use std::time::Instant;
use anyhow::Result;
use rayon::prelude::*;
use vault_config_updater::{
    parse_env_args, find_config_files, update_vault_token_in_file, UpdateStats,
    scan_vault_tokens_in_file, ScanStats
};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let start_time = Instant::now();
    let mut cli = parse_env_args()?;

    if cli.verbose {
        if cli.dry_run {
            println!("🔍 Vault Config Updater v0.1.0 - DRY RUN MODE");
        } else {
            println!("🔍 Vault Config Updater v0.1.0");
        }
        println!("📁 Searching in: {:?}", cli.get_search_path());
    }

    let search_path = cli.get_search_path();
    let config_files = find_config_files(search_path)?;

    if config_files.is_empty() {
        println!("⚠️  No config.json or globalConfig.json files found in {:?}", search_path);
        return Ok(());
    }

    if cli.verbose {
        println!("📋 Found {} config files:", config_files.len());
        for file in &config_files {
            println!("   • {}", file.display());
        }
    }

    if cli.dry_run {
        if cli.verbose {
            println!("🔍 DRY RUN MODE - No files will be modified");
        }
        let stats = scan_files_parallel(&config_files, cli.verbose)?;
        print_scan_results(&stats, start_time.elapsed());
        if !stats.errors.is_empty() {
            process::exit(1);
        }
    } else {
        let token = cli.get_token()?;
        if cli.verbose {
            println!("🎯 Token obtained (length: {} chars)", token.len());
        }
        let stats = update_files_parallel(&config_files, &token, cli.verbose)?;
        print_results(&stats, start_time.elapsed());
        if !stats.errors.is_empty() {
            process::exit(1);
        }
    }

    Ok(())
}

fn update_files_parallel(
    files: &[std::path::PathBuf],
    token: &str,
    verbose: bool
) -> Result<UpdateStats> {
    if verbose {
        println!("⚡ Processing {} files in parallel...", files.len());
    }

    let results: Vec<_> = files
        .par_iter()
        .map(|file| {
            let result = update_vault_token_in_file(file, token);
            (file.clone(), result)
        })
        .collect();

    let mut stats = UpdateStats::new();
    stats.files_processed = files.len();

    for (file, result) in results {
        match result {
            Ok(_) => {
                // Check if file was actually updated by reading it
                if let Ok(content) = std::fs::read_to_string(&file) {
                    let token_count = content.matches(&format!(r#""vaultToken": "{}""#, token)).count();
                    if token_count > 0 {
                        stats.files_updated += 1;
                        stats.tokens_replaced += token_count;
                        if verbose {
                            println!("   ✅ Updated {} (replaced {} tokens)", file.display(), token_count);
                        }
                    } else if verbose {
                        println!("   ⏭️  Skipped {} (no vaultToken fields)", file.display());
                    }
                } else if verbose {
                    println!("   ⚠️  Could not verify updates in {}", file.display());
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to process {}: {}", file.display(), e);
                stats.add_error(error_msg.clone());
                if verbose {
                    println!("   ❌ {}", error_msg);
                }
            }
        }
    }

    Ok(stats)
}

fn scan_files_parallel(
    files: &[std::path::PathBuf],
    verbose: bool
) -> Result<ScanStats> {
    if verbose {
        println!("🔍 Scanning {} files in parallel...", files.len());
    }

    let results: Vec<_> = files
        .par_iter()
        .map(|file| {
            let result = scan_vault_tokens_in_file(file);
            (file.clone(), result)
        })
        .collect();

    let mut stats = ScanStats::new();
    stats.files_scanned = files.len();

    for (file, result) in results {
        match result {
            Ok(token_count) => {
                if token_count > 0 {
                    stats.files_with_tokens += 1;
                    stats.total_tokens_found += token_count;
                    if verbose {
                        println!("   🎯 {} ({} vaultToken field{})",
                            file.display(),
                            token_count,
                            if token_count == 1 { "" } else { "s" });
                    }
                } else if verbose {
                    println!("   ⏭️  {} (no vaultToken fields)", file.display());
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to scan {}: {}", file.display(), e);
                stats.add_error(error_msg.clone());
                if verbose {
                    println!("   ❌ {}", error_msg);
                }
            }
        }
    }

    Ok(stats)
}

fn print_results(stats: &UpdateStats, duration: std::time::Duration) {
    println!("\n🎉 Update completed in {:.2}s", duration.as_secs_f64());
    println!("📊 Results:");
    println!("   • Files processed: {}", stats.files_processed);
    println!("   • Files updated: {}", stats.files_updated);
    println!("   • Tokens replaced: {}", stats.tokens_replaced);

    if !stats.errors.is_empty() {
        println!("   • Errors: {}", stats.errors.len());
        println!("\n❌ Errors encountered:");
        for error in &stats.errors {
            println!("   • {}", error);
        }
    }

    if stats.files_updated > 0 {
        println!("\n✨ Successfully updated vault tokens in {} files!", stats.files_updated);
    } else if stats.errors.is_empty() {
        println!("\nℹ️  No files needed updating (no vaultToken fields found).");
    }
}

fn print_scan_results(stats: &ScanStats, duration: std::time::Duration) {
    println!("\n🔍 DRY RUN scan completed in {:.2}s", duration.as_secs_f64());
    println!("📊 Results:");
    println!("   • Files scanned: {}", stats.files_scanned);
    println!("   • Files that would be updated: {}", stats.files_with_tokens);
    println!("   • Total tokens that would be replaced: {}", stats.total_tokens_found);

    if !stats.errors.is_empty() {
        println!("   • Errors: {}", stats.errors.len());
        println!("\n❌ Errors encountered:");
        for error in &stats.errors {
            println!("   • {}", error);
        }
    }

    if stats.files_with_tokens > 0 {
        println!("\n💡 {} file{} would be updated with new vault tokens!",
            stats.files_with_tokens,
            if stats.files_with_tokens == 1 { "" } else { "s" });
        println!("🚀 Run without --dry-run to apply these changes.");
    } else if stats.errors.is_empty() {
        println!("\nℹ️  No files would need updating (no vaultToken fields found).");
    }
}