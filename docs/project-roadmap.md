# Claudekill Project Roadmap

## v0.1.0 (January 2026) - ✓ RELEASED

### Completed Features
- **Core CLI**: Interactive TUI with ratatui for browsing and selecting .claude directories
- **Directory Scanning**: Parallel jwalk-based traversal with streaming results
- **Trash Integration**: Safe deletion via OS trash with fallback permanent delete
- **Project Detection**: Identifies parent project types (.git, Cargo.toml, package.json, pyproject.toml)
- **Multi-platform Support**: macOS (x86_64/ARM64) and Linux (x86_64 GNU)
- **Keyboard Navigation**: Full vim-keybind support (hjkl, arrows, a/n for bulk ops)
- **Distribution**: Cargo/Homebrew/GitHub Releases with CI/CD automation
- **Testing**: 6 unit tests with comprehensive error coverage
- **Documentation**: README, code standards, architecture, and deployment guides

### Metrics
- Binary size: 5-10MB per platform (with strip + LTO)
- Scan performance: <5 seconds for typical home directories
- Test coverage: 100% of trash.rs critical paths
- CI/CD coverage: All 3 target platforms

---

## v0.2.0 (Planned - H1 2026)

### New Features
- **Configuration file support**: `~/.claudekillrc` or `.claudekill.toml` for defaults
  - Set default scan paths
  - Configure global exclusion rules
  - Save user preferences (e.g., always permanent delete)
- **Size-based filtering**: `--min-size`, `--max-size` for selective cleanup
- **Pattern-based exclusion**: Skip directories matching patterns
- **Detailed statistics**: Show total space reclaimed and percentage per project
- **Multiple selections**: Visual feedback for bulk operations with counts
- **Windows support**: Native Windows API for trash integration (research phase)

### Non-functional
- Benchmarking suite with `cargo bench`
- Input validation fuzzing tests
- CHANGELOG.md with semver tracking
- Performance optimization pass

---

## v0.3.0 (Planned - H2 2026)

### New Features
- **Watch mode**: Monitor .claude folder growth over time
  - `--watch` flag for continuous monitoring
  - Alerts when threshold exceeded
  - Automatic cleanup (optional scheduled runs)
- **Exclude by project type**: `--exclude-type rust` to skip specific project types
- **Dry-run improvements**: Show size savings without deletion
- **Colorized output**: Enhanced terminal visuals for better UX
- **Interactive filtering**: Real-time search/filter in TUI
- **Batch operations**: Group by project and delete entire projects at once

---

## Future Considerations (v1.0+)

### Major Features
- **Cloud sync**: Integrate with cloud storage SDKs (AWS S3, Google Drive)
- **Performance profiler**: Show compilation time impact of keeping .claude dirs
- **Analytics**: Track cleanup patterns and suggest optimization
- **Community plugins**: Plugin system for custom project type detection
- **Mobile companion**: iOS/Android app for remote deletion
- **API server**: REST API for programmatic access

### Platform Expansion
- **macOS app bundle**: Native Mac app distribution (.dmg)
- **Windows native**: Full Windows API integration with installer
- **Docker image**: Containerized version for CI/CD pipelines
- **systemd service**: Linux background service for scheduled cleanup

### Maintainability
- Architectural Decision Records (ADRs)
- Fuzzing CI/CD integration
- Performance regression testing
- Security audit and code review

---

## Maintenance Schedule

### Release Cadence
- **Patch releases** (v0.x.y): Monthly security/bug fixes
- **Minor releases** (v0.x.0): Quarterly feature drops
- **Major releases** (v1.0.0): Annual stability milestone

### Support Timeline
- v0.1.x: 6 months of active support
- v0.2.x: 6 months of active support
- v1.0.x: 12 months of LTS support

---

## Community & Contribution Roadmap

- **Homebrew community formula**: Transfer to official tap when ready
- **crates.io trending**: Aim for top 100 CLI utilities
- **GitHub stars**: Target 500+ by end of 2026
- **Contributors**: Build contributor community through clear issues and PRs
- **Documentation translations**: Consider i18n for international users

---

## Known Limitations & Non-Goals

### Won't Implement
- **GUI version**: Stick with CLI/TUI (desktop GUI out of scope)
- **VCS integration**: Won't commit cleanup history
- **Network operations**: No cloud upload or remote management
- **Package manager cleanup**: Focus only on .claude directories
- **System-wide cleanup**: User-scoped operation only

### Acknowledged Limitations
- Windows support blocked pending Windows trash API research
- Performance scales with directory count, not size (typical/acceptable)
- No undo capability once Trash is emptied
- Requires read permissions on scanned directories

---

## Success Criteria

- **v0.1.0**: 100 GitHub stars, active downloads
- **v0.2.0**: Configuration support proven useful via feedback
- **v0.3.0**: Watch mode reduces manual cleanup overhead
- **v1.0.0**: Stable, feature-complete, production-ready
