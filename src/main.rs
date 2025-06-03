use clap::{Parser, Subcommand};
use std::process::ExitCode;

pub mod actions;

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
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(long)]
        origin: String,
        #[arg(short = 'o', long = "output")]
        target: String,
    },
    Delete{
        #[arg(short = 'n', long, required = false)]
        number: Option<u8>,
        #[arg(short = 'o', long)]
        origin: String,
    },
    List {
        #[arg(short = 'p', long, required = false)]
        path: String,
    },
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
            let _ = actions::backup_file(&source, &target);
            return Ok(());
        },
        Commands::Restore { number, origin, target } => {
            if let Some(nth) = number {
                let _ = actions::restore(nth, &origin, &target);
            }
            else {
                let _ = actions::restore(1, &origin, &target);
            }
            return Ok(());
        },
        Commands::Delete { number, origin } => {
            if let Some(nth) = number {
                actions::delete(nth, origin);
            }else {
                actions::delete(1, origin);
            }
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
