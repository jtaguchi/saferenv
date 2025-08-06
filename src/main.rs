use std::env;
use std::ffi::OsString;
use std::process;

use clap::Parser;
use log::{debug, info};

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

/// Apply changes to envvironment variables per options given
fn apply_env_var_filters(set: &[OsString], unset: &[OsString], ignore_environment: bool) {
    for (ref key, _) in env::vars_os() {
        if set.contains(key) {
            info!("Keep key {key:?} (explicit-set)");
        } else if unset.contains(key) {
            info!("Remove key {key:?} (explicit-unset)");
            unsafe {
                env::remove_var(key);
            }
        } else if ignore_environment {
            unsafe {
                env::remove_var(key);
            }
        }
    }
}

#[derive(Parser, Default, Debug)]
#[command(version, about)]
/// env but a little safer
struct Cli {
    /// Start with an empty environment
    #[arg(help_heading = Some("env options"), short, long)]
    ignore_environment: bool,

    /// Remove variable from the environment (--set has higher priority)
    #[arg(help_heading = Some("env options"), short, long, value_name="NAME")]
    unset: Vec<OsString>,

    // /// Config file path
    // #[arg(help_heading = Some("saferenv options"), short, long)]
    // config: Option<String>,
    /// Pass variable to the new environment
    #[arg(help_heading = Some("saferenv options"), short, long, value_name="NAME")]
    set: Vec<OsString>,

    /// Print more detailed logs (repeat up to 3 times: -v, -vv, -vvv)
    #[arg(short, long = "debug", action = clap::ArgAction::Count)]
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

fn main() -> process::ExitCode {
    let cli = Cli::parse();

    match setup_logging(cli.verbosity) {
        Ok(_) => debug!("Logging initialized at level {}", cli.verbosity),
        Err(e) => return process::ExitCode::from(e as u8),
    }

    debug!("{cli:?}");

    apply_env_var_filters(&cli.set, &cli.unset, cli.ignore_environment);

    match cli.command {
        Some(command) => println!("{command:?}"),
        _ => print_env_vars(),
    }

    process::ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_environment() {
        apply_env_var_filters(&[], &[], true);
        assert_eq!(env::vars_os().count(), 0);
    }

    // This isn't working because it's sharing the same process
    #[test]
    fn test_ignore_environment_with_set() {
        let check_key = OsString::from("SHELL");
        dbg!(env::vars_os());
        env::vars_os().any(|x| x.0 == check_key);
        assert!(env::vars_os().any(|x| x.0 == check_key));

        let set = [check_key];
        dbg!(&set);
        dbg!(env::vars_os());
        apply_env_var_filters(&set, &[], true);
        dbg!(env::vars_os());
        assert_eq!(env::vars_os().count(), 1);
    }
}
