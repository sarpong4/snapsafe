use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod actions;
pub mod common;

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
        #[arg(short = 's', long = "source", required = true)]
        source: String,
        #[arg(short = 'd', long = "dest", required = true)]
        target: String,
    },
    Restore {
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(long, required = true)]
        origin: String,
        #[arg(short = 'o', long = "output", required = true)]
        target: String,
    },
    Delete{
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(short = 'o', long, required = true)]
        origin: String,
        #[arg(long)]
        force: bool
    },
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
            if let Err(err) = actions::backup_file(&source, &target) {
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
