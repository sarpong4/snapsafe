use clap::{Parser, Subcommand};
use std::path::Path;

use crate::{actions, config::{build_global_config, build_local_config}, utils::{self, SnapError}};

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


pub fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Config { global: _, local } => {
            let conf_build = if local {
                build_local_config()
            } else {
                build_global_config()
            };

            if let Err(err) = conf_build {
                return Err(Box::new(err));
            }

            return Ok(());
        },
        Commands::Backup { source, target,  comp} => {
            let src = Path::new(&source);
            let dest = Path::new(&target);

            if !src.exists() {
                eprintln!("Source directory does not exist.");
                let err = utils::get_error(SnapError::Command);
                return Err(Box::new(err));
            }

            let config = Some(utils::get_config());

            if let Err(err) = actions::backup(src, dest, comp, config) {
                return  Err(Box::new(err));
            }
            return Ok(());
        },
        Commands::Restore { number, origin, target } => {
            let src = Path::new(&origin);
            let output_dir = Path::new(&target);

            if !src.exists() {
                eprintln!("Directory with expected backed up data does not exist.");
                let err = utils::get_error(SnapError::Command);
                return Err(Box::new(err));
            }

            let restore_return = actions::restore(
                number.unwrap_or(1),
                src, 
                output_dir
            );

            if let Err(err) = restore_return {
                return Err(Box::new(err))
            }
            return Ok(());
        },
        Commands::Delete { number, origin, force} => {
            let target = Path::new(&origin);
            
            if !target.try_exists().unwrap_or(false) {
                eprintln!("Target Directory with expected backed up data does not exist");
                let err = utils::get_error(SnapError::Command);
                return Err(Box::new(err));
            }

            let delete_return = 
                actions::delete(number.unwrap_or(1), target, force);
            if let Err(err) = delete_return {
                return Err(Box::new(err))
            }
            return Ok(());
        }
        Commands::List => {
            let _ = actions::list();
            return Ok(());
        }
    }
}
