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

## Reliability & Failure Modes

SnapSafe is designed to be resilient against common risks in backup systems, including partial state, corruption, and inconsistent behavior.

### 1. File Corruption & Crashes

**Concern:** If the system crashes during backup, can it leave a corrupted snapshot?

**Mitigation:**

- Snapshots are atomic: either the entire snapshot is written successfully, or it is not written at all.
- Snapshots are written to a **staging directory** and only moved into the final location once all files are written and validated.
- Incomplete snapshots are automatically detected and cleaned on startup.
- Manifest includes hash for each file — used to validate post-write integrity.

### 2. Encryption Correctness

**Concern:** One wrong byte makes restoration impossible.

**Mitigation:**

- Each snapshot manifest is encrypted with a user-provided password.
- The password is used to derive a key via **Argon2id** (or PBKDF2).
- The manifest includes metadata about the encryption method and checksum.
- Encrypted payloads use **AES-GCM**, which provides both encryption and authentication (integrity check).
- If decryption fails due to tampering or corruption, the operation aborts.
- Each snapshot can optionally include a checksum of the decrypted manifest for additional validation.

### 3. Chunking Logic (For Future Deduplication)

**Concern:** Inconsistent chunk boundaries break deduplication.

**Mitigation:**

- Chunking is not yet implemented, but future designs will ensure deterministic chunking.
- Chunking logic will be clearly defined and deterministic (e.g., Rabin fingerprinting or fixed-size blocks).
- For now, full-file snapshot is used, but the design is chunk-aware.
- Tests will cover round-trip chunking + re-chunking stability.

### 4. Restore Safety

**Concern:** What happens if restore fails halfway?

**Mitigation:**

- Restores are designed to be atomic: either the entire restore completes successfully, or it does not.
- Restore is done into a **temporary directory** and only moved to final target if fully successful.
- If restore fails, the partial directory is deleted or left isolated with a `.failed` suffix.
- Future versions may support atomic overlay restores or journaling.

### 5. Idempotent Behavior

**Concern:** Re-running a backup should not re-process unchanged files.

**Mitigation (Planned):**

- SnapSafe will implement a version diffing mechanism to detect changes.
- Files will be compared against the last snapshot to determine if they need re-processing.
- Files will be hashed before processing.
- If snapshot metadata matches previous snapshot, file reuse will be skipped.
- Configurable deduplication based on file content hash.

### 6. Metadata Integrity

**Concern:** What if `.snapsafe/registry.json` is corrupted?

**Mitigation (Planned):**

- Metadata is stored in a separate file (`backup_registry.json`) within each snapshot directory.
- Each snapshot directory contains its own manifest, which is self-contained.
- Metadata is versioned and backed up within each snapshot directory (`manifest.json`).
- If the main index is lost, SnapSafe can reconstruct metadata by scanning snapshot folders.
- Each manifest is self-describing and independent.

### 7. User Error Prevention

**Concern:** Users accidentally deleting snapshots or restoring wrong versions.

**Mitigation:**

- Commands require explicit confirmation for destructive actions (e.g., `--force` flag).
- SnapSafe will prompt for confirmation before deleting snapshots.
- Restores will require the user to specify the exact version or origin directory.
- SnapSafe will provide a `--dry-run` option to simulate actions without making changes. (future)

## Future Work

- **Diff-based Incremental Backups**
- **Configurable Backup Policies**
- **Remote Storage Support (S3, GCS, WebDAV)**
- **Snapshot Scheduling Integration**
- **System Service Mode (daemon)**

---
