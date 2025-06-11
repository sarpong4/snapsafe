# Snapsafe

A systems-level tool that enforces fast, secure, and incremental backups with encryption and optional cloud upload. This is a CLI tool that allows users to specify files and directories they want to back up.

## Part 1: Local Backup & Restore

Making local backups of files and directories with encryption and compression. User gets to decide what, when, and how to backup and restore their data.

### Process

- Each backup creates a snapshot that is a record of the state of the files at that time.

        - We only target new files and files with changes
        - We also can restore to a previous snapshot

- The backup data is encrypted using AES-256. Incidentally, the backup data is also compressed to save space.
- Backup version history is not perpetual, but rather limited to a certain number of snapshots (e.g., 10). Old snapshots are deleted when the limit is reached. Right now, the limit is set to 3 snapshots. When the CLI `config` feature is implemented, this limit can be adjusted by the user.

### Features

- **Incremental Backups**: Only new or changed files are backed up after the initial backup.
- **Encryption**: All backups are encrypted using AES-256. Passwords are hashed and stored securely.
- **Compression**: Backups are compressed to save space.
- **File Integrity Check**: Each backup includes a checksum to verify file integrity.
- **Snapshot Management**: Users can view, restore, and delete snapshots.

### Usage

Usage in shell:

```bash

snapsafe backup --source <source> --dest <target>
snapsafe restore --origin <origin> --output <target> 
snapsafe -n <nth> restore --origin <origin> --output <target>
snapsafe delete --origin <origin>
snapsafe delete -n <nth> --origin <origin>
snapsafe list 

```

- After each command, the CLI will prompt for a password to encrypt or decrypt the data.
- The `backup` command creates a new backup of the specified source directory. Each backup strictly enforces the password it was initialized with. This means that when you use a different password, the backup will not be accessible.
- The `restore` command restores files from a specified snapshot version or the latest snapshot version in a backup directory to the target directory.
- The `delete` command removes a specified backup or the latest backup.
- The `list` command displays all available backups.
