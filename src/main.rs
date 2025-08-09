use nix::unistd::execvp;
use std::env;
use std::ffi::{CString, OsString};
use std::process;

use clap::Parser;
use log::{debug, info, trace};

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

struct Rule {
    name: String,
    pattern: String,
}

struct Config {
    rules: Vec<Rule>,
}

/// Apply changes to envvironment variables per options given
fn apply_env_var_filters(keep: &[OsString], unset: &[OsString], ignore_environment: bool) {
    if ignore_environment {
        info!("ignore_environment is on. All variables will be removed unless kept explicitly");
    }
    for (ref key, _) in env::vars_os() {
        if keep.contains(key) {
            info!("Keep key {key:?} (explicit_set)");
        } else if unset.contains(key) {
            info!("Remove key {key:?} (explicit_unset)");
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
/// saferenv - env but a little safer
struct Cli {
    /// Start with an empty environment
    #[arg(help_heading = Some("env options"), short, long)]
    ignore_environment: bool,

    /// Remove variable from the environment (--keep has higher priority)
    #[arg(help_heading = Some("env options"), short, long, value_name="NAME")]
    unset: Vec<OsString>,

    // /// Config file path
    // #[arg(help_heading = Some("saferenv options"), short, long)]
    // config: Option<String>,
    /// Pass variable to the new environment
    #[arg(help_heading = Some("saferenv options"), short, long, value_name="NAME")]
    keep: Vec<OsString>,

    /// Print more detailed logs (repeat up to 3 times: -v, -vv, -vvv)
    #[arg(short, long = "debug", action = clap::ArgAction::Count)]
    verbosity: u8,

    /// The COMMAND to run in the resulting environment. If no COMMAND, print the resulting environment.
    #[arg(trailing_var_arg = true)]
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

    let rules: Vec<Rule> = Vec::new();

    apply_env_var_filters(&cli.keep, &cli.unset, cli.ignore_environment);

    match cli.command {
        Some(command) => {
            info!("Executing command...");
            let Ok(program) = CString::new(command[0].clone()) else {
                return process::ExitCode::from(exitcode::DATAERR as u8);
            };
            let mut argv: Vec<CString> = Vec::new();
            // argv0 is added separately here for when I implement the --argv0 option someday
            argv.push(CString::new(command[0].clone()).expect("Could not process arg0"));
            trace!("{argv:?}");
            for arg in &command[1..] {
                argv.push(CString::new(arg.clone()).expect("Could not process arg"));
                trace!("{argv:?}")
            }
            execvp(&program, &argv).expect_err("execvp should never return if successful");
        }
        // If a command was not given, print env variables
        _ => {
            info!("No command. Printing environment variables");
            print_env_vars();
        }
    }

    process::ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    struct SavedEnv {
        env: env::VarsOs,
    }

    impl Drop for SavedEnv {
        fn drop(&mut self) {
            for (key, val) in &mut self.env {
                unsafe {
                    env::set_var(key, val);
                }
            }
        }
    }

    #[test]
    #[serial(env)]
    fn test_ignore_environment() {
        let _saved_env = SavedEnv {
            env: env::vars_os(),
        };
        apply_env_var_filters(&[], &[], true);
        assert_eq!(env::vars_os().count(), 0);
    }

    // This isn't working because it's sharing the same process
    #[test]
    #[serial(env)]
    fn test_ignore_environment_with_set() {
        let _saved_env = SavedEnv {
            env: env::vars_os(),
        };
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
