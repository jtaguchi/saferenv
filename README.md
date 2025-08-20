# **saferenv**: Secure Your Environment Variables

`saferenv` is a Rust-based utility that improves on the classic `env` command by automatically redacting sensitive environment variables, such as API keys and other secrets, before printing them to the terminal. With `saferenv`, you can securely inspect your environment variables without accidentally exposing sensitive data.

---

## **Features**

- **Redaction of Sensitive Variables**: Automatically identifies and redacts sensitive environment variables such as `API_KEY`, `SECRET`, `PASSWORD`, etc.
- **Improved Safety**: Useful for developers who want to avoid printing secrets to the terminal in a development or CI/CD environment.
- **Compatible with env**: Works just like the `env` command with (mostly) the same options and flags.
- **Customizable**: Define custom patterns for redacting your specific sensitive data. (Optional config file work in progress)

Note: saferenv is not a secrets detection tool. Use [TruffleHog](https://github.com/trufflesecurity/trufflehog) or [Gitleaks](https://github.com/gitleaks/gitleaks) instead for detecting potential secrets.

---

## **Usage**

### Using saferenv to print the environment
```bash
# List all environment variables, with sensitive data redacted
$ saferenv
HOME=/home/user
AWS_SECRET_ACCESS_KEY=[REDACTED]
...

# List all environment variables, while allowing specific variables from being redacted
$ saferenv --keep AWS_SECRET_ACCESS_KEY
HOME=/home/user
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
...
```
By default, saferenv will redact secret variables matching common names like API_KEY. You can control which specific variables to keep or remove using the -k/--keep and -u/--unset options. See the [rules section](#rules) below for the full list of patterns.

### Using saferenv to run commands in a new environment
```bash
# Run a command with redacted environment variables
$ saferenv command-to-run

# Run a command with an empty environment
$ saferenv -i command-to-run

# Run a command but unset a variable in the nex environment
$ saferenv --unset UNUSED_VARIABLE command-to-run

```

### Viewing the rules
```bash
# Print all of the rules being applied, including any options passed in the command
$ saferenv --show-rules --keep KEEPTHIS --unset REMOVETHAT
Rule 1: cli_explicit_keep
    pattern: "^KEEPTHIS$"
    action: Keep
Rule 2: cli_explicit_unset
    pattern: "^REMOVETHAT$"
    action: Unset
...
```

---

## Installation

### From source
```bash
$ git clone https://github.com/jtaguchi/saferenv.git
$ cd saferenv
$ cargo install --path .
```

---

## Why Use saferenv?

Environment variables are commonly used to store and secrets which are accessible by any application running in that environment. As development teams work with sensitive environment variables, it's easy to expose secrets accidentally, whether it's by printing variables while sharing the screen, or a malicious script that looks for secrets in variables. saferenv can help mitigate this by launching the command in a new environment with only the variables that you choose to pass.

---

## Rules
By default, variables that match the regular expression patterns below will be automatically redacted. Matching is case insensitive.

```
"SECRETS?$"
"TOKENS?$"
"KEYS?$"
"PASSWORDS?$"
"(_|-)PW$"
```