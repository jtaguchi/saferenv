use std::env;

fn main() {
    for (var_key, var_val) in env::vars() {
        println!("{var_key}={var_val}");
    }
}
