//! Deletion history and undo functionality

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Maximum history entries to retain
const MAX_HISTORY_ENTRIES: usize = 100;

/// Deletion method used
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeletionMethod {
    Trash,
    Permanent,
}

/// Record of a single deletion operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionRecord {
    pub timestamp: DateTime<Utc>,
    pub paths: Vec<PathBuf>,
    pub total_size: u64,
    pub method: DeletionMethod,
}

impl DeletionRecord {
    pub fn new(paths: Vec<PathBuf>, total_size: u64, method: DeletionMethod) -> Self {
        Self {
            timestamp: Utc::now(),
            paths,
            total_size,
            method,
        }
    }

    pub fn can_undo(&self) -> bool {
        self.method == DeletionMethod::Trash
    }
}

/// Deletion history manager
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct History {
    pub records: Vec<DeletionRecord>,
}

impl History {
    /// Load history from disk
    pub fn load() -> Result<Self> {
        let path = Self::history_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read history: {}", path.display()))?;

        serde_json::from_str(&content).with_context(|| "Failed to parse history")
    }

    /// Save history to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::history_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Add a deletion record
    pub fn add(&mut self, record: DeletionRecord) {
        self.records.push(record);

        // Trim to max entries
        if self.records.len() > MAX_HISTORY_ENTRIES {
            self.records
                .drain(0..self.records.len() - MAX_HISTORY_ENTRIES);
        }
    }

    /// Get the last undoable deletion
    pub fn last_undoable(&self) -> Option<&DeletionRecord> {
        self.records.iter().rev().find(|r| r.can_undo())
    }

    /// Remove the last undoable record (after successful undo)
    pub fn remove_last_undoable(&mut self) {
        if let Some(pos) = self
            .records
            .iter()
            .rposition(|r| r.method == DeletionMethod::Trash)
        {
            self.records.remove(pos);
        }
    }

    /// Get history file path
    pub fn history_path() -> PathBuf {
        ProjectDirs::from("", "", "claudekill")
            .map(|dirs| dirs.cache_dir().join("history.json"))
            .unwrap_or_else(|| {
                dirs::cache_dir()
                    .unwrap_or_default()
                    .join("claudekill/history.json")
            })
    }
}

/// Attempt to undo the last trash-based deletion
/// Returns the list of paths that were successfully restored
pub fn undo_last() -> Result<Option<Vec<PathBuf>>> {
    let mut history = History::load()?;

    let Some(record) = history.last_undoable().cloned() else {
        return Ok(None);
    };

    if record.method != DeletionMethod::Trash {
        anyhow::bail!("Last deletion was permanent and cannot be undone");
    }

    // Attempt to restore from trash
    let mut restored = Vec::new();
    let mut errors = Vec::new();

    for path in &record.paths {
        match restore_from_trash(path) {
            Ok(()) => restored.push(path.clone()),
            Err(e) => errors.push(format!("{}: {}", path.display(), e)),
        }
    }

    // If at least one was restored, remove from history
    if !restored.is_empty() {
        history.remove_last_undoable();
        history.save()?;
    }

    if !errors.is_empty() {
        eprintln!("Some folders could not be restored:");
        for err in errors {
            eprintln!("  {}", err);
        }
    }

    Ok(Some(restored))
}

/// Restore a path from system trash (platform-specific)
fn restore_from_trash(path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Try trash CLI first (brew install trash)
        let result = Command::new("trash")
            .args(["-r", &path.to_string_lossy()])
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                return Ok(());
            }
        }

        // Fallback: inform user to restore manually
        anyhow::bail!(
            "Could not auto-restore. Please restore manually from Trash: {}",
            path.display()
        );
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Try gio trash restore
        let trash_path = format!(
            "trash:///{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let result = Command::new("gio")
            .args(["trash", "--restore", &trash_path])
            .output();

        if let Ok(output) = result {
            if output.status.success() {
                return Ok(());
            }
        }

        // Fallback: inform user
        anyhow::bail!(
            "Could not auto-restore. Please restore manually from Trash: {}",
            path.display()
        );
    }

    #[cfg(target_os = "windows")]
    {
        // Windows requires complex COM interfaces for trash restoration
        anyhow::bail!(
            "Auto-restore not supported on Windows. Please restore manually from Recycle Bin: {}",
            path.display()
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        anyhow::bail!("Undo not supported on this platform");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deletion_record_can_undo() {
        let trash_record = DeletionRecord::new(
            vec![PathBuf::from("/test/.claude")],
            1024,
            DeletionMethod::Trash,
        );
        assert!(trash_record.can_undo());

        let permanent_record = DeletionRecord::new(
            vec![PathBuf::from("/test/.claude")],
            1024,
            DeletionMethod::Permanent,
        );
        assert!(!permanent_record.can_undo());
    }

    #[test]
    fn test_history_add() {
        let mut history = History::default();
        let record = DeletionRecord::new(vec![], 0, DeletionMethod::Trash);
        history.add(record);
        assert_eq!(history.records.len(), 1);
    }

    #[test]
    fn test_history_last_undoable() {
        let mut history = History::default();

        // Add permanent (not undoable)
        history.add(DeletionRecord::new(vec![], 0, DeletionMethod::Permanent));
        assert!(history.last_undoable().is_none());

        // Add trash (undoable)
        history.add(DeletionRecord::new(vec![], 0, DeletionMethod::Trash));
        assert!(history.last_undoable().is_some());
    }

    #[test]
    fn test_history_remove_last_undoable() {
        let mut history = History::default();
        history.add(DeletionRecord::new(vec![], 100, DeletionMethod::Trash));
        history.add(DeletionRecord::new(vec![], 200, DeletionMethod::Permanent));
        history.add(DeletionRecord::new(vec![], 300, DeletionMethod::Trash));

        assert_eq!(history.records.len(), 3);
        history.remove_last_undoable();
        assert_eq!(history.records.len(), 2);
        // The 300-size trash record should be gone
        assert!(history.records.iter().all(|r| r.total_size != 300));
    }

    #[test]
    fn test_history_path_not_empty() {
        let path = History::history_path();
        assert!(!path.as_os_str().is_empty());
    }
}
