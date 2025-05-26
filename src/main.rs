use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, env, hash::{DefaultHasher, Hash, Hasher}, iter::Peekable, process::ExitCode};


type FileToHash = HashMap<String, String>;

// You can initialize file_to_hash inside a function when needed

#[derive(Parser)]
#[command(name = "snapshot", version = "1.0")]
struct CLI {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Backup {
        source: String,
        #[arg(short = 'd', long = "dest")]
        target: String,
        #[arg(short = 'p', long, value_parser = hash_password)]
        password: String,
    },
    Restore {
        dest: String,
        #[arg(short = 's', long = "snapshot")]
        snapshot_id: u8,
        #[arg(short = 'o', long = "output")]
        target: String,
        #[arg(short = 'p', long, value_parser = hash_password)]
        password: String
    },
    List,
}

fn hash_password(password: &str) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    Ok(format!("{:x}", hasher.finalize()))
}

fn verify_hash(file_path: &str, hashed: &str, file_to_hash: FileToHash) -> Result<(), String> {
    match file_to_hash.get(file_path) {
        Some(stored_hash) => {
            if hashed == stored_hash {
                Ok(())
            } else {
                Err("Invalid password".to_string())
            }
        }
        None => {
            eprintln!("ERROR: Files in {file_path} have not been backed up");
            Err("File not found in backup".to_string())
        }
    }
}


fn usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("         snapsafe backup <source> --dest <target> --password <pwd>");
    eprintln!("         snapsafe restore <target> --dest <source> --password <pwd>");
    eprintln!("         snapsafe list")
}

fn entry() -> Result<(), ()> {
    let mut args = env::args();
    let program = args.next().expect("Path to program is provided");


    let subcommand = args.next().ok_or_else(|| {
        let _ = usage(&program);
        eprintln!("ERROR: no subcommand provided");
    })?;

    match subcommand.as_str() {
        "backup" => {
            println!("We are backing up");
            // we will need the target, the destination, and optional password
            let target = args.next().ok_or_else(|| {
                let _ = usage("backup");
                eprintln!("ERROR: no target file provided");
            })?;

            let dest = args.next().ok_or_else(||{
                let _ = usage("backup");
                eprintln!("ERROR: destination file not provided");
            })?;

            let passwd = match args.next() {
                Some(pwd) => pwd,
                None => String::new()
            };
        }
        _ => {
            println!("This is the subcommand: {subcommand}");
            // let _ = &usage(&program);
            // eprintln!("ERROR: unknown subcommand {subcommand:?}");
            // return Err(());
        }
    }


    Ok(())
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE, 
    }
}
