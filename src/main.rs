use std::process::ExitCode;

use snapsafe::commands::entry;

fn main() -> ExitCode {
    match entry() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[ERROR]: {e}");
            ExitCode::FAILURE
        }
    }
}
