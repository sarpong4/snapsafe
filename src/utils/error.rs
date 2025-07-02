use std::{io, error};

use crate::crypto::password::PasswordError;

#[derive(Debug)]
pub enum SnapError {
    CommandError(String),
    Config(String),
    Backup,
    Restore,
    Delete,
    List,
    Password(PasswordError),
    IOError(io::Error),
    DirError(walkdir::Error),
    EncryptError(Box<dyn error::Error>),
}

impl From<Box<dyn error::Error>> for SnapError {
    fn from(err: Box<dyn error::Error>) -> Self {
        Self::EncryptError(err)
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

pub fn get_error(err: SnapError) -> io::Error {
    match err {
        SnapError::CommandError(err) => {
                        eprintln!("Process Failed: {err}");
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "An error occured before the command could complete execution"
                        )
            },
        SnapError::Config(msg) => {
                eprintln!("Config Build Aborted: {msg}");
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "An error occured before the config process could complete"
                )
            },
        SnapError::Backup => {
                eprintln!("Backup Aborted!");
                io::Error::new(
                    io::ErrorKind::NotFound, 
                    format!("An Error occurred during Backup."))
            },
        SnapError::Restore | SnapError::Delete | SnapError::List => {
                eprintln!("Process Aborted!");
                io::Error::new(
                    io::ErrorKind::NotFound, 
                    "No data backup available at specified origin path: Check that your path is correct and password is valid".to_string())
            },
        SnapError::Password(err) => {
                eprintln!("Password Error: {err:?}");
                io::Error::new(
                    io::ErrorKind::InvalidData, 
                    "An error occurred because of your password. Please make sure your password is valid")
            },
        SnapError::IOError(err) => {
                eprintln!("File I/O Error: {err}");
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "An error occurred while parsing a file."
                )
            },
        SnapError::DirError(err) => {
                eprintln!("Path Traversal Error: {err}");
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "An error occurred while traversing the directory."
                )
            }
        SnapError::EncryptError(err) => {
            eprintln!("Encryption/Decryption Error: {err}");
            io::Error::new(
                    io::ErrorKind::InvalidData,
                    "An error occurred during the encryption/decryption process"
                )
        },
    }
}

