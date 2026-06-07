# Zed Config Sync

A CLI tool for syncing your [Zed](https://zed.dev) editor settings across multiple machines via GitHub Gist. Push, pull, browse history, and roll back to any saved version.

## Installation

**Prerequisites:** Rust toolchain

```bash
git clone <repo-url>
cd zed-config-sync
cargo build --release
```

The binary is `target/release/zed-config` (`zed-config.exe` on Windows). Place it somewhere on your `PATH` for convenient access.

## Configuration

### If you already have Zed but don't have a settings Gist yet

Create a GitHub Personal Access Token with the `gist` permission scope. Then create a new Gist (either empty or with a placeholder file) and copy its ID from the URL.

```bash
# Save your token (validated against GitHub API before saving)
zed-config set token ghp_xxxxxxxxxxxx

# Save the Gist ID
zed-config set gist <your-gist-id>

# Upload your current settings
zed-config push
```

At this point your settings are backed up. On another machine, follow the steps below.

### If you're setting up a fresh Zed and want to pull your existing settings

Ensure you have your GitHub token and Gist ID at hand.

```bash
zed-config set token ghp_xxxxxxxxxxxx
zed-config set gist <your-gist-id>
zed-config pull
```

This will download your latest config snapshot and apply it to the local Zed config directory.

## Usage

### Push

Upload your current local settings to the Gist. The tool keeps the last 5 versions automatically.

```bash
zed-config push
```

### Pull

Download the latest settings from your Gist and apply them locally. Existing local files are overwritten or added; nothing is deleted.

```bash
zed-config pull
```

### History

See all saved versions, when they were created, and which machine uploaded them.

```bash
zed-config history
```

Example output:

```
2025-06-01_09:00    DESKTOP-A1B2C3
2025-06-03_14:30    LAPTOP-XYZ
```

### Restore

Roll back to a specific version. Use the date from `history` output as the identifier.

```bash
zed-config restore 2025-06-03_14:30
```

### View current config

```bash
zed-config token   # Show saved token (or "not set")
zed-config gist    # Show saved Gist ID (or "not set")
zed-config version # Show tool version
```

## How It Works

- Config snapshots are stored as Base64-encoded zip files in your Gist, named `cfg_YYYY-MM-DD_HH-MM`.
- A `history.json` file in the same Gist tracks which machine uploaded each snapshot.
- Local credentials (`token` and `gist_id`) are saved in a `config.json` file alongside the binary.
- `push` merges your local files on top of the latest cloud snapshot before uploading, so you never lose changes made on another machine.

## License

MIT
