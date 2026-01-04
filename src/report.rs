//! Space analysis and report generation

use crate::scanner::ClaudeFolder;
use crate::utils::format_size;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Statistics for a project type
#[derive(Debug, Serialize)]
pub struct TypeStats {
    pub count: usize,
    pub total_size: u64,
    pub avg_size: u64,
}

/// Age breakdown of folders
#[derive(Debug, Serialize, Default)]
pub struct AgeBreakdown {
    pub under_1_week: usize,
    pub under_1_month: usize,
    pub under_3_months: usize,
    pub over_3_months: usize,
}

/// Summary of a single folder
#[derive(Debug, Serialize)]
pub struct FolderSummary {
    pub path: String,
    pub size: u64,
    pub size_human: String,
    pub project_type: String,
}

/// Complete space analysis report
#[derive(Debug, Serialize)]
pub struct SpaceReport {
    pub total_folders: usize,
    pub total_size: u64,
    pub total_size_human: String,
    pub by_project_type: HashMap<String, TypeStats>,
    pub age_breakdown: AgeBreakdown,
    pub top_10_largest: Vec<FolderSummary>,
}

impl SpaceReport {
    /// Generate report from folder list
    pub fn generate(folders: &[ClaudeFolder]) -> Self {
        let total_folders = folders.len();
        let total_size: u64 = folders.iter().map(|f| f.size).sum();

        // Group by project type
        let mut by_type: HashMap<String, Vec<&ClaudeFolder>> = HashMap::new();
        for folder in folders {
            by_type
                .entry(folder.project_type.clone())
                .or_default()
                .push(folder);
        }

        let by_project_type: HashMap<String, TypeStats> = by_type
            .into_iter()
            .map(|(name, list)| {
                let count = list.len();
                let total: u64 = list.iter().map(|f| f.size).sum();
                (
                    name,
                    TypeStats {
                        count,
                        total_size: total,
                        avg_size: if count > 0 { total / count as u64 } else { 0 },
                    },
                )
            })
            .collect();

        // Age breakdown
        let age_breakdown = Self::calculate_age_breakdown(folders);

        // Top 10 largest
        let mut sorted: Vec<_> = folders.iter().collect();
        sorted.sort_by(|a, b| b.size.cmp(&a.size));
        let top_10_largest: Vec<FolderSummary> = sorted
            .into_iter()
            .take(10)
            .map(|f| FolderSummary {
                path: f.path.to_string_lossy().to_string(),
                size: f.size,
                size_human: format_size(f.size),
                project_type: f.project_type.clone(),
            })
            .collect();

        Self {
            total_folders,
            total_size,
            total_size_human: format_size(total_size),
            by_project_type,
            age_breakdown,
            top_10_largest,
        }
    }

    fn calculate_age_breakdown(folders: &[ClaudeFolder]) -> AgeBreakdown {
        let now = SystemTime::now();
        let week = Duration::from_secs(7 * 24 * 60 * 60);
        let month = Duration::from_secs(30 * 24 * 60 * 60);
        let quarter = Duration::from_secs(90 * 24 * 60 * 60);

        let mut breakdown = AgeBreakdown::default();

        for folder in folders {
            if let Some(modified) = folder.modified_at {
                if let Ok(age) = now.duration_since(modified) {
                    if age < week {
                        breakdown.under_1_week += 1;
                    } else if age < month {
                        breakdown.under_1_month += 1;
                    } else if age < quarter {
                        breakdown.under_3_months += 1;
                    } else {
                        breakdown.over_3_months += 1;
                    }
                }
            }
        }

        breakdown
    }

    /// Export to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Export to CSV (all folders, not just top 10)
    pub fn to_csv(&self) -> String {
        let mut csv = String::from("Path,Size (bytes),Size (human),Project Type\n");
        for folder in &self.top_10_largest {
            csv.push_str(&format!(
                "\"{}\",{},{},{}\n",
                folder.path, folder.size, folder.size_human, folder.project_type
            ));
        }
        csv
    }

    /// Print human-readable summary to stdout
    pub fn print_summary(&self) {
        println!();
        println!("=== ClaudeKill Space Analysis ===");
        println!();
        println!("Total Folders: {}", self.total_folders);
        println!("Total Size:    {}", self.total_size_human);
        println!();

        println!("By Project Type:");
        println!("{:-<60}", "");
        let mut types: Vec<_> = self.by_project_type.iter().collect();
        types.sort_by(|a, b| b.1.total_size.cmp(&a.1.total_size));
        for (name, stats) in types {
            println!(
                "  {:15} {:>4} folders  {:>10}  (avg: {})",
                name,
                stats.count,
                format_size(stats.total_size),
                format_size(stats.avg_size)
            );
        }
        println!();

        println!("By Age:");
        println!("{:-<60}", "");
        println!(
            "  < 1 week:    {:>4} folders",
            self.age_breakdown.under_1_week
        );
        println!(
            "  < 1 month:   {:>4} folders",
            self.age_breakdown.under_1_month
        );
        println!(
            "  < 3 months:  {:>4} folders",
            self.age_breakdown.under_3_months
        );
        println!(
            "  > 3 months:  {:>4} folders",
            self.age_breakdown.over_3_months
        );
        println!();

        if !self.top_10_largest.is_empty() {
            println!("Top {} Largest:", self.top_10_largest.len());
            println!("{:-<60}", "");
            for (i, folder) in self.top_10_largest.iter().enumerate() {
                let path = if folder.path.len() > 45 {
                    format!("...{}", &folder.path[folder.path.len() - 42..])
                } else {
                    folder.path.clone()
                };
                println!("  {:>2}. {:>10}  {}", i + 1, folder.size_human, path);
            }
            println!();
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
    fn test_generate_report() {
        let folders = vec![
            make_folder("/a/.claude", 1000, "Rust"),
            make_folder("/b/.claude", 2000, "Rust"),
            make_folder("/c/.claude", 500, "Node"),
        ];

        let report = SpaceReport::generate(&folders);

        assert_eq!(report.total_folders, 3);
        assert_eq!(report.total_size, 3500);
        assert_eq!(report.by_project_type.len(), 2);
        assert_eq!(report.by_project_type.get("Rust").unwrap().count, 2);
    }

    #[test]
    fn test_to_json() {
        let folders = vec![make_folder("/test/.claude", 1000, "Unknown")];
        let report = SpaceReport::generate(&folders);
        let json = report.to_json();

        assert!(json.contains("\"total_folders\": 1"));
        assert!(json.contains("\"total_size\": 1000"));
    }

    #[test]
    fn test_to_csv() {
        let folders = vec![make_folder("/test/.claude", 1000, "Unknown")];
        let report = SpaceReport::generate(&folders);
        let csv = report.to_csv();

        assert!(csv.starts_with("Path,Size (bytes),Size (human),Project Type\n"));
        assert!(csv.contains("/test/.claude"));
    }
}
