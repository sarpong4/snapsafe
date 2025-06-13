# SnapSafe – Secure Incremental Backup Tool

## Table of Contents

1. [Overview](#overview)
2. [Goals & Non-Goals](#goals--non-goals)
3. [System Architecture](#system-architecture)
4. [Core Components](#core-components)
5. [Data Structures](#data-structures)
6. [Security Considerations](#security-considerations)
7. [CLI Interface](#cli-interface)
8. [Extensibility Plan](#extensibility-plan)
9. [Testing Strategy](#testing-strategy)
10. [Future Work](#future-work)

---

## Overview

**SnapSafe** is a command-line tool for secure, [incremental backups](PART1.md). It allows users to back up directories, restore them, list available snapshots, and delete old backups. The system prioritizes **data integrity**, **security**, and **safe-by-default behavior**.

**Current Capabilities:**

- Backup directories with snapshot versioning
- Restore a specific snapshot to its original directory or some defined directory
- List and delete snapshots
- Password-based encryption
- Basic metadata registry for snapshots

---

## Goals & Non-Goals

### Goals

- Provide a minimal and secure backup tool with intuitive commands
- Ensure encrypted backups with verifiable integrity
- Enforce strict password per backup action (User cannot use a password different from the password used at first backup call.)
- Enable versioned snapshot creation and restoration
- Lay a foundation for configurable diffing and storage options

### Non-Goals

- Real-time file sync
- GUI interface
- Distributed storage (at this stage)
- Comprehensive backup scheduling (cron integration is left to the user)

---

## System Architecture

```txt
+--------------------+
|      CLI Frontend  |
+--------+-----------+
         |
         v
+--------+------------+      +-----------------+
| Snapshot Manager    | <--> | Registry Store  |
+---------------------+      +-----------------+
         |
         v
+---------------------+
| File Walker & Hash  |
+---------------------+
         |
         v
+----------------------+
| Compressor & Encrypt |
+----------------------+
         |
         v
+----------------------+
|   Filesystem Writer  |
+----------------------+
```

- **CLI Frontend:** Parses user commands (backup, restore, list, delete)
- **Snapshot Manager:** Orchestrates snapshot lifecycle, interfaces with registry
- **Registry Store:** Local snapshot metadata index
- **File Walker & Hash:** Detects file changes (for future diff support)
- **Compressor & Encrypt:** Handles compression and AES encryption
- **Filesystem Writer:** Writes snapshot contents to target location

---

## Core Components

### 1. SnapshotManager

- `create_snapshot(source, dest, password)`
- `restore_snapshot(version, dest, password)`
- `list_snapshots()`
- `delete_snapshot(version)`

### 2. Registry

Stores metadata such as:

```json
{
  "snapshots": [
    {
      "id": "20250612_1730",
      "created_at": "2025-06-12T17:30:00Z",
      "files_count": 43,
      "encrypted": true
    }
  ]
}
```

### 3. Encryptor

- Uses AES-256 with password-derived keys via PBKDF2 or Argon2
- Zeroes out keys in memory after use (where possible)

### 4. File Handler

- Recursive file collector and hasher
- Prepares files for compression and encryption
- Verifies integrity on restore

---

## Data Structures

### Snapshot Layout (on disk)

```bash
.snapsafe/
|── backup_registry.json
```

### `backup_registry.json` example

```json
{
  "files": [
    { 
        "timestamp": "2025-06-12T17:30:00Z",
        "origin_path": "docs/notes.txt", 
        "backup_path": "backups/notes",  
        "hash": "abc123", 
        "snapshots": 512 
    }
  ],
}
```

---

## Security Considerations

- **Encryption:** AES-GCM (authenticated encryption)
- **Key Derivation:** Argon2id with user-supplied password
- **No plaintext leak:** Intermediate files are not persisted
- **Config hardening:** Defaults enforce encryption, future versions may allow opt-out with explicit flags

---

## CLI Interface

### Commands

```bash
snapsafe backup --source <source> --dest <dest>
snapsafe restore --number <version> --origin <dest> or snapsafe restore --orign <dest>
snapsafe list
snapsafe delete --number <version> --origin <dest> [--force] or snapsafe delete --origin <dest> [--force]
```

### Example

```bash
snapsafe backup --source ~/Documents --dest /mnt/backups
snapsafe restore --origin /mnt/backups
```

Each of the above examples prompts the user for their password. [Part 1](PART1.md)

---

## Extensibility Plan

### Planned Additions

| Feature | Purpose |
|--------|---------|
| Config File | compression level, encryption toggle, version limit |
| Diff Mode | Avoid re-backing unchanged files |
| Remote Target | Upload snapshot to S3 or GCS |
| Backup Tags | Easier snapshot retrieval by name |
| Compression Options | zstd, brotli support |

---

## Testing Strategy

- **Unit tests** for all modules: snapshot manager, registry, encryptor, file walker
- **Integration tests** for end-to-end commands
- **Property-based tests** (future) for file restoration fidelity
- **Fuzzing** input paths and corrupt manifests

---

## Future Work

- **Diff-based Incremental Backups**
- **Configurable Backup Policies**
- **Remote Storage Support (S3, GCS, WebDAV)**
- **Snapshot Scheduling Integration**
- **System Service Mode (daemon)**

---
