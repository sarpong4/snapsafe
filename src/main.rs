use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod actions;

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
    },
    Restore {
        dest: String,
        #[arg(short = 's', long = "snapshot")]
        snapshot_id: u8,
        #[arg(short = 'o', long = "output")]
        target: String,
    },
    Delete{
        snapshot_id:u8,
    },
    List {
        #[arg(short = 'p', long, required = false)]
        path: String,
    },
}

fn usage(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("         snapsafe backup <source> --dest <target> --password <pwd>");
    eprintln!("         snapsafe restore <target> --dest <source> --password <pwd>");
    eprintln!("         snapsafe list")
}

fn entry() -> Result<(), ()> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Backup { source, target, } => {
            let _ = actions::backup_file(&source, &target);
            return Ok(());
        },
        Commands::Restore { dest, snapshot_id, target } => {
            actions::restore(&dest, snapshot_id, &target);
            return Ok(());
        },
        Commands::Delete { snapshot_id } => {
            actions::delete(snapshot_id);
            return Ok(());
        }
        Commands::List { path }=> {
            actions::list(if path.is_empty() { None } else { Some(path) });
            return Ok(());
        },
        _ => {
            usage("snapsafe");
            return Err(());
        }
    }
}

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE, 
    }
}
