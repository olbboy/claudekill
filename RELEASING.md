# Release Checklist

## Pre-release

- [ ] Update version in Cargo.toml
- [ ] Update CHANGELOG.md (if exists)
- [ ] Run `cargo clippy` - no warnings
- [ ] Run `cargo test` - all pass
- [ ] Test on macOS manually
- [ ] Commit: "chore: prepare release v0.x.x"

## Release

- [ ] Tag: `git tag -a v0.x.x -m "Release v0.x.x"`
- [ ] Push: `git push origin v0.x.x`
- [ ] Wait for GitHub Actions to complete
- [ ] Verify binaries download and run

## Post-release

- [ ] Update Homebrew formula SHA256
- [ ] Publish to crates.io: `cargo publish`
- [ ] Announce on social media (optional)
