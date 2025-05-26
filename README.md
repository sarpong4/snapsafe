# snapsafe

SnapSafe Challenge: A systems-level tools that enforces fast, secure, and incremental backups with encryption and optional cloud upload. This is a CLI tool that allows users to specify files and directories to back up.

## Part 1: Local Backup & Restore

Making local backups of files and directories with encryption and compression. User gets to decide what, when, and how to backup and restore their data.

### Process

- Each backup creates a snapshot that is a record of the state of the files at that time.

        - We only target new files and files with changes
        - We also can restore to a previous snapshot

- The backup data is encrypted using AES-256. Incidentally, the backup data is also compressed to save space.

### Features

- **Incremental Backups**: Only new or changed files are backed up after the initial backup.
- **Encryption**: All backups are encrypted using AES-256. Passwords are hashed and stored securely.
- **Compression**: Backups are compressed to save space.
- **File Integrity Check**: Each backup includes a checksum to verify file integrity.
- **Snapshot Management**: Users can view, restore, and delete snapshots.
- **Configuration**: Users can specify which files and directories to back up, as well as backup frequency and retention policies.

### Usage

Usage in shell:

```bash

snapsafe backup <source> --dest <target> --password <passwd>
snapsafe restore <dest> --snapshot <snapshot_id> --output <target> --password <passwd>
snapsafe list
    
```
