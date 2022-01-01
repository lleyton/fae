//! Implements autocorrection / recommendations for the fae.toml file

use crate::out;

fn remove_whitespace(s: &str) -> String {
    s.split_whitespace().collect()
}

/// Hacky method for checking if a field exists. It will remove all the
/// whitespace and then check if '{field_name}=' is present. Should work mostly
/// fine
fn field_exists(config: &str, field_name: &str) -> bool {
    remove_whitespace(config).contains(&format!("{}=", field_name))
}

/// Checks if an incorrect field is present using `field_exists` and provides a
/// console warning to the user
fn correct(config: &str, replacement: &str, field_names: Vec<&str>) {
    for field_name in field_names {
        if field_exists(config, field_name) {
            out::warning(&format!(
                "Fae doesn't support the field '{}'. You might want to use the field '{}' instead",
                field_name, replacement
            ));
        }
    }
}

/// Where we check everything to keep code complexity out of the main.rs file
pub fn run(config: &str) {
    correct(
        config,
        "uses",
        vec!["use", "needs", "dependencies", "before", "depends"],
    );

    correct(config, "run", vec!["command", "cmd", "script"]);
}
