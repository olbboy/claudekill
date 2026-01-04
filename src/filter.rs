//! Filtering and search functionality for folder lists

use crate::scanner::ClaudeFolder;
use std::time::{Duration, SystemTime};

/// Filter criteria for folders
#[derive(Default, Clone)]
pub struct Filter {
    /// Text search in path
    pub search_query: Option<String>,
    /// Filter by project types (empty = all)
    pub project_types: Vec<String>,
    /// Minimum size in bytes
    pub min_size: Option<u64>,
    /// Maximum age (folders older than this pass)
    pub max_age: Option<Duration>,
}

impl Filter {
    /// Check if folder matches all filter criteria
    pub fn matches(&self, folder: &ClaudeFolder) -> bool {
        // Search query filter (case-insensitive path match)
        if let Some(ref query) = self.search_query {
            let path_str = folder.path.to_string_lossy().to_lowercase();
            if !path_str.contains(&query.to_lowercase()) {
                return false;
            }
        }

        // Project type filter
        if !self.project_types.is_empty() && !self.project_types.contains(&folder.project_type) {
            return false;
        }

        // Size filter
        if let Some(min) = self.min_size {
            if folder.size < min {
                return false;
            }
        }

        // Age filter (folders older than max_age pass)
        if let Some(max_age) = self.max_age {
            if let Some(modified) = folder.modified_at {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed < max_age {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Check if any filter is active
    pub fn is_active(&self) -> bool {
        self.search_query.is_some()
            || !self.project_types.is_empty()
            || self.min_size.is_some()
            || self.max_age.is_some()
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Sort order for folder list
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SortOrder {
    #[default]
    SizeDesc,
    SizeAsc,
    NameAsc,
    NameDesc,
    DateDesc,
    DateAsc,
}

impl SortOrder {
    /// Cycle to next sort order
    pub fn next(&self) -> Self {
        match self {
            Self::SizeDesc => Self::SizeAsc,
            Self::SizeAsc => Self::NameAsc,
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::DateDesc,
            Self::DateDesc => Self::DateAsc,
            Self::DateAsc => Self::SizeDesc,
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::SizeDesc => "Size ↓",
            Self::SizeAsc => "Size ↑",
            Self::NameAsc => "Name A-Z",
            Self::NameDesc => "Name Z-A",
            Self::DateDesc => "Newest",
            Self::DateAsc => "Oldest",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_folder(path: &str, size: u64, project_type: &str) -> ClaudeFolder {
        ClaudeFolder {
            path: PathBuf::from(path),
            size,
            project_type: project_type.to_string(),
            selected: false,
            modified_at: Some(SystemTime::now()),
        }
    }

    #[test]
    fn test_filter_search_query() {
        let folder = make_folder("/home/user/rust_project/.claude", 1000, "Rust");
        let mut filter = Filter::default();

        // No filter matches everything
        assert!(filter.matches(&folder));

        // Matching search
        filter.search_query = Some("rust".to_string());
        assert!(filter.matches(&folder));

        // Non-matching search
        filter.search_query = Some("python".to_string());
        assert!(!filter.matches(&folder));
    }

    #[test]
    fn test_filter_size() {
        let folder = make_folder("/test/.claude", 1000, "Unknown");
        let mut filter = Filter::default();

        // No min_size
        assert!(filter.matches(&folder));

        // Below threshold
        filter.min_size = Some(2000);
        assert!(!filter.matches(&folder));

        // At threshold
        filter.min_size = Some(1000);
        assert!(filter.matches(&folder));
    }

    #[test]
    fn test_filter_project_type() {
        let folder = make_folder("/test/.claude", 1000, "Rust");
        let mut filter = Filter::default();

        // Empty types matches all
        assert!(filter.matches(&folder));

        // Matching type
        filter.project_types = vec!["Rust".to_string()];
        assert!(filter.matches(&folder));

        // Non-matching type
        filter.project_types = vec!["Python".to_string()];
        assert!(!filter.matches(&folder));
    }

    #[test]
    fn test_filter_is_active() {
        let mut filter = Filter::default();
        assert!(!filter.is_active());

        filter.search_query = Some("test".to_string());
        assert!(filter.is_active());

        filter.clear();
        assert!(!filter.is_active());
    }

    #[test]
    fn test_sort_order_cycle() {
        let order = SortOrder::SizeDesc;
        assert_eq!(order.next(), SortOrder::SizeAsc);
        assert_eq!(order.next().next(), SortOrder::NameAsc);
    }

    #[test]
    fn test_sort_order_labels() {
        assert_eq!(SortOrder::SizeDesc.label(), "Size ↓");
        assert_eq!(SortOrder::NameAsc.label(), "Name A-Z");
    }
}
