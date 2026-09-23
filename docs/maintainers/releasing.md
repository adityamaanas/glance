# Release packaging and installation

[Docs home](../README.md) · [Compatibility](../explanation/compatibility.md) · [Changelog](../../CHANGELOG.md)

For maintainers: how releases are built, checked and published.

Glance uses cargo-dist 0.32.0 to build versioned release archives, SHA-256
checksums, and shell and PowerShell installers. The package
and executable are both named `glance-panel`.

## Supported build targets

| Platform | Architecture | Release runner |
| --- | --- | --- |
| Linux (glibc) | x86-64 | Ubuntu 22.04 |
| Linux (glibc) | ARM64 | Ubuntu 22.04 ARM |
| macOS | Apple Silicon | macOS 14 |
| macOS | Intel | macOS 15 Intel |
| Windows | x86-64 | Windows Server 2022 |

The release workflow builds and runs the binary on every listed runner. Older
OS versions, Linux distributions with older glibc, Alpine/musl and native
Windows ARM64 builds are not validated by this matrix. Inspect the release
manifest for runtime requirements. Building from source requires Rust 1.88 or
newer; users of prebuilt archives do not need Rust or SQLite installed.

## Install a published release

The one-line installers always fetch the latest release:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/adityamaanas/glance/releases/latest/download/glance-panel-installer.sh | sh
```

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/adityamaanas/glance/releases/latest/download/glance-panel-installer.ps1 | iex"
```

Open the [releases page](https://github.com/adityamaanas/glance/releases), choose
a version, and download its archive and matching `.sha256` file. Verify the
checksum with `sha256sum` on Linux, `shasum -a 256` on macOS, or
`Get-FileHash -Algorithm SHA256` in PowerShell before extracting the binary.
Put `glance-panel` (or `glance-panel.exe`) in a directory on your PATH.

Once a release produced by this workflow is published, its assets also include
`glance-panel-installer.sh` and `glance-panel-installer.ps1`. Download and inspect
the appropriate script, then run it with `sh` or PowerShell. These installers
select an archive for the host and normally install into Cargo's binary
directory (`$CARGO_HOME/bin`, or `~/.cargo/bin` when unset). They may update PATH.

To install into a specific directory without changing shell profiles or PATH,
set `GLANCE_PANEL_UNMANAGED_INSTALL` to that directory before running the script.
Add that directory to PATH yourself if desired. No background updater is
installed. Installers do not set up agent hooks or invoke summary models.

Verify with `glance-panel --version`. Hook setup is optional and explicit:

```sh
glance-panel setup                  # Claude Code
glance-panel setup --harness cursor # Cursor IDE
```

See [agent transcript formats](../reference/agents.md),
[Follow Cursor](../how-to/cursor.md), and [summary providers](../how-to/summary-providers.md)
for agent-specific configuration and validation limits.

Homebrew is switched off: with the `homebrew` installer enabled, cargo-dist
writes a `brew install` line into every release's notes, which fails without a
tap. To enable it, create the tap repository, add `"homebrew"` back to
`installers` in `dist-workspace.toml` together with `tap` and
`publish-jobs = ["homebrew"]`, configure the tap's publishing token, and run
`dist generate`.

## Validate packaging

Install the pinned cargo-dist version, then run:

```sh
dist generate --check
dist plan
dist build --artifacts=local --target=<host Rust target triple>
dist build --artifacts=global
TARGET=<host Rust target triple> python3 tests/release_smoke.py target/distrib
cargo publish --locked --dry-run
```

On Windows use `$env:TARGET = 'x86_64-pc-windows-msvc'` and `python` for the
smoke command. Python 3.12 or newer is required. The smoke test verifies the
archive checksum, safely extracts it, starts a loopback-only artifact server,
runs the real native installer into a temporary directory containing spaces,
and exercises version reporting, a synthetic Codex transcript and personal
todo creation through both binaries. Agent settings are checked for changes;
no authentication or model calls are needed.

PRs build artifacts and run these checks without creating a GitHub release.
The smoke workflow regenerates installers from the same local artifacts and
release tag as the global build; successful smoke checks gate release upload.

## Publish a release

1. Merge the reviewed changes, update the version and changelog in a focused
   PR, and let CI pass. This packaging change does not pick the next version.
2. Run `dist generate --check`, inspect `dist plan`, and validate the crate with
   `cargo publish --locked --dry-run`.
3. Push a matching version tag, such as `v0.3.0`, from the reviewed commit.
   The Release workflow builds five target archives, verifies installation and
   creates the GitHub release. A prerelease suffix produces a prerelease.
4. If publishing to crates.io, run the separate **Publish crate** workflow with
   that existing tag. Leave `publish` false for a dry run first. The workflow
   checks the tag against Cargo.toml and requires the commit to be on `main`.
5. Configure a scoped `CARGO_REGISTRY_TOKEN` in the `crates-io` GitHub environment
   and enable that environment's required reviewers before running with
   `publish` true. Confirm ownership/availability of `glance-panel` on crates.io.

The crate workflow never publishes merely because a tag was pushed. Neither
the token, environment approval rules nor a Homebrew tap are created by this
repository. Re-running a published version does not replace an existing crate;
fix a bad release with a new version. To change generated CI, edit
`dist-workspace.toml` and run `dist generate` rather than editing `release.yml`.
