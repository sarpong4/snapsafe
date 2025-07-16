use std::fmt;
use std::path::PathBuf;
use std::{io, error};

use crate::crypto::password::PasswordError;

#[derive(Debug)]
pub enum SnapError {
    Command(String),
    Config(String),
    Backup(String),
    Restore(String),
    Delete(String),
    Password(PasswordError),
    IOError(io::Error),
    DirError(walkdir::Error),
    EncryptError(String, Box<dyn error::Error>),
    InvalidCompressor(String),
    InvalidSnapshotLayout(PathBuf),
    SerializationError(String),
}

impl fmt::Display for SnapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapError::Command(msg) => write!(f, "Command Error: {msg}"),
            SnapError::Config(msg) => write!(f, "Config Error: {msg}"),
            SnapError::Backup(msg) => write!(f, "Backup Error: {msg}"),
            SnapError::Restore(msg) => write!(f, "Restore Error: {msg}"),
            SnapError::Delete(msg) => write!(f, "Delete Error: {msg}"),
            SnapError::Password(err) => write!(f, "Password Error: {err:?}"),
            SnapError::IOError(err) => write!(f, "IO Error: {err}"),
            SnapError::DirError(err) => write!(f, "Directory Traversal Error: {err:?}"),
            SnapError::EncryptError(msg, _) => write!(f, "Encryption/Decryption Error: {msg}"),
            SnapError::InvalidCompressor(msg) => write!(f, "Compress/Decompress Error: {msg}"),
            SnapError::InvalidSnapshotLayout(path) => write!(f, "Invalid Snapshot Layout: {:?}", path),
            SnapError::SerializationError(msg) => write!(f, "Serialization Error: {msg}"),
        }
    }
}

impl From<(String, Box<dyn error::Error>)> for SnapError {
    fn from(ctx: (String, Box<dyn error::Error>)) -> Self {
        let msg = ctx.0;
        let err = ctx.1;
        Self::EncryptError(msg, err)
    }
}

impl From<PasswordError> for SnapError {
    fn from(err: PasswordError) -> Self {
        Self::Password(err)
    }
}

impl From<io::Error> for SnapError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<walkdir::Error> for SnapError {
    fn from(err: walkdir::Error) -> Self {
        Self::DirError(err)
    }
}
