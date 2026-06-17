# Release verification

Every MochiFlow release ships checksums and SLSA build provenance.

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
