# 📦 Part2: Compression and Version Diffing

The goal here is to implement compression and version diffing for the backup system. This will allow users to save space and track changes between different versions of files. The user will also obtain the capability to build the config files they may need to customize the backup and restore process.

---

## ⚙️ Process

- I will talk about the compression and version diffing process in the context of the backup system.
- I have not decided on the compression algorithm yet. I am thinking to have a default algorithm and when config is implemented, the user can choose a different from the available options.
- Compression will be applied to the backup data to reduce storage space. This will be done transparently, so users will not need to worry about the details of how it works.
- Compression will be applied to the backup data after encryption, ensuring that the data remains secure while still benefiting from reduced storage space.
- Version diffing will be implemented to track changes between different versions of files. This will allow users to see what has changed between backups and restore specific versions if needed.
- When a tracked file is renamed, the system still treats it as the same file, allowing users to restore the file to its previous state even after renaming.
- When a restore on a renamed file occurs at a point in time before the rename, the system will restore the file with its original name. This ensures that users can access previous versions of files even if they have been renamed since the backup was created.
- The compression and version diffing will be implemented in a way that is transparent to the user. The user will not need to worry about the details of how it works, but they will benefit from the reduced storage space and the ability to track changes.

---

## 🧩 Features

- **Compression**: The backup system will use a default compression algorithm to reduce the size of backups. Users can choose a different algorithm when the config feature is implemented.
- **Version Diffing**: The system will track changes between different versions of files, allowing users to see what has changed and restore specific versions if needed.
- **Renaming Support**: The system will treat renamed files as the same file, allowing users to restore previous versions even after renaming.
- **Transparent Implementation**: The compression and version diffing will be implemented in a way that is transparent to the user, ensuring ease of use while still providing powerful features.
- **Configurable Options**: When the config feature is implemented, users will be able to customize the compression algorithm and other settings to suit their needs.

---

## ✅ Usage

See usage in [part1](./PART1.md).

- In addition to the commands in [part1](./PART1.md), users will be able to introduce new flags for compression and version diffing when the config feature is implemented.
- The CLI will handle compression and version diffing automatically, so users will not need to specify anything extra when running backup or restore commands.  
- Users will still be prompted for a password to encrypt or decrypt the data, ensuring that backups remain secure.
- The `backup` command will now also handle compression and version diffing transparently, so users will not need to worry about the details.
- The `restore` command will allow users to restore specific versions of files, taking into account any renaming that has occurred since the backup was created.
- The `delete` command will continue to remove specified backups or the latest backup, and it will also handle any associated compressed data.
- The `list` command will display all available backups, including information about compression and versioning.
- The CLI will provide feedback on the compression and versioning process, allowing users to see how much space has been saved and what changes have been tracked.

Usage in shell:

```bash

snapsafe diff --folder <path>
snapsafe config

```

`snapsafe list` will show the backup history recorded by the registry and user can use the source directory in the `snapsafe diff` command to see the differences between versions of files in that directory.

---
