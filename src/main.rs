use std::env;

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

    /// Print more detailed logs
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,

    /// The COMMAND to run in the resulting environment. If no COMMAND, print the resulting environment.
    command: Option<Vec<String>>,
}

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => {
            eprintln!("verbosity level cannot be greater than 3 (-vvv)");
            std::process::exit(1);
        }
    };

    env_logger::builder()
        .format_timestamp(None)
        .filter_level(log_level)
        .init();

    debug!("{cli:?}");

    // if cli.command
    // println!(command);
    // print_env_vars();
}
