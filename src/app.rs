// App state module - manages TUI application state

use crate::config::Config;
use crate::filter::{Filter, SortOrder};
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

/// Input mode for keyboard handling
#[derive(Default, Clone, Copy, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
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
    // Filter/search state
    pub filter: Filter,
    pub sort_order: SortOrder,
    pub input_mode: InputMode,
    pub search_input: String,
    pub show_filter_bar: bool,
}

impl App {
    #[allow(dead_code)]
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
            filter: Filter::default(),
            sort_order: SortOrder::default(),
            input_mode: InputMode::Normal,
            search_input: String::new(),
            show_filter_bar: false,
        }
    }

    /// Create App with config-based defaults
    pub fn new_with_config(permanent_delete: bool, config: &Config) -> Self {
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
            filter: Filter::default(),
            sort_order: config.parse_sort_order(),
            input_mode: InputMode::Normal,
            search_input: String::new(),
            show_filter_bar: config.display.show_filter_bar,
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
        let visible_count = self.visible_folder_indices().len();
        if self.selected_index < visible_count.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Move selection up by page_size items
    pub fn page_up(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    /// Move selection down by page_size items
    pub fn page_down(&mut self, page_size: usize) {
        let visible_count = self.visible_folder_indices().len();
        let max_index = visible_count.saturating_sub(1);
        self.selected_index = (self.selected_index + page_size).min(max_index);
    }

    /// Move selection to first item
    pub fn go_home(&mut self) {
        self.selected_index = 0;
    }

    /// Move selection to last item
    pub fn go_end(&mut self) {
        let visible_count = self.visible_folder_indices().len();
        if visible_count > 0 {
            self.selected_index = visible_count - 1;
        }
    }

    /// Get the actual folder index from the visible list position
    fn get_actual_folder_index(&self) -> Option<usize> {
        let visible = self.visible_folder_indices();
        visible.get(self.selected_index).copied()
    }

    pub fn toggle_selection(&mut self) {
        if let Some(actual_idx) = self.get_actual_folder_index() {
            if let Some(folder) = self.folders.get_mut(actual_idx) {
                folder.selected = !folder.selected;
            }
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

    /// Get filtered and sorted folder indices
    pub fn visible_folder_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .folders
            .iter()
            .enumerate()
            .filter(|(_, f)| self.filter.matches(f))
            .map(|(i, _)| i)
            .collect();

        // Sort by current sort order
        match self.sort_order {
            SortOrder::SizeDesc => {
                indices.sort_by(|&a, &b| self.folders[b].size.cmp(&self.folders[a].size))
            }
            SortOrder::SizeAsc => {
                indices.sort_by(|&a, &b| self.folders[a].size.cmp(&self.folders[b].size))
            }
            SortOrder::NameAsc => {
                indices.sort_by(|&a, &b| self.folders[a].path.cmp(&self.folders[b].path))
            }
            SortOrder::NameDesc => {
                indices.sort_by(|&a, &b| self.folders[b].path.cmp(&self.folders[a].path))
            }
            SortOrder::DateDesc => indices.sort_by(|&a, &b| {
                self.folders[b]
                    .modified_at
                    .cmp(&self.folders[a].modified_at)
            }),
            SortOrder::DateAsc => indices.sort_by(|&a, &b| {
                self.folders[a]
                    .modified_at
                    .cmp(&self.folders[b].modified_at)
            }),
        }

        indices
    }

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_input.clear();
    }

    /// Exit search mode without applying
    pub fn exit_search_mode(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Apply search query and exit search mode
    pub fn apply_search(&mut self) {
        if self.search_input.is_empty() {
            self.filter.search_query = None;
        } else {
            self.filter.search_query = Some(self.search_input.clone());
        }
        self.input_mode = InputMode::Normal;
        self.selected_index = 0;
    }

    /// Toggle filter bar visibility
    pub fn toggle_filter_bar(&mut self) {
        self.show_filter_bar = !self.show_filter_bar;
    }

    /// Cycle through sort orders
    pub fn cycle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        self.filter.clear();
        self.search_input.clear();
        self.selected_index = 0;
    }

    /// Get visible folder count (after filtering)
    pub fn visible_count(&self) -> usize {
        self.visible_folder_indices().len()
    }
}
