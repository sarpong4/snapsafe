use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod actions;
pub mod common;

// You can initialize file_to_hash inside a function when needed

#[derive(Parser)]
#[command(name = "snapshot", version = "1.0", about = "A secure backup and restore tool.", after_help = "Strict password enforcement:\n\
             - Backups are bound to the password used during creation.\n\
             - If a different password is provided for the same destination, the operation will fail.\n\
             - This is to prevent accidental overwrite or mismatched encryption keys.\n\
             - To change the password in the future, use a planned `snapsafe rekey` command.")]
struct CLI {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// use this to create a backup of a folder in some destination folder: `snapsafe backup --help` for usage info
    Backup {
        #[arg(short = 's', long = "source", required = true)]
        source: String, 
        #[arg(short = 'd', long = "dest", required = true)]
        target: String,
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

// fn usage(program: &str) {
//     eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
//     eprintln!("Subcommands:");
//     eprintln!("         snapsafe backup <source> --dest <target> --password <pwd>");
//     eprintln!("         snapsafe restore <target> --dest <source> --password <pwd>");
//     eprintln!("         snapsafe list")
// }

fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Backup { source, target, } => {
            if let Err(err) = actions::backup_data(&source, &target) {
                return  Err(Box::new(err));
            }
            return Ok(());
        },
        Commands::Restore { number, origin, target } => {
            let restore_return = actions::restore(
                number.unwrap_or(1),
                &origin, 
                &target
            );

            if let Err(err) = restore_return {
                return Err(Box::new(err))
            }
            return Ok(());
        },
        Commands::Delete { number, origin, force} => {
            let _delete_return = 
                actions::delete(number.unwrap_or(1), &origin, force);
            return Ok(());
        }
        Commands::List => {
            let _ = actions::list_from_registry();
            return Ok(());
        }
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
