mod rules;

use crate::rules::{RuleAction, load_rules};

use nix::unistd::execvp;
use regex::RegexBuilder;
use std::env;
use std::ffi::CString;
use std::process;

use clap::Parser;
use log::{debug, info, trace, warn};

#[derive(Debug, PartialEq, Eq, Clone)]
struct Config {
    // The list of rules
    rules: Vec<rules::Rule>,

    // The value to set for the 'Redact' action
    redact_value: String,
}

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

/// Apply changes to environment variables per options given
fn apply_env_var_filters(config: &Config, ignore_environment: bool) {
    if ignore_environment {
        info!("ignore_environment is on. All variables will be removed unless kept explicitly");
    }
    for (ref key_os, _) in env::vars_os() {
        trace!("Processing key: {:?}", &key_os);
        let key = match key_os.clone().into_string() {
            Ok(decoded_key) => {
                trace!("Successfully decoded key");
                decoded_key
            }
            Err(_) => {
                warn!("Skip proccessing non UTF-8 key: {key_os:?}");
                break;
            }
        };

        'rule_matching: {
            for rule in &config.rules {
                trace!("Checking rule {}", &rule.name);
                let re = RegexBuilder::new(&rule.pattern)
                    .case_insensitive(true)
                    .build()
                    .unwrap();
                if re.is_match(&key) {
                    info!(
                        "Key '{}' matched rule '{}'. Will take action '{:?}'",
                        &key, &rule.name, &rule.action
                    );
                    match rule.action {
                        RuleAction::Redact => {
                            if ignore_environment {
                                unsafe { env::remove_var(&key) }
                            } else {
                                unsafe { env::set_var(&key, &config.redact_value) }
                            }
                        }
                        RuleAction::Unset => unsafe {
                            env::remove_var(&key);
                        },
                        RuleAction::Keep => {}
                    }
                    break 'rule_matching;
                }
            }
            // No rules matched
            if ignore_environment {
                trace!("ignore_environment is on. Removing key '{key}'");
                unsafe {
                    env::remove_var(key);
                }
            }
        };
    }
}

#[derive(Parser, Default, Debug)]
#[command(version, about)]
/// saferenv - env but a little safer (Source code: https://github.com/jtaguchi/saferenv)
struct Cli {
    /// Start with an empty environment
    #[arg(help_heading = Some("env options"), short, long)]
    ignore_environment: bool,

    /// Remove variable from the environment (--keep has higher priority)
    #[arg(help_heading = Some("env options"), short, long, value_name="NAME")]
    unset: Vec<String>,

    /// Prevent variable from being redacted or unset
    #[arg(help_heading = Some("saferenv options"), short, long, value_name="NAME")]
    keep: Vec<String>,

    /// Set any redacted variables to this value
    #[arg(help_heading = Some("saferenv options"), short, long, value_name="VALUE", default_value="[REDACTED]")]
    redact_value: String,

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
        if !val.ends_with(".UTF-8") {
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

    let rules = load_rules(&cli.keep, &cli.unset);
    let config = Config {
        rules,
        redact_value: cli.redact_value,
    };

    debug!("{:#?}", config.rules);

    apply_env_var_filters(&config, cli.ignore_environment);

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
            info!("No command provided. Printing environment variables");
            print_env_vars();
        }
    }

    process::ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::ffi::OsString;

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

        // Add a var that should be unset in ignore_environment mode
        unsafe { env::set_var("MY_REDACTED_TOKEN", "blahblah") };
        let config = Config {
            rules: Vec::new(),
            redact_value: String::from("[REDACTED]"),
        };
        apply_env_var_filters(&config, true);
        assert_eq!(env::vars_os().count(), 0);
    }

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
        let key_value = env::var(&check_key).unwrap();

        let keep = vec![check_key.clone().into_string().unwrap()];
        dbg!(&keep);
        dbg!(env::vars_os());
        let rules = load_rules(&keep, &vec![]);
        let config = Config {
            rules,
            redact_value: String::from("[REDACTED]"),
        };
        apply_env_var_filters(&config, true);
        dbg!(env::vars_os());
        assert_eq!(env::vars_os().count(), 1);
        assert_eq!(env::var(&check_key).unwrap(), key_value)
    }

    #[test]
    #[serial(env)]
    fn test_default_generic_rules() {
        let _saved_env = SavedEnv {
            env: env::vars_os(),
        };

        unsafe {
            env::set_var("MY_TOKEN", "secretvalue");
            env::set_var("MY-TOKEN", "secretvalue");
            env::set_var("MY_SECRET", "secretvalue");
            env::set_var("MY-SECRET", "secretvalue");
            env::set_var("MY_KEY", "secretvalue");
            env::set_var("MY-KEY", "secretvalue");
        };

        dbg!(env::vars_os());
        let rules = load_rules(&vec![], &vec![]);
        let config = Config {
            rules,
            redact_value: String::from("[REDACTED]"),
        };
        apply_env_var_filters(&config, false);
        dbg!(env::vars_os());
        assert!(env::var("MY_TOKEN").unwrap() == "[REDACTED]");
        assert!(env::var("MY-TOKEN").unwrap() == "[REDACTED]");
        assert!(env::var("MY_SECRET").unwrap() == "[REDACTED]");
        assert!(env::var("MY-SECRET").unwrap() == "[REDACTED]");
        assert!(env::var("MY_KEY").unwrap() == "[REDACTED]");
        assert!(env::var("MY-KEY").unwrap() == "[REDACTED]");
    }
}
