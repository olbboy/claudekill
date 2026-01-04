// Trash module - handles moving folders to Trash or permanent deletion

use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

/// Move folders to system Trash/Recycle Bin
pub fn move_to_trash(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        trash::delete(path)
            .with_context(|| format!("Failed to move to trash: {}", path.display()))?;
    }
    Ok(())
}

/// Permanently delete folders (bypass Trash)
pub fn permanent_delete(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to delete: {}", path.display()))?;
    }
    Ok(())
}

/// Validate paths before deletion - safety checks
pub fn validate_deletion(paths: &[PathBuf]) -> Result<()> {
    // Forbidden system directories (platform-specific)
    #[cfg(target_os = "windows")]
    let forbidden: &[&str] = &[
        "C:\\",
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Program Files (x86)",
        "C:\\Users",
    ];

    #[cfg(not(target_os = "windows"))]
    let forbidden: &[&str] = &["/", "/Users", "/System", "/Library", "/Applications"];

    for path in paths {
        let path_str = path.to_string_lossy();

        // Check against forbidden paths (case-insensitive on Windows)
        for forbidden_path in forbidden {
            #[cfg(target_os = "windows")]
            let matches = path_str.eq_ignore_ascii_case(forbidden_path);
            #[cfg(not(target_os = "windows"))]
            let matches = path_str == *forbidden_path;

            if matches {
                anyhow::bail!("Refusing to delete system directory: {}", path_str);
            }
        }

        // Verify it's actually a .claude folder
        if path.file_name() != Some(OsStr::new(".claude")) {
            anyhow::bail!("Not a .claude folder: {}", path_str);
        }

        // Verify path exists
        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path_str);
        }

        // Verify it's a directory
        if !path.is_dir() {
            anyhow::bail!("Path is not a directory: {}", path_str);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_deletion_valid_claude_folder() {
        let temp = tempdir().unwrap();
        let claude_path = temp.path().join(".claude");
        fs::create_dir(&claude_path).unwrap();

        let result = validate_deletion(&[claude_path]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_deletion_rejects_non_claude_folder() {
        let temp = tempdir().unwrap();
        let other_path = temp.path().join("other");
        fs::create_dir(&other_path).unwrap();

        let result = validate_deletion(&[other_path]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a .claude folder"));
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_validate_deletion_rejects_system_paths() {
        let result = validate_deletion(&[PathBuf::from("/Users")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("system directory"));
    }

    #[test]
    fn test_validate_deletion_rejects_nonexistent() {
        let result = validate_deletion(&[PathBuf::from("/nonexistent/.claude")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_permanent_delete() {
        let temp = tempdir().unwrap();
        let claude_path = temp.path().join(".claude");
        fs::create_dir(&claude_path).unwrap();
        fs::write(claude_path.join("test.txt"), "test").unwrap();

        let result = permanent_delete(&[claude_path.clone()]);
        assert!(result.is_ok());
        assert!(!claude_path.exists());
    }

    #[test]
    fn test_move_to_trash() {
        let temp = tempdir().unwrap();
        let claude_path = temp.path().join(".claude");
        fs::create_dir(&claude_path).unwrap();
        fs::write(claude_path.join("test.txt"), "test").unwrap();

        let result = move_to_trash(&[claude_path.clone()]);
        assert!(result.is_ok());
        assert!(!claude_path.exists());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_deletion_rejects_windows_system_paths() {
        let result = validate_deletion(&[PathBuf::from("C:\\Users")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("system directory"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_validate_deletion_rejects_windows_root() {
        let result = validate_deletion(&[PathBuf::from("C:\\")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("system directory"));
    }
}
