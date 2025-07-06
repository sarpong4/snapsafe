use clap::{Parser, Subcommand};
use std::path::Path;

use crate::{actions, utils::{self, error::SnapError}};

#[derive(Parser)]
#[command(name = "snapshot", version = "1.0", about = "A secure backup and restore tool.", after_help = "Strict password enforcement:\n\
             - Backups are bound to the password used during creation.\n\
             - If a different password is provided for the same destination, the operation will fail.\n\
             - This is to prevent accidental overwrite or mismatched encryption keys.\n\
             - To change the password in the future, use a planned `snapsafe rekey` command.")]
pub struct CLI {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    /// generate the process to build a config file for your system.
    /// you have the option to build a local config and global config, add it to your command
    Config {
        #[arg(short = 'g', long, required = false)]
        global: bool,
        #[arg(short = 'l', long, required = false)]
        local: bool,
    },
    /// use this to create a backup of a folder in some destination folder: `snapsafe backup --help` for usage info
    Backup {
        #[arg(short = 's', long = "source", required = true)]
        source: String, 
        #[arg(short = 'd', long = "dest", required = true)]
        target: String,
        #[arg(short = 'c', long = "comp", required = false)]
        comp: Option<String>
    },
    /// use this to restore backup at a certain origin to an output directory: `snapsafe restore --help` for usage info
    Restore {
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(long, required = true)]
        origin: String,
        #[arg(short = 'o', long = "output", required = true)]
        target: String,
    },
    /// use this to delete the latest backup or the nth backup where 1 is the latest: `snapsafe delete --help` for usage info
    Delete{
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(short = 'o', long, required = true)]
        origin: String,
        #[arg(long)]
        force: bool
    },
    /// use this to list all backups a user has made: `snapsafe list`
    List, 
}


pub fn entry() -> Result<(), SnapError> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Config { global: _, local } => {
            let _ = actions::config(local)?;
        },
        Commands::Backup { source, target,  comp} => {
            let src = Path::new(&source);
            let dest = Path::new(&target);

            if !src.exists() {
                let err = SnapError::Command("Source directory does not exist".into());
                return Err(err);
            }

            let config = Some(utils::get_config());

            actions::backup(src, dest, comp, config)?;
        },
        Commands::Restore { number, origin, target } => {
            let src = Path::new(&origin);
            let output_dir = Path::new(&target);

            if !src.exists() {
                let message = "Directory with expected backed up data does not exist.";
                let err = SnapError::Command(message.into());
                return Err(err);
            }

            actions::restore(number.unwrap_or(1), src, output_dir)?;
        },
        Commands::Delete { number, origin, force} => {
            let target = Path::new(&origin);
            
            if !target.try_exists().unwrap_or(false) {
                let message = "Target Directory with expected backed up data does not exist";
                let err = SnapError::Command(message.into());
                return Err(err);
            }

            actions::delete(number.unwrap_or(1), target, force)?;
        },
        Commands::List => {
            let _ = actions::list();
        }
    }
    Ok(())
}
