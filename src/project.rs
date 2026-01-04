// Project detection module - identifies project type from parent directory

use std::path::Path;

/// Detect project type based on parent directory contents
pub fn detect(claude_path: &Path) -> String {
    let Some(parent) = claude_path.parent() else {
        return "Unknown".to_string();
    };

    // Rust
    if parent.join("Cargo.toml").exists() {
        return "Rust".to_string();
    }

    // Node.js / JS ecosystem
    if parent.join("package.json").exists() {
        if parent.join("next.config.js").exists()
            || parent.join("next.config.mjs").exists()
            || parent.join("next.config.ts").exists()
        {
            return "Next.js".to_string();
        }
        if parent.join("nuxt.config.ts").exists() || parent.join("nuxt.config.js").exists() {
            return "Nuxt".to_string();
        }
        if parent.join("vite.config.ts").exists() || parent.join("vite.config.js").exists() {
            return "Vite".to_string();
        }
        if parent.join("angular.json").exists() {
            return "Angular".to_string();
        }
        return "Node.js".to_string();
    }

    // Python
    if parent.join("pyproject.toml").exists()
        || parent.join("setup.py").exists()
        || parent.join("requirements.txt").exists()
    {
        return "Python".to_string();
    }

    // Go
    if parent.join("go.mod").exists() {
        return "Go".to_string();
    }

    // Flutter/Dart
    if parent.join("pubspec.yaml").exists() {
        return "Flutter".to_string();
    }

    // Ruby
    if parent.join("Gemfile").exists() {
        return "Ruby".to_string();
    }

    // Java/Kotlin
    if parent.join("pom.xml").exists()
        || parent.join("build.gradle").exists()
        || parent.join("build.gradle.kts").exists()
    {
        return "Java".to_string();
    }

    "Unknown".to_string()
}
