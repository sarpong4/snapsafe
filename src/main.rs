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
        #[arg(short = 's', long = "source")]
        source: String,
        #[arg(short = 'd', long = "dest")]
        target: String,
    },
    Restore {
        #[arg(long)]
        origin: String,
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

fn entry() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();

    match cli.command {
        Commands::Backup { source, target, } => {
            let _ = actions::backup_file(&source, &target);
            println!("Backup complete");
            return Ok(());
        },
        Commands::Restore { origin, snapshot_id, target } => {
            actions::restore(&origin, snapshot_id, &target);
            return Ok(());
        },
        Commands::Delete { snapshot_id } => {
            actions::delete(snapshot_id);
            return Ok(());
        }
        Commands::List { path }=> {
            actions::list(if path.is_empty() { None } else { Some(path) });
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
