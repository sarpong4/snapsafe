# SnapSafe Configuration Guide

SnapSafe is safe-by-default and requires no configuration to get started. However, future versions will support a user-defined configuration file to customize behavior.

## Planned Configuration Options

```toml
# snapsafe.toml

[general]
snapshot_dir = "/mnt/backups/snapshots"
registry_dir = "/mnt/backups/.snapregistry"
compression = "zstd"  # or "brotli", "none"
encryption = true

[security]
encryption_algorithm = "aes-gcm"
key_derivation = "argon2id"
iterations = 100_000

[diff]
enabled = true
hash_algorithm = "sha256"

[cloud]
enabled = false
provider = "aws"
bucket = "my-snapsafe-backups"
```

## Default Behavior (When No Config Is Present)

- Snapshots stored in `<dest>/snapshots/`
- Registry stored in `$HOME/.snapsafe/`
- AES-256 encryption enabled by default
- No compression
- No remote uploads

## Config File Location

SnapSafe will look for a config file in the following order:

1. `--config <option>` (explicitly provided)
2. `./snapsafe.toml` (in current directory)
3. `$HOME/.snapsafe/.snapsafe.toml` (user config)

## Overriding via CLI

CLI flags (e.g., `--no-encrypt`) will always override config file settings.

## Notes

Configuration support is under active development. Stay tuned for updates!
