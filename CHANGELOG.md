# Changelog

All notable changes to this project will be documented in this file.

## [v0.5.0] - 2026-01-05

### Added
- **Search & Filter**: Real-time search functionality to filter directories by name.
- **Sort Options**: Sort results by name, size, or modification date.
- **Configuration**: Persistent settings via TOML config file (`~/.config/claudekill/config.toml`).
- **History Management**: Track and recall previously scanned directories.
- **Page Navigation**: Paginated list view with `PgUp`/`PgDn` support.
- **Windows Compatibility**: Platform-specific path handling and improved cross-platform support.

### Changed
- Added new dependencies: `toml`, `serde`, `serde_json`, `directories`, `chrono`.
- DRY refactor for improved code maintainability.
- Updated Rust edition to 2021.

### Fixed
- Homebrew formula SHA256 checksums.
- Binary artifact distribution.

## [v0.1.1] - 2026-01-05

### Changed
- **Branding**: Complete aesthetic overhaul to "Retro Punk / Cyberpunk".
- **Documentation**: New `README.md` with ASCII art, gradient logo, and badge system.
- **Assets**: Added `docs/logo.svg`.

## [v0.1.0] - 2026-01-04

### Added
- Implement claudekill CLI with TUI for `.claude` folder cleanup (`a332ee4`)
- Add distribution packaging (`999800c`)

### Fixed
- Update GitHub username to olbboy (`1f3b711`)
