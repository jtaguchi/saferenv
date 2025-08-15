#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rule {
    pub name: String,
    pub pattern: String,
    pub action: RuleAction,
}

// Types of actions that can be taken when a Rule matches
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuleAction {
    Keep,
    Redact,
    Unset,
}

pub fn load_rules(keep: &Vec<String>, unset: &Vec<String>) -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();
    for key in keep {
        rules.push(Rule {
            name: String::from("cli_explicit_keep"),
            pattern: format!("^{key}$"),
            action: RuleAction::Keep,
        });
    }
    for key in unset {
        rules.push(Rule {
            name: String::from("cli_explicit_unset"),
            pattern: format!("^{key}$"),
            action: RuleAction::Unset,
        });
    }

    // Add default rules
    // Generic patterns
    rules.push(Rule {
        name: String::from("generic_secret"),
        pattern: String::from(r"(_|-)SECRET$"),
        action: RuleAction::Redact,
    });
    rules.push(Rule {
        name: String::from("generic_secret_token"),
        pattern: String::from(r"(_|-)TOKEN$"),
        action: RuleAction::Redact,
    });
    rules.push(Rule {
        name: String::from("generic_secret_key"),
        pattern: String::from(r"(_|-)KEY$"),
        action: RuleAction::Redact,
    });
    rules.push(Rule {
        name: String::from("saferenv_test"),
        pattern: String::from(r"^SAFERENV_TEST$"),
        action: RuleAction::Redact,
    });

    // Specific patterns
    // ...but then I realized that the generic patterns were pretty decent

    rules
}
