# claudekill

> Find and delete `.claude` folders to reclaim disk space.

Like [npkill](https://github.com/voidcosmos/npkill) but for Claude Code cache directories.

## Features

- Fast parallel scanning of home directory
- Interactive TUI with keyboard navigation
- Moves to Trash by default (safe)
- Shows folder sizes and project types
- Excludes global `~/.claude` by default
- Cross-platform (macOS, Linux)

## Installation

### Homebrew (macOS) - Recommended

```bash
brew install olbboy/tap/claudekill
```

**First time setup** (one-time):
```bash
brew tap olbboy/tap
brew install claudekill
```

### Cargo (Rust developers)

```bash
cargo install claudekill
```

**Faster with pre-compiled binaries**:
```bash
cargo install cargo-binstall
cargo binstall claudekill
```

### Binary Release (Manual install)

Download pre-compiled binaries from [GitHub Releases](https://github.com/olbboy/claudekill/releases):

- `claudekill-x86_64-apple-darwin.tar.gz` - macOS Intel
- `claudekill-aarch64-apple-darwin.tar.gz` - macOS Apple Silicon (M1/M2/M3)
- `claudekill-x86_64-unknown-linux-gnu.tar.gz` - Linux

Extract and move to PATH:
```bash
tar xzf claudekill-*.tar.gz
sudo mv claudekill /usr/local/bin/
```

## Usage

```bash
# Interactive mode
claudekill

# Scan specific directory
claudekill --path ~/Projects

# List only (no TUI)
claudekill --dry-run

# Include global ~/.claude
claudekill --include-global

# Permanent delete (skip Trash)
claudekill --permanent
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| ↑/k | Move up |
| ↓/j | Move down |
| Space | Toggle selection |
| a | Select all |
| n | Deselect all |
| d | Delete selected |
| ? | Toggle help |
| q/Esc | Quit |

## Documentation

For detailed information, see:
- [Project Overview & PDR](docs/project-overview-pdr.md) - Phase 5 requirements and goals
- [System Architecture](docs/system-architecture.md) - Technical design and CI/CD pipeline
- [Code Standards](docs/code-standards.md) - Development guidelines and testing
- [Deployment Guide](docs/deployment-guide.md) - Installation and release procedures
- [Codebase Summary](docs/codebase-summary.md) - Project structure and components

## License

MIT License - See [LICENSE](LICENSE) for details.
