use nix::unistd::execvp;
use std::env;
use std::ffi::{CString, OsString};
use std::process;

use clap::Parser;
use log::{debug, info, trace, warn};

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Rule {
    name: String,
    pattern: String,
    action: RuleAction,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Config {
    rules: Vec<Rule>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum RuleAction {
    Keep,
    Redact,
    Unset,
}

fn load_rules() -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    // for keep in &cli.keep {
    //     rules.push(Rule {
    //         name: String::from("cli_explicit_keep"),
    //         pattern: String::from(keep),
    //     });
    // }
    rules.push(Rule {
        name: String::from("aws_secret_access_key"),
        pattern: String::from(r"^AWS_SECRET_ACCESS_KEY$"),
        action: RuleAction::Redact,
    });
    rules.push(Rule {
        name: String::from("aws_secret_access_key"),
        pattern: String::from(r"TERM"),
        action: RuleAction::Redact,
    });

    rules
}

/// Apply changes to environment variables per options given
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

fn detect_and_warn_non_utf8_environment() {
    if let Ok(val) = env::var("LANG") {
        debug!("LANG={val:?}");
        if val.ends_with(".UTF-8") {
            warn!(
                "Non UTF-8 environment detected. Only UTF-8 is currently supported and errors may occur."
            );
        }
    }
}

fn main() -> process::ExitCode {
    let cli = Cli::parse();

    match setup_logging(cli.verbosity) {
        Ok(_) => debug!("Logging initialized at level {}", cli.verbosity),
        Err(e) => return process::ExitCode::from(e as u8),
    }

    detect_and_warn_non_utf8_environment();

    debug!("{cli:?}");

    let rules = load_rules();

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

    // Used to save and restore environment variables after each test
    #[derive(Debug)]
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
        // Using SHELL as a test variable
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
