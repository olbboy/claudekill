mod app;
mod project;
mod scanner;
mod trash;
mod tui;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "claudekill")]
#[command(author, version, about = "Find and delete .claude folders")]
struct Args {
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine root directory to scan
    let root = match args.path {
        Some(p) => PathBuf::from(p),
        None => dirs::home_dir().expect("Could not find home directory"),
    };

    // Dry-run mode - just list without TUI
    if args.dry_run {
        return dry_run(&root, args.include_global);
    }

    // Interactive TUI mode
    run_tui(&root, args.include_global, args.permanent)
}

/// Dry-run mode: scan and list all .claude folders without TUI
fn dry_run(root: &Path, include_global: bool) -> Result<()> {
    println!("Scanning: {}", root.display());
    println!();

    let scanner = scanner::Scanner::new(root.to_path_buf(), include_global);
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
fn run_tui(root: &Path, include_global: bool, permanent: bool) -> Result<()> {
    // Initialize terminal
    let mut terminal = tui::init()?;

    // Initialize app state
    let mut app = app::App::new(permanent);

    // Start scanner in background
    let scanner = scanner::Scanner::new(root.to_path_buf(), include_global);
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
                    let result = if app.permanent_delete {
                        trash::permanent_delete(&folders)
                    } else {
                        trash::move_to_trash(&folders)
                    };

                    match result {
                        Ok(()) => {
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
