mod app;
mod config;
mod filter;
mod history;
mod project;
mod report;
mod scanner;
mod trash;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use config::Config;
use history::{DeletionMethod, DeletionRecord, History};
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "claudekill")]
#[command(author, version, about = "Find and delete .claude folders")]
pub struct Args {
    /// Directory to scan (default: home directory)
    #[arg(short, long)]
    path: Option<String>,

    /// List folders without interactive TUI
    #[arg(long)]
    dry_run: bool,

    /// Include global ~/.claude folder
    #[arg(long)]
    include_global: bool,

    /// Permanently delete instead of moving to Trash
    #[arg(long)]
    permanent: bool,

    /// Create default config file
    #[arg(long)]
    init_config: bool,

    /// Show config file location
    #[arg(long)]
    config_path: bool,

    /// Undo last trash-based deletion
    #[arg(long)]
    undo: bool,

    /// Show deletion history
    #[arg(long)]
    history: bool,

    /// Generate space analysis report
    #[arg(long)]
    report: bool,

    /// Export format: json, csv
    #[arg(long, value_name = "FORMAT")]
    export: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle config-related commands first
    if args.config_path {
        println!("{}", Config::config_path().display());
        return Ok(());
    }

    if args.init_config {
        match Config::create_default_if_missing() {
            Ok(true) => println!("Created config at: {}", Config::config_path().display()),
            Ok(false) => println!("Config already exists: {}", Config::config_path().display()),
            Err(e) => eprintln!("Failed to create config: {}", e),
        }
        return Ok(());
    }

    // Handle undo command
    if args.undo {
        return handle_undo();
    }

    // Handle history command
    if args.history {
        return handle_history();
    }

    // Load config (with graceful fallback to defaults)
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config: {}", e);
        Config::default()
    });

    // Determine root directory (CLI arg > config > home)
    let root = match &args.path {
        Some(p) => PathBuf::from(p),
        None => {
            if !config.scan.default_paths.is_empty() {
                config.scan.default_paths[0].clone()
            } else {
                dirs::home_dir().expect("Could not find home directory")
            }
        }
    };

    // Merge CLI flags with config (CLI takes precedence)
    let include_global = args.include_global || config.scan.include_global;
    let permanent = args.permanent || config.behavior.permanent_delete;
    let exclude_patterns = config.scan.exclude_patterns.clone();

    // Report mode - scan and generate report
    if args.report {
        return handle_report(
            &root,
            include_global,
            &exclude_patterns,
            args.export.as_deref(),
        );
    }

    // Dry-run mode - just list without TUI
    if args.dry_run {
        return dry_run(&root, include_global, &exclude_patterns);
    }

    // Interactive TUI mode
    run_tui(&root, include_global, permanent, &config, &exclude_patterns)
}

/// Handle --undo command
fn handle_undo() -> Result<()> {
    match history::undo_last()? {
        Some(restored) if !restored.is_empty() => {
            println!("Restored {} folder(s):", restored.len());
            for path in restored {
                println!("  {}", path.display());
            }
        }
        Some(_) => {
            println!("No folders could be restored.");
        }
        None => {
            println!("No undoable deletion found.");
            println!("Note: Only trash-based deletions can be undone.");
        }
    }
    Ok(())
}

/// Handle --history command
fn handle_history() -> Result<()> {
    let hist = History::load()?;

    if hist.records.is_empty() {
        println!("No deletion history.");
        return Ok(());
    }

    println!("Deletion History (most recent first):");
    println!("{:-<70}", "");

    for record in hist.records.iter().rev().take(20) {
        let method = match record.method {
            DeletionMethod::Trash => "Trash",
            DeletionMethod::Permanent => "Permanent",
        };
        let undo_marker = if record.can_undo() { " [undoable]" } else { "" };

        println!(
            "{}  {:>4} folder(s)  {:>10}  ({}){}",
            record.timestamp.format("%Y-%m-%d %H:%M"),
            record.paths.len(),
            utils::format_size(record.total_size),
            method,
            undo_marker
        );
    }

    if hist.records.len() > 20 {
        println!("... and {} more entries", hist.records.len() - 20);
    }

    Ok(())
}

/// Handle --report command
fn handle_report(
    root: &Path,
    include_global: bool,
    exclude_patterns: &[String],
    export_format: Option<&str>,
) -> Result<()> {
    println!("Scanning: {}", root.display());

    let scanner = scanner::Scanner::new(
        root.to_path_buf(),
        include_global,
        exclude_patterns.to_vec(),
    );
    let rx = scanner.scan();

    let mut folders = Vec::new();
    for event in rx {
        match event {
            scanner::ScanEvent::Found(folder) => folders.push(folder),
            scanner::ScanEvent::Complete => break,
            _ => {}
        }
    }

    let report = report::SpaceReport::generate(&folders);

    match export_format {
        Some("json") => println!("{}", report.to_json()),
        Some("csv") => print!("{}", report.to_csv()),
        Some(fmt) => eprintln!("Unknown export format: {}. Use 'json' or 'csv'.", fmt),
        None => report.print_summary(),
    }

    Ok(())
}

