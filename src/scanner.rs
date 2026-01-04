// Scanner module - finds .claude folders recursively using parallel walking

use crate::project;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::SystemTime;

/// Represents a found .claude folder with metadata
#[derive(Debug, Clone)]
pub struct ClaudeFolder {
    pub path: PathBuf,
    pub size: u64,
    pub project_type: String,
    pub selected: bool,
    pub modified_at: Option<SystemTime>,
}

impl ClaudeFolder {
    /// Format size for display (e.g., "156.2 MB")
    pub fn size_display(&self) -> String {
        crate::utils::format_size(self.size)
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
    exclude_patterns: Vec<String>,
}

impl Scanner {
    pub fn new(root: PathBuf, include_global: bool, exclude_patterns: Vec<String>) -> Self {
        Self {
            root,
            include_global,
            exclude_patterns,
        }
    }

    /// Scan for .claude folders, returns receiver for streaming results
    pub fn scan(&self) -> Receiver<ScanEvent> {
        let (tx, rx) = channel();
        let root = self.root.clone();
        let include_global = self.include_global;
        let exclude_patterns = self.exclude_patterns.clone();
        let global_path = dirs::home_dir().map(|h| h.join(".claude"));

        thread::spawn(move || {
            Self::scan_dir(
                &root,
                &tx,
                include_global,
                global_path.as_deref(),
                &exclude_patterns,
            );
            let _ = tx.send(ScanEvent::Complete);
        });

        rx
    }

    /// Check if a path should be excluded based on patterns
    fn should_exclude(path: &Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        patterns.iter().any(|pattern| path_str.contains(pattern))
    }

    fn scan_dir(
        root: &Path,
        tx: &Sender<ScanEvent>,
        include_global: bool,
        global_path: Option<&Path>,
        exclude_patterns: &[String],
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

                // Skip if matches exclusion pattern
                if Self::should_exclude(&path, exclude_patterns) {
                    continue;
                }

                // Send progress update
                let _ = tx.send(ScanEvent::Scanning(path.to_path_buf()));

                // Calculate folder size
                let size = calculate_dir_size(&path);

                // Detect project type from parent directory
                let project_type = project::detect(&path);

                // Get modification time
                let modified_at = std::fs::metadata(&path).and_then(|m| m.modified()).ok();

                let folder = ClaudeFolder {
                    path: path.to_path_buf(),
                    size,
                    project_type,
                    selected: false,
                    modified_at,
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
