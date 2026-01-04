# System Architecture

## Overview

Claudekill is a single-binary Rust CLI with modular separation of concerns:
- **Directory scanning**: Parallel traversal with jwalk
- **TUI**: Interactive interface via ratatui
- **Trash integration**: Cross-platform safe deletion
- **CLI handling**: Clap command-line parsing

## Build & Release Pipeline

### Compilation Targets

```
x86_64-apple-darwin    → macOS Intel 64-bit
aarch64-apple-darwin   → macOS ARM64 (M1/M2/M3)
x86_64-unknown-linux-gnu → Linux GNU 64-bit
```

### Release Optimization (Cargo.toml)

```toml
[profile.release]
lto = true              # Link-Time Optimization (slower build, smaller binary)
codegen-units = 1      # Single codegen unit for better optimization
strip = true           # Remove debug symbols
panic = "abort"        # Abort on panic (smaller binary)
```

**Result**: ~5-10MB binary per target (estimated)

### CI/CD Workflow

```
Code Push
    ↓
[CI] Format Check (cargo fmt)
[CI] Linting (cargo clippy)
[CI] Build (cargo build)
[CI] Test (cargo test)
    ↓
Tag Push (v0.x.x)
    ↓
[Release] Multi-target Build
    ├─ macOS x86_64 (macos-latest)
    ├─ macOS ARM64 (macos-latest)
    └─ Linux GNU (ubuntu-latest)
    ↓
[Release] Package (tar.gz)
    ↓
[Release] GitHub Release (auto-generated notes)
```

### Dependency Graph

```
claudekill
├── ratatui 0.29       → TUI rendering
├── crossterm 0.28     → Terminal handling
├── jwalk 0.8          → Parallel directory scanning
├── trash 5            → Safe deletion (Trash integration)
├── clap 4.5           → CLI argument parsing
├── dirs 6.0           → Home directory resolution
└── anyhow 1.0         → Error handling

[dev-dependencies]
└── tempfile 3         → Test fixtures
```

## Distribution Architecture

### 1. Cargo Registry (crates.io)

**Installation**: `cargo install claudekill`

- Published via `cargo publish`
- Metadata: name, version, authors, license, repository, keywords
- Binary install support via `cargo-install` (automatic compilation)

**Packaging Metadata** (Cargo.toml):
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }{ archive-suffix }"
```

Enables `cargo-binstall` to download pre-compiled binaries from GitHub Releases.

### 2. Homebrew (macOS)

**Installation**: `brew install olbboy/tap/claudekill`

**Formula Location**: `homebrew/claudekill.rb` in project repo
(Installed in olbboy/tap tap)

**Formula Structure**:
- Platform detection: ARM64 vs Intel macOS
- SHA256 verification for both architectures
- Binary extraction and installation
- Test validation: `--version` check

**SHA256 Updates**: Manual post-release (from GitHub Releases)

### 3. GitHub Releases

**Installation**: Manual download from releases page

**Artifacts**:
- `claudekill-x86_64-apple-darwin.tar.gz`
- `claudekill-aarch64-apple-darwin.tar.gz`
- `claudekill-x86_64-unknown-linux-gnu.tar.gz`

**Release Notes**: Auto-generated from commit messages

## Data Flow

### Interactive Mode

```
User Input (keyboard)
    ↓
[crossterm] Event handling
    ↓
[ratatui] State update (selection, scroll)
    ↓
[TUI] Render UI
    ↓
Terminal Display
```

### Directory Scanning

```
Start Path (--path or ~)
    ↓
[jwalk] Parallel traversal
    ├─ Find .claude directories
    ├─ Calculate folder sizes
    └─ Detect project types
    ↓
Populate UI list
    ↓
User selects targets
    ↓
[trash] Safe deletion or permanent removal
    ↓
Update UI (remove deleted)
```

## Error Handling Strategy

- **Anyhow Result types**: Propagates errors with context
- **Graceful degradation**: Missing dirs logged, scanning continues
- **TUI error display**: User-friendly error messages in terminal
- **Trash fallback**: Permanent delete if trash fails (with warning)

## Security Considerations

- **No network access**: Fully offline tool
- **File system scope**: Limited to user-accessible directories
- **Trash safety**: Default behavior prevents accidental deletion
- **Permission handling**: Respects file system ACLs

## Scalability Notes

- **jwalk parallelism**: Adapts to CPU cores
- **Memory usage**: Streaming directory listing (not loading entire tree)
- **Large directories**: Tested with deep hierarchies; no known limits
