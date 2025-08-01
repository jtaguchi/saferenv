use std::env;
use std::process;

use clap::Parser;
use log::{debug, error, info, trace, warn};

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

#[derive(Parser, Default, Debug)]
#[command(version, about)]
/// env but a little safer
struct Cli {
    /// Start with an empty environment
    #[arg(short, long)]
    ignore_environment: bool,

    /// Print more detailed logs (use up to 3: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,

    /// The COMMAND to run in the resulting environment. If no COMMAND, print the resulting environment.
    command: Option<Vec<String>>,
}

fn setup_logging(verbosity: u8) -> Result<(), exitcode::ExitCode> {
    let log_level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => {
            eprintln!("verbosity level cannot be greater than 3 (-vvv)");
            return Err(exitcode::USAGE);
        }
    };

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log_level)
        .init();

    Ok(())
}

fn remove_all_env_vars() {
    for (ref key, _) in env::vars_os() {
        unsafe {
            env::remove_var(key);
        }
    }
}

fn main() -> process::ExitCode {
    let cli = Cli::parse();

    match setup_logging(cli.verbosity) {
        Ok(_) => debug!("Logging initialized at level {}", cli.verbosity),
        Err(e) => return process::ExitCode::from(e as u8),
    }

    debug!("{cli:?}");

    if cli.ignore_environment {
        remove_all_env_vars();
    }

    match cli.command {
        Some(command) => println!("{command:?}"),
        _ => print_env_vars(),
    }

    process::ExitCode::SUCCESS
}
