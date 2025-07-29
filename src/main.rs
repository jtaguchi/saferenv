use std::env;

fn print_env_vars() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        print_env_vars();
    }
}