/// Dry-run mode: scan and list all .claude folders without TUI
fn dry_run(root: &Path, include_global: bool, exclude_patterns: &[String]) -> Result<()> {
    println!("Scanning: {}", root.display());
    println!();

    let scanner = scanner::Scanner::new(
        root.to_path_buf(),
        include_global,
        exclude_patterns.to_vec(),
    );
    let rx = scanner.scan();

    let mut folders = Vec::new();

    for event in rx {
        match event {
            scanner::ScanEvent::Found(folder) => {
                folders.push(folder);
            }
            scanner::ScanEvent::Complete => {
                break;
            }
            _ => {}
        }
    }

    // Sort by size descending
    folders.sort_by(|a, b| b.size.cmp(&a.size));

    // Display results
    if folders.is_empty() {
        println!("No .claude folders found.");
        return Ok(());
    }

    println!("Found {} .claude folder(s):\n", folders.len());
    println!("{:>10}  {:50}  PROJECT", "SIZE", "PATH");
    println!("{}", "-".repeat(80));

    for folder in &folders {
        let path_str = folder.path.display().to_string();
        let display_path = if path_str.len() > 50 {
            format!("...{}", &path_str[path_str.len() - 47..])
        } else {
            path_str
        };

        println!(
            "{:>10}  {:50}  {}",
            folder.size_display(),
            display_path,
            folder.project_type
        );
    }

    // Summary
    let total_size: u64 = folders.iter().map(|f| f.size).sum();
    println!("{}", "-".repeat(80));
    println!("{:>10}  Total", utils::format_size(total_size));

    Ok(())
}

/// Interactive TUI mode
fn run_tui(
    root: &Path,
    include_global: bool,
    permanent: bool,
    config: &Config,
    exclude_patterns: &[String],
) -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::init()?;

    // Initialize app state with config
    let mut app = app::App::new_with_config(permanent, config);

    // Start scanner in background
    let scanner = scanner::Scanner::new(
        root.to_path_buf(),
        include_global,
        exclude_patterns.to_vec(),
    );
    let rx = scanner.scan();

    // Main loop
    let result = (|| -> Result<()> {
        loop {
            // Process scanner events (non-blocking)
            loop {
                match rx.try_recv() {
                    Ok(scanner::ScanEvent::Scanning(path)) => {
                        app.set_scanning(path);
                    }
                    Ok(scanner::ScanEvent::Found(folder)) => {
                        app.add_folder(folder);
                    }
                    Ok(scanner::ScanEvent::Complete) => {
                        app.complete_scan();
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        if !app.scan_complete {
                            app.complete_scan();
                        }
                        break;
                    }
                }
            }

            // Render UI
            terminal.draw(|f| ui::render(f, &app))?;

            // Handle input
            let action = ui::handle_events(&mut app, Duration::from_millis(100))?;

            match action {
                ui::Action::Quit => break,
                ui::Action::Delete => {
                    // Collect paths to delete
                    let folders: Vec<PathBuf> = app
                        .get_selected_folders()
                        .iter()
                        .map(|f| f.path.clone())
                        .collect();

                    let deleted_size: u64 = app.get_selected_folders().iter().map(|f| f.size).sum();

                    // Safety validation before deletion
                    if let Err(e) = trash::validate_deletion(&folders) {
                        app.message = Some(format!("Safety check failed: {}", e));
                        app.state = app::AppState::Browsing;
                        continue;
                    }

                    // Perform deletion
                    let deletion_method = if app.permanent_delete {
                        DeletionMethod::Permanent
                    } else {
                        DeletionMethod::Trash
                    };

                    let result = if app.permanent_delete {
                        trash::permanent_delete(&folders)
                    } else {
                        trash::move_to_trash(&folders)
                    };

                    match result {
                        Ok(()) => {
                            // Record in history
                            let record = DeletionRecord::new(
                                folders.clone(),
                                deleted_size,
                                deletion_method.clone(),
                            );
                            if let Ok(mut hist) = History::load() {
                                hist.add(record);
                                let _ = hist.save();
                            }

                            let method = if app.permanent_delete {
                                "Deleted"
                            } else {
                                "Moved to Trash"
                            };
                            app.remove_deleted(&folders);
                            app.message = Some(format!(
                                "{} {} folder(s). {} reclaimed.",
                                method,
                                folders.len(),
                                utils::format_size(deleted_size)
                            ));
                            app.state = app::AppState::Browsing;
                        }
                        Err(e) => {
                            app.message = Some(format!("Error: {}", e));
                            app.state = app::AppState::Browsing;
                        }
                    }
                }
                ui::Action::None => {}
            }

            if app.should_quit {
                break;
            }
        }
        Ok(())
    })();

    // Always restore terminal, even on error
    tui::restore()?;

    result
}
