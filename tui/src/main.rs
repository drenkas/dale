use std::io::IsTerminal;
use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use dale_tui::cli::{Cli, Ctx};

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = Ctx::from_cli(&cli);
    let code = match cli.command {
        Some(command) => dale_tui::cli::run_command(&ctx, command),
        None => {
            if std::io::stdout().is_terminal() && std::io::stdin().is_terminal() {
                match dale_tui::tui::run(ctx) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("dale: {e}");
                        1
                    }
                }
            } else {
                // Not a terminal: behave like a plain CLI and show help.
                let _ = Cli::command().print_help();
                println!();
                0
            }
        }
    };
    ExitCode::from(code)
}
