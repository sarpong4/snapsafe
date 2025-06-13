# SnapSafe

SnapSafe is a secure, and incremental backups tool with encryption and optional cloud upload. It is designed to protect your files through versioned snapshots and password-based encryption. It is built with simplicity and security in mind and allows you to specify files and directories you want to back-up/restore.

## Features

- 📦 Incremental backup with snapshot versioning
- 🔐 AES-256 encryption with password-derived keys
- 📂 Restore to any snapshot version
- 🧾 List and delete snapshots
- 🧪 Built-in testable architecture

## Installation

```bash
cargo build --release
```

Or add it as a local dependency in your `Cargo.toml`.

## Example

```bash
snapsafe backup --source ~/Documents --dest /mnt/backups
snapsafe restore --origin ./restore --output ./after_restore
```

Refer to [this file](./docs/PART1.md) for more information on command line logic.

## Project Structure

```bash
src/
├── actions             # Core backup/restore actions logic
|  ├── mod.rs
|  ├── backup.rs
|  ├── restore.rs
|  ├── delete.rs
├── commands.rs            # Command-line parsing
├── crytpo.rs           # encryption logic
├── utils             # helper functions and other system logic
|  ├── mod.rs
|  ├── gc.rs            # Garbage Collector to handle version limits
|  ├── registry.rs      # registry management
|  ├── snapshot.rs      # snapshot implementation logic
tests/
```

## Security

SnapSafe encrypts your data using AES-256 in GCM mode. Passwords are processed using PBKDF2 or Argon2 to generate keys securely.

## Roadmap

- ✅ Basic backup/restore/delete
- 🔜 Configurable compression/encryption
- 🔜 Cloud sync (S3/GCS)
- 🔜 Daemon mode for scheduling
