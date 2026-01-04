# Claudekill - Codebase Summary

## Project Overview

**Claudekill** is a command-line utility written in Rust that finds and interactively deletes `.claude` cache directories to reclaim disk space. Inspired by npkill, it provides a safe, user-friendly interface for managing Claude Code cache artifacts.

## Technology Stack

- **Language**: Rust (Edition 2024)
- **TUI Framework**: ratatui 0.29 + crossterm 0.28
- **Directory Scanning**: jwalk 0.8 (parallel traversal)
- **File Deletion**: trash 5 (safe, OS-native)
- **CLI Parsing**: clap 4.5 (argument handling)
- **Error Handling**: anyhow 1.0 (context propagation)

## Project Structure

```
claudekill/
├── Cargo.toml                 # Package metadata, dependencies, release config
├── Cargo.lock                 # Dependency lock
├── README.md                  # Quick start, installation methods
├── LICENSE                    # MIT license
├── RELEASING.md               # Release process checklist
├── src/
│   ├── main.rs               # Entry point, CLI arg parsing, app flow
│   ├── scanner.rs            # .claude directory detection & scanning
│   ├── tui.rs                # Terminal UI state, rendering, events
│   └── trash.rs              # Trash integration, safe/permanent deletion
├── .github/workflows/
│   ├── ci.yml                # Format/clippy/test on PR/push
│   └── release.yml           # Multi-target build & GitHub Release
├── homebrew/
│   └── claudekill.rb         # macOS Homebrew formula
└── docs/                      # Comprehensive documentation
    ├── project-overview-pdr.md
    ├── system-architecture.md
    ├── code-standards.md
    ├── deployment-guide.md
    └── codebase-summary.md (this file)
```

## Key Components

### 1. CLI Entry Point (`main.rs`)

**Responsibilities**:
- Parse command-line arguments using clap
- Initialize app state (scan path, filter options)
- Launch TUI or execute non-interactive mode
- Handle graceful shutdown

**Key flags**:
- `--path`: Custom directory to scan (default: home)
- `--dry-run`: List mode without TUI
- `--include-global`: Include ~/.claude (default: excluded)
- `--permanent`: Permanent delete vs Trash

### 2. Directory Scanner (`scanner.rs`)

**Responsibilities**:
- Recursively traverse directory tree using jwalk
- Identify `.claude` directories by name
- Calculate folder sizes and detect project types
- Handle permission errors gracefully

**Algorithm**:
1. Start from root path (usually home directory)
2. Parallel walk with multiple threads
3. Match directories named `.claude`
4. Collect metadata (size, depth, parent project)
5. Return sorted results to TUI

**Performance**:
- Parallelism adapts to CPU cores
- Memory-efficient streaming (no tree load-to-memory)
- Estimated 1-5 seconds for typical home directory

### 3. Terminal UI (`tui.rs`)

**Responsibilities**:
- Render interactive list of .claude directories
- Handle keyboard input (navigation, selection, deletion)
- Display folder sizes and project information
- Show help/status messages
- Update display after deletions

**State Machine**:
- **Browse**: Navigate list, view details
- **Confirm**: Ask user to confirm deletion
- **Deleting**: Show progress/results
- **Help**: Display keyboard shortcuts

**Keyboard Controls**:
| Key | Action |
|-----|--------|
| ↑/k | Up |
| ↓/j | Down |
| Space | Toggle |
| a | Select all |
| n | Deselect all |
| d | Delete selected |
| ? | Help |
| q | Quit |

### 4. Trash Integration (`trash.rs`)

**Responsibilities**:
- Safely move directories to OS Trash
- Fallback to permanent deletion if trash fails
- Handle permission/ownership issues
- Report results to user

**Implementation**:
- Uses `trash` crate (cross-platform)
- macOS: Moves to ~/.Trash
- Linux: Moves to ~/.local/share/Trash
- Provides undo capability

## Build & Release

### Release Profile Configuration

```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Remove debug symbols
panic = "abort"        # Smaller binary
```

**Result**: Single executable ~5-10MB per target

### Supported Platforms

| Platform | Target Triple | Status |
|----------|---|---|
| macOS Intel | x86_64-apple-darwin | ✓ Active |
| macOS Apple Silicon | aarch64-apple-darwin | ✓ Active |
| Linux GNU | x86_64-unknown-linux-gnu | ✓ Active |
| Windows | - | Not supported |

### CI/CD Pipeline

**Continuous Integration**:
- Runs on: macOS latest
- Triggers: PR, push to main/master
- Checks: format, clippy, build, test
- Blocks: PRs with failures

**Release Automation**:
- Trigger: Git tag push (v*)
- Builds: All 3 platform targets in parallel
- Artifacts: tar.gz packages
- Release: GitHub Releases with generated notes

## Distribution Channels

### 1. Cargo Registry (crates.io)

```bash
cargo install claudekill
```

**Metadata** (from Cargo.toml):
- Name: claudekill
- License: MIT
- Repository: https://github.com/leozqin/claudekill
- Keywords: cli, tui, claude, cleanup, disk-space
- Cargo-binstall support for pre-compiled downloads

