// App state module - manages TUI application state

use crate::scanner::ClaudeFolder;
use std::path::PathBuf;

/// Application states
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum AppState {
    Scanning,
    Browsing,
    Confirming,
    Deleting,
    Done,
}

/// Main application state
pub struct App {
    pub state: AppState,
    pub folders: Vec<ClaudeFolder>,
    pub selected_index: usize,
    pub scan_path: Option<PathBuf>,
    pub scan_complete: bool,
    pub should_quit: bool,
    pub permanent_delete: bool,
    pub show_help: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(permanent_delete: bool) -> Self {
        Self {
            state: AppState::Scanning,
            folders: Vec::new(),
            selected_index: 0,
            scan_path: None,
            scan_complete: false,
            should_quit: false,
            permanent_delete,
            show_help: false,
            message: None,
        }
    }

    pub fn add_folder(&mut self, folder: ClaudeFolder) {
        self.folders.push(folder);
        self.folders.sort_by(|a, b| b.size.cmp(&a.size));
    }

    pub fn set_scanning(&mut self, path: PathBuf) {
        self.scan_path = Some(path);
    }

    pub fn complete_scan(&mut self) {
        self.scan_complete = true;
        self.state = AppState::Browsing;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.folders.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(folder) = self.folders.get_mut(self.selected_index) {
            folder.selected = !folder.selected;
        }
    }

    pub fn select_all(&mut self) {
        for folder in &mut self.folders {
            folder.selected = true;
        }
    }

    pub fn select_none(&mut self) {
        for folder in &mut self.folders {
            folder.selected = false;
        }
    }

    pub fn selected_count(&self) -> usize {
        self.folders.iter().filter(|f| f.selected).count()
    }

    pub fn selected_size(&self) -> u64 {
        self.folders
            .iter()
            .filter(|f| f.selected)
            .map(|f| f.size)
            .sum()
    }

    pub fn total_size(&self) -> u64 {
        self.folders.iter().map(|f| f.size).sum()
    }

    pub fn get_selected_folders(&self) -> Vec<&ClaudeFolder> {
        self.folders.iter().filter(|f| f.selected).collect()
    }

    pub fn remove_deleted(&mut self, paths: &[PathBuf]) {
        self.folders.retain(|f| !paths.contains(&f.path));
        if self.selected_index >= self.folders.len() && !self.folders.is_empty() {
            self.selected_index = self.folders.len() - 1;
        }
    }
}
