# Deployment & Distribution Guide

## Installation Methods

### 1. Cargo (Recommended for Rust developers)

```bash
cargo install claudekill
```

**How it works**:
- Downloads source from crates.io
- Compiles locally on your machine
- Installs binary to `~/.cargo/bin/`
- Requires Rust toolchain installed

**When to use**: Rust development environment available

### 2. Homebrew (Recommended for macOS users)

```bash
brew install olbboy/tap/claudekill
```

**How it works**:
- Uses pre-compiled macOS binaries
- Supports both Intel (x86_64) and Apple Silicon (ARM64)
- Installs to `/usr/local/bin/` or `/opt/homebrew/bin/`
- Zero compilation required

**Setup** (one-time):
```bash
# Add tap (alternative: done automatically on install)
brew tap olbboy/tap
```

**Verification**:
```bash
which claudekill
claudekill --version
```

**When to use**: macOS user without Rust toolchain

### 3. GitHub Releases (For all platforms)

**Manual download**:
1. Visit https://github.com/olbboy/claudekill/releases
2. Download binary for your platform:
   - `claudekill-x86_64-apple-darwin.tar.gz` (Intel macOS)
   - `claudekill-aarch64-apple-darwin.tar.gz` (Apple Silicon)
   - `claudekill-x86_64-unknown-linux-gnu.tar.gz` (Linux)
3. Extract: `tar xzf claudekill-*.tar.gz`
4. Move to PATH: `sudo mv claudekill /usr/local/bin/`

**When to use**: Manual installation, CI/CD pipelines, no package manager

### 4. cargo-binstall (Optional speedup for Cargo)

```bash
cargo binstall claudekill
```

**Benefit**: Downloads pre-compiled binary instead of compiling
**Requirement**: `cargo-binstall` installed (`cargo install cargo-binstall`)

## Release Process

### Pre-Release Checklist

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # From 0.1.0
   ```

2. **Run quality checks**:
   ```bash
   cargo fmt            # Format code
   cargo clippy         # Lint check
   cargo test           # Run tests
   cargo build --release  # Final build
   ```

3. **Manual testing** on macOS:
   ```bash
   ./target/release/claudekill --help
   ./target/release/claudekill --dry-run
   ```

4. **Commit** release preparation:
   ```bash
   git add Cargo.toml
   git commit -m "chore: prepare release v0.2.0"
   git push origin main
   ```

### Release Execution

1. **Create annotated git tag**:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   ```

2. **Push tag** (triggers CI/CD):
   ```bash
   git push origin v0.2.0
   ```

3. **Monitor GitHub Actions**:
   - Visit `.github/workflows/release.yml`
   - Wait for all matrix builds to complete (typically 5-10 min)
   - Verify artifacts uploaded to GitHub Releases

### Post-Release Steps

1. **Update Homebrew formula** (`homebrew/claudekill.rb`):

   Get SHA256 hashes from GitHub Releases artifacts:
   ```bash
   shasum -a 256 claudekill-aarch64-apple-darwin.tar.gz
   shasum -a 256 claudekill-x86_64-apple-darwin.tar.gz
   ```

   Update formula:
   ```ruby
   class Claudekill < Formula
     version "0.2.0"

     on_macos do
       on_arm do
         sha256 "ACTUAL_SHA256_FOR_ARM64"  # Replace PLACEHOLDER
       end
       on_intel do
         sha256 "ACTUAL_SHA256_FOR_X64"   # Replace PLACEHOLDER
       end
     end
   end
   ```

2. **Publish to crates.io**:
   ```bash
   cargo publish
   ```

   Verify on https://crates.io/crates/claudekill

3. **Announce release** (optional):
   - GitHub Releases page (auto-generated notes)
   - Social media / community forums
   - Email notification to users

## CI/CD Pipeline Details

### GitHub Actions: CI Workflow

**Trigger**: Push to main/master, Pull requests

**Steps**:
1. Checkout code
2. Install Rust toolchain (stable + clippy, rustfmt)
3. Format check: `cargo fmt -- --check`
4. Lint: `cargo clippy -- -D warnings`
5. Build: `cargo build`
6. Tests: `cargo test`

**Platform**: macOS latest

**Failure handling**: Blocks merge to main

### GitHub Actions: Release Workflow

**Trigger**: Git tag push matching `v*` pattern

**Matrix builds** (parallel):

| OS | Target | Purpose |
|----|--------|---------|
| macOS latest | x86_64-apple-darwin | Intel Macs |
| macOS latest | aarch64-apple-darwin | Apple Silicon Macs |
| Ubuntu latest | x86_64-unknown-linux-gnu | Linux |

**Steps per target**:
1. Checkout code
2. Install Rust with target triple
3. Build release binary: `cargo build --release --target <target>`
4. Package: `tar czvf claudekill-<target>.tar.gz claudekill`
5. Upload artifact to GitHub

