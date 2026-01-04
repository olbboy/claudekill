// Scanner module - finds .claude folders recursively using parallel walking

use crate::project;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// Represents a found .claude folder with metadata
#[derive(Debug, Clone)]
pub struct ClaudeFolder {
    pub path: PathBuf,
    pub size: u64,
    pub project_type: String,
    pub selected: bool,
}

impl ClaudeFolder {
    /// Format size for display (e.g., "156.2 MB")
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.1} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

/// Events emitted during scanning
#[derive(Debug)]
pub enum ScanEvent {
    Scanning(PathBuf),
    Found(ClaudeFolder),
    Complete,
}

/// Scanner for finding .claude folders with parallel directory walking
pub struct Scanner {
    root: PathBuf,
    include_global: bool,
}

impl Scanner {
    pub fn new(root: PathBuf, include_global: bool) -> Self {
        Self {
            root,
            include_global,
        }
    }

    /// Scan for .claude folders, returns receiver for streaming results
    pub fn scan(&self) -> Receiver<ScanEvent> {
        let (tx, rx) = channel();
        let root = self.root.clone();
        let include_global = self.include_global;
        let global_path = dirs::home_dir().map(|h| h.join(".claude"));

        thread::spawn(move || {
            Self::scan_dir(&root, &tx, include_global, global_path.as_deref());
            let _ = tx.send(ScanEvent::Complete);
        });

        rx
    }

    fn scan_dir(
        root: &Path,
        tx: &Sender<ScanEvent>,
        include_global: bool,
        global_path: Option<&Path>,
    ) {
        // Use jwalk for parallel directory walking
        // Skip hidden directories except .claude for performance
        for entry in WalkDir::new(root)
            .skip_hidden(false)
            .process_read_dir(|_, _, _, children| {
                // Filter: keep .claude dirs, skip other hidden dirs
                children.retain(|e| {
                    if let Ok(e) = e {
                        let name = e.file_name.to_string_lossy();
                        // Keep if it's .claude or not hidden
                        name == ".claude" || !name.starts_with('.')
                    } else {
                        false
                    }
                });
            })
            .into_iter()
            .flatten()
        {
            let path = entry.path();

            // Check if it's a .claude directory
            if path.file_name().map(|n| n == ".claude").unwrap_or(false) && path.is_dir() {
                // Skip global ~/.claude unless include_global flag set
                if !include_global && global_path.map(|g| path == g).unwrap_or(false) {
                    continue;
                }

                // Send progress update
                let _ = tx.send(ScanEvent::Scanning(path.to_path_buf()));

                // Calculate folder size
                let size = calculate_dir_size(&path);

                // Detect project type from parent directory
                let project_type = project::detect(&path);

                let folder = ClaudeFolder {
                    path: path.to_path_buf(),
                    size,
                    project_type,
                    selected: false,
                };

                let _ = tx.send(ScanEvent::Found(folder));
            }
        }
    }
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