### 2. Homebrew (macOS)

```bash
brew install leozqin/tap/claudekill
```

**Formula** (`homebrew/claudekill.rb`):
- Detects macOS architecture (ARM64 vs Intel)
- Fetches pre-compiled binary from GitHub Releases
- Validates SHA256 checksum
- Installs to standard PATH

**Update process**: Manual SHA256 updates post-release

### 3. GitHub Releases

Direct binary downloads for all platforms. Artifacts:
- `claudekill-x86_64-apple-darwin.tar.gz`
- `claudekill-aarch64-apple-darwin.tar.gz`
- `claudekill-x86_64-unknown-linux-gnu.tar.gz`

## Dependencies Analysis

### Runtime Dependencies

| Crate | Version | Purpose | Maintenance |
|-------|---------|---------|--------------|
| ratatui | 0.29 | Terminal UI rendering | Active |
| crossterm | 0.28 | Cross-platform terminal | Stable |
| jwalk | 0.8 | Parallel directory walk | Well-maintained |
| trash | 5 | Safe file deletion | Community standard |
| clap | 4.5 | CLI argument parsing | Actively maintained |
| dirs | 6.0 | Home directory resolution | Stable |
| anyhow | 1.0 | Error handling | Widely used |

### Dev Dependencies

- **tempfile**: Temporary file fixtures for tests

**Total indirect dependencies**: ~15-20 (managed by Cargo.lock)

## Testing Strategy

### Unit Tests
- Colocated in modules with `#[cfg(test)]`
- Use tempfile for file system fixtures
- Test public APIs and error cases
- Example: `test_detects_claude_directory`

### Integration Tests
- Future: `tests/` directory
- CLI workflow validation
- Multi-platform compatibility checks

### CI/CD Testing
- Runs on macOS latest
- Part of GitHub Actions CI workflow
- Blocking for release readiness

## Code Quality Standards

**Enforced in CI**:
- Format: `cargo fmt -- --check` (enforces rustfmt)
- Lint: `cargo clippy -- -D warnings` (fail on warnings)
- Build: Debug and release profiles
- Test: `cargo test` (all tests must pass)

**Practices**:
- Semantic versioning (MAJOR.MINOR.PATCH)
- Minimal unsafe code (zero in current codebase)
- Preference for iterator chains over loops
- Pattern matching for exhaustive handling
- Error context with anyhow

## Configuration & Metadata

### Cargo.toml Key Fields

```toml
[package]
name = "claudekill"
version = "0.1.0"
edition = "2024"
authors = ["Leo <leo@example.com>"]
description = "Interactive CLI to find and delete .claude folders"
license = "MIT"
repository = "https://github.com/leozqin/claudekill"
readme = "README.md"
keywords = ["cli", "tui", "claude", "cleanup", "disk-space"]
categories = ["command-line-utilities"]
```

### Excluded from Package

```toml
exclude = [".github/*", "plans/*", ".claude/*"]
```

Keeps published crate lean and excludes internal tools.

## Performance Characteristics

- **Startup**: <1 second for typical directories
- **Scanning**: 1-5 seconds for home directory (200-500 .claude dirs)
- **Memory**: <100MB for 1M+ files scanned
- **Deletion**: <1 second per directory to trash
- **Binary size**: ~5-10MB per target (release build with strip)

## Security Model

- **Offline operation**: No network access
- **File system scope**: User-accessible directories only
- **Permission respect**: Respects OS ACLs
- **Safe by default**: Trash instead of permanent deletion
- **User confirmation**: Interactive approval for deletions
- **License**: MIT (permissive, well-understood)

## Future Enhancement Areas

- [ ] Windows support (requires Windows trash API)
- [ ] Configuration file support (.claudekill.toml)
- [ ] Benchmark suite (cargo bench)
- [ ] Fuzzing for input validation
- [ ] CHANGELOG.md automation
- [ ] Architectural Decision Records (ADRs)
- [ ] Watch mode (monitor .claude growth)
- [ ] Size-based filtering (--min-size, --max-size)
- [ ] Pattern-based exclusion
- [ ] Scheduled cleanup automation

## Getting Started for Contributors

1. Clone repository
2. Install Rust (latest stable)
3. Build: `cargo build`
4. Test: `cargo test`
5. Format: `cargo fmt`
6. Lint: `cargo clippy`

See `docs/code-standards.md` for detailed contribution guidelines.

## Release Checklist Summary

**Pre-release**:
- [ ] Update Cargo.toml version
- [ ] Run clippy/fmt/test
- [ ] Manual testing on macOS
- [ ] Commit & push

**Release**:
- [ ] Create annotated git tag
- [ ] Push tag (triggers CI/CD)
- [ ] Verify artifacts on GitHub

**Post-release**:
- [ ] Update Homebrew SHA256
- [ ] `cargo publish`
- [ ] Announce release

See `RELEASING.md` for detailed checklist.