**Release job** (runs after all builds):
1. Download all artifacts
2. Create GitHub Release with artifacts
3. Auto-generate release notes from commits

**Artifacts uploaded**:
- `claudekill-x86_64-apple-darwin.tar.gz`
- `claudekill-aarch64-apple-darwin.tar.gz`
- `claudekill-x86_64-unknown-linux-gnu.tar.gz`

## Troubleshooting

### Installation Issues

**Problem**: `cargo install claudekill` fails with "crate not found"
- **Cause**: Package not yet published to crates.io
- **Solution**: Run `cargo publish` in release workflow

**Problem**: Homebrew install fails with "Formula not found"
- **Cause**: Tap not properly configured
- **Solution**:
  ```bash
  brew tap olbboy/tap
  brew install claudekill
  ```

**Problem**: Binary from GitHub Releases won't execute
- **Cause**: Quarantine attribute set by macOS
- **Solution**:
  ```bash
  xattr -d com.apple.quarantine ./claudekill
  ```

### Release Failures

**Problem**: GitHub Actions release workflow fails on build
- **Check**: CI workflow passed
- **Check**: Rust version compatibility
- **Check**: All dependencies available on stable

**Problem**: Release artifacts missing from GitHub
- **Check**: Tag format is `v*` (e.g., `v0.1.0`)
- **Check**: Permissions allow `contents: write`
- **Re-run**: Manual GitHub Actions trigger

## Version Management

### Semantic Versioning

```
MAJOR.MINOR.PATCH
  ↓      ↓       ↓
  0      1       0
```

- **MAJOR**: Breaking changes (rare)
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes

Examples:
- `0.1.0` → Initial release
- `0.2.0` → New feature (non-breaking)
- `0.2.1` → Bug fix
- `1.0.0` → Stable release

### Updating in Files

1. `Cargo.toml` - Required for publish
2. `Cargo.lock` - Auto-updated by cargo
3. `homebrew/claudekill.rb` - Manual update
4. Git tag - Created from Cargo.toml version
5. `RELEASING.md` - Update checklist version examples

## Platform-Specific Notes

### macOS (Intel & Apple Silicon)

- **Binary size**: ~12MB each
- **Homebrew**: Primary distribution method
- **Signing**: Not required for unsigned distribution
- **Minimum OS**: 10.13+ (via Homebrew; binary may support newer)

### Linux

- **Glibc requirement**: 2.17+ (Ubuntu 16.04+, CentOS 7+)
- **Distribution**: Via GitHub Releases (no package manager)
- **Binary size**: ~10MB

### Windows

- **Status**: Not currently supported
- **Reason**: Uses POSIX-specific APIs (trash, paths)
- **Future**: Requires Windows trash API integration

## Security Considerations

- **Binary verification**: SHA256 hashes in Homebrew formula
- **Source verification**: GitHub's built-in tag signing
- **Update frequency**: Monitor dependency security advisories
- **HTTPS**: All download URLs use HTTPS

## v0.1.0 Release Status

### Release Date
January 4, 2026

### Distribution Status
- ✓ GitHub Releases: Pre-compiled binaries available
- ✓ Homebrew: Formula ready at `olbboy/tap/claudekill`
- ✓ Cargo: Published to crates.io
- ✓ cargo-binstall: Metadata configured for pre-compiled downloads

### Validated Installation Methods
All three installation channels tested and working:
1. `cargo install claudekill` (compiles locally)
2. `brew install olbboy/tap/claudekill` (macOS only)
3. Manual binary download from GitHub Releases

### Multi-platform Binaries Available
- macOS x86_64 (Intel): claudekill-x86_64-apple-darwin.tar.gz
- macOS ARM64 (M1/M2/M3): claudekill-aarch64-apple-darwin.tar.gz
- Linux x86_64: claudekill-x86_64-unknown-linux-gnu.tar.gz

### Test Results
- CI workflow: ✓ Pass (fmt, clippy, build, test)
- Release workflow: ✓ Pass (all 3 platforms)
- Manual testing: ✓ Pass (macOS Intel & ARM64)

## Maintenance Schedule

- **Weekly**: Monitor CI/CD pipeline health
- **Monthly**: Review dependency updates
- **Per-release**: Follow pre/post-release checklists
- **Ad-hoc**: Security patches for critical vulnerabilities

## v0.1.0 → v0.2.0 Upgrade Path

Future releases will maintain backward compatibility with existing CLI flags.

### Expected Timeline
- v0.2.0: Q2 2026 (configuration support + size filtering)
- v0.3.0: Q4 2026 (watch mode + pattern-based exclusion)
- v1.0.0: End 2026 (stable, feature-complete)
