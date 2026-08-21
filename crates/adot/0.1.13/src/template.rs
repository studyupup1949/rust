use std::collections::HashMap;

use crate::config::Variable;

/// Render a template string by replacing `{{@@ key @@}}` with variable values,
/// evaluating `{%@@ if profile == "X" @@%}` or `{%@@ if os == "X" @@%}` conditionals,
/// and stripping `{#@@ ... @@#}` comments.
///
/// Built-in variables: `profile`, `os`
pub fn render(
    content: &str,
    variables: &HashMap<String, Variable>,
    profile: &str,
) -> Result<String, String> {
    let os = std::env::consts::OS;

    // Inject built-in variables
    let mut vars = variables.clone();
    vars.insert("profile".to_string(), Variable::Value(profile.to_string()));
    vars.insert("os".to_string(), Variable::Value(os.to_string()));

    let result = strip_comments(content);
    let result = eval_conditionals(&result, profile, os)?;
    let result = replace_variables(&result, &vars)?;
    Ok(result)
}

/// Strip `{#@@ ... @@#}` comment lines
fn strip_comments(content: &str) -> String {
    let mut result = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{#@@") && trimmed.ends_with("@@#}") {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Evaluate `{%@@ if/elif/endif @@%}` blocks
fn eval_conditionals(content: &str, profile: &str, os: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut inside_block = false;
    let mut include_block = false;
    let mut already_matched = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("{%@@") && trimmed.contains("endif") && trimmed.ends_with("@@%}") {
            inside_block = false;
            include_block = false;
            already_matched = false;
            continue;
        }

        if trimmed.starts_with("{%@@") && trimmed.contains("elif ") && trimmed.ends_with("@@%}") {
            if already_matched {
                include_block = false;
                continue;
            }

            let condition = trimmed
                .trim_start_matches("{%@@")
                .trim_end_matches("@@%}")
                .trim()
                .trim_start_matches("el");

            include_block = eval_condition(condition, profile, os)?;
            if include_block {
                already_matched = true;
            }
            continue;
        }

        if trimmed.starts_with("{%@@") && trimmed.contains("else") && trimmed.ends_with("@@%}") {
            if already_matched {
                include_block = false;
            } else {
                include_block = true;
                already_matched = true;
            }
            continue;
        }

        if trimmed.starts_with("{%@@") && trimmed.contains("if ") && trimmed.ends_with("@@%}") {
            let condition = trimmed
                .trim_start_matches("{%@@")
                .trim_end_matches("@@%}")
                .trim();

            include_block = eval_condition(condition, profile, os)?;
            if include_block {
                already_matched = true;
            }
            inside_block = true;
            continue;
        }

        if inside_block && !include_block {
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    if inside_block {
        return Err("unclosed {%@@ if ... @@%} block".to_string());
    }

    Ok(result)
}

/// Parse `if profile == "value"` or `if os == "value"` condition (supports == and !=)
fn eval_condition(condition: &str, profile: &str, os: &str) -> Result<bool, String> {
    let condition = condition.trim_start_matches("if").trim();

    // Check for != first (before ==, since == is a substring)
    let (parts, negate) = if condition.contains("!=") {
        (condition.splitn(2, "!=").collect::<Vec<&str>>(), true)
    } else if condition.contains("==") {
        (condition.splitn(2, "==").collect::<Vec<&str>>(), false)
    } else {
        return Err(format!("unsupported condition: {condition}"));
    };

    if parts.len() != 2 {
        return Err(format!("unsupported condition: {condition}"));
    }

    let lhs = parts[0].trim();
    let rhs = parts[1].trim().trim_matches('"');

    let result = match lhs {
        "profile" => profile == rhs,
        "os" => os == rhs,
        _ => {
            return Err(format!(
                "unsupported condition variable: {lhs} (only 'profile' and 'os' supported)"
            ))
        }
    };

    Ok(if negate { !result } else { result })
}

/// Replace all `{{@@ key.path @@}}` with resolved variable values
fn replace_variables(
    content: &str,
    variables: &HashMap<String, Variable>,
) -> Result<String, String> {
    let mut result = content.to_string();

    while let Some(start) = result.find("{{@@") {
        let end = match result[start..].find("@@}}") {
            Some(pos) => start + pos + 4,
            None => return Err(format!("unclosed {{{{@@ at position {start}")),
        };

        let key = result[start + 4..end - 4].trim();
        let value = resolve_variable(key, variables)?;
        result.replace_range(start..end, &value);
    }

    Ok(result)
}

/// Resolve a dotted key like `git.email` or `colors.foreground` from nested variables
fn resolve_variable(key: &str, variables: &HashMap<String, Variable>) -> Result<String, String> {
    let parts: Vec<&str> = key.split('.').collect();

    let first = parts
        .first()
        .expect("split always returns at least one element");
    if first.is_empty() {
        return Err("empty variable key".to_string());
    }

    let var = variables
        .get(*first)
        .ok_or_else(|| format!("undefined variable: {key}"))?;

    let mut current = var;
    for part in &parts[1..] {
        match current {
            Variable::Nested(map) => {
                current = map
                    .get(*part)
                    .ok_or_else(|| format!("undefined variable: {key}"))?;
            }
            Variable::Value(_) => {
                return Err(format!(
                    "variable '{first}' is not nested, cannot resolve: {key}"
                ));
            }
        }
    }

    match current {
        Variable::Value(v) => Ok(v.clone()),
        Variable::Nested(_) => Err(format!("variable '{key}' is a nested map, not a value")),
    }
}
