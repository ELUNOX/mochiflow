# Release verification

Every MochiFlow release ships checksums and SLSA build provenance.

Pull requests run `dist plan` only when release configuration, Cargo package
metadata, or the Rust toolchain changes. Publication runs only for release tags,
after the tag commit is verified reachable from `origin/main` and its version
matches the Cargo workspace. The generated release workflow is retained from
cargo-dist 0.32.0; repository-owned hardening is limited to the trigger,
least-privilege permissions, locked installation, and provenance preflight.

Linux CI remains the complete pre-merge gate for tests, formatting, Clippy,
freeze, and cargo-deny. A separate read-only macOS job runs the Rust tests only
after relevant CLI, Cargo, or toolchain changes reach `main`, or by manual
dispatch; it does not run for pull requests or on a schedule.

## Check the archive

Verify the SHA256 checksum for the archive you downloaded:

```bash
shasum -a 256 -c mochiflow-cli-<target>.tar.xz.sha256
```

## Verify provenance

Use the GitHub CLI to verify the release attestation:

```bash
gh attestation verify mochiflow-cli-<target>.tar.xz --repo ELUNOX/mochiflow
```

## macOS quarantine

Binaries installed with Homebrew or the shell installer run without a Gatekeeper
prompt. If you download a release tarball with a web browser and macOS blocks
the extracted binary, clear the quarantine flag once:

```bash
xattr -d com.apple.quarantine ./mochiflow
```
