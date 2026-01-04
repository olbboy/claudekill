# Claudekill - Product Overview & Requirements

## Project Summary

**Claudekill** is an interactive CLI utility for finding and deleting `.claude` cache directories to reclaim disk space, inspired by npkill. Built in Rust for performance and safety.

## Phase 5: Distribution

### Functional Requirements

1. **Multi-platform installation support**
   - Cargo: `cargo install claudekill` (crates.io)
   - Homebrew: `brew install leozqin/tap/claudekill` (macOS)
   - GitHub Releases: Direct binary downloads
   - cargo-binstall support for optimized downloads

2. **Automated release pipeline**
   - Multi-target builds (x86_64 macOS, ARM64 macOS, Linux GNU)
   - Artifact packaging with tar.gz compression
   - GitHub Releases with generated notes
   - CI/CD validation (formatting, clippy, tests)

3. **Release process documentation**
   - Pre-release checklist (version updates, testing)
   - Release execution (tagging, pushing)
   - Post-release tasks (Homebrew SHA256, crates.io publish)

### Non-Functional Requirements

- **Performance**: Binary stripping and LTO enabled in release builds
- **Safety**: Moves to Trash by default (safe deletion)
- **Reliability**: Parallel directory scanning with error handling
- **Maintainability**: Clear release checklist for repeatable deployments

### Acceptance Criteria

- [ ] Binaries available via all three distribution channels
- [ ] Homebrew formula with correct SHA256 hashes
- [ ] GitHub Actions workflows execute without errors
- [ ] Release notes auto-generated on GitHub Releases
- [ ] Installation methods documented in README
- [ ] Version matrix tested across target platforms

### Technical Constraints

- Rust edition: 2024
- Target platforms: macOS x86_64, macOS ARM64, Linux x86_64 GNU
- MIT license compliance across all distribution channels
- Homebrew formula compatible with standard tap structure

### Success Metrics

- Zero failed CI runs on main/master
- Successful multi-target release builds
- Cargo.toml crates.io metadata complete
- Homebrew formula validated and functional
- Release checklist followed consistently

## Core Features (Maintained)

- Fast parallel scanning with `jwalk`
- Interactive TUI with ratatui/crossterm
- Safe deletion via trash crate
- Customizable scan paths and filters
- Help system and keyboard shortcuts

## Architecture Overview

```
claudekill/
├── src/
│   ├── main.rs        # CLI entry point
│   ├── scanner.rs     # Directory scanning logic
│   ├── tui.rs         # Terminal UI
│   └── trash.rs       # Trash integration
├── .github/workflows/
│   ├── ci.yml         # Format/clippy/test on PRs
│   └── release.yml    # Multi-target builds & GH release
├── homebrew/
│   └── claudekill.rb  # macOS Homebrew formula
└── Cargo.toml         # Package metadata & dependencies
```

## Key Decisions

- **Distributed via crates.io**: Preferred method for Rust community
- **Homebrew formula in separate tap**: Community-driven installation
- **Direct GitHub Releases**: Advanced users / manual installation
- **cargo-binstall support**: Optimized pre-built binaries
- **Release tagging strategy**: Git tags trigger automated builds (v0.x.x format)
