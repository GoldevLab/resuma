//! Minimal interactive prompts when stdin is a TTY.

use std::io::{self, IsTerminal, Write};

use anyhow::{anyhow, Result};

pub fn is_interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn read_line(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub fn prompt_required(label: &str) -> Result<String> {
    loop {
        let value = read_line(label)?;
        if value.is_empty() {
            eprintln!("  (required — enter a value)");
            continue;
        }
        if value.contains(['/', '\\']) || value.contains("..") {
            eprintln!("  (invalid — use a simple directory name)");
            continue;
        }
        return Ok(value);
    }
}

const TEMPLATE_CHOICES: &[(&str, &str)] = &[
    ("basic", "static SSR page, zero client JS"),
    ("todo", "full showcase (signals, server, islands)"),
    ("flow", "multi-page app with src/pages/"),
    ("flow-booking", "appointments + query-driven #[load]"),
    ("flow-fullstack", "Flow + SQLx SQLite sample"),
    ("production", "Flow + security stub + Docker + Fly"),
];

/// Map a prompt reply (index, empty = 1, or template id) to a template name.
fn resolve_template_choice(choice: &str) -> Option<&'static str> {
    match choice {
        "" | "1" => Some("basic"),
        "2" => Some("todo"),
        "3" => Some("flow"),
        "4" => Some("flow-booking"),
        "5" => Some("flow-fullstack"),
        "6" => Some("production"),
        other => TEMPLATE_CHOICES
            .iter()
            .find(|(id, _)| *id == other)
            .map(|(id, _)| *id),
    }
}

pub fn prompt_template() -> Result<String> {
    println!("\nChoose a template:");
    for (i, (id, desc)) in TEMPLATE_CHOICES.iter().enumerate() {
        println!("  {}) {:<16} — {}", i + 1, id, desc);
    }
    loop {
        let choice = read_line("\nTemplate [1]: ")?;
        match resolve_template_choice(&choice) {
            Some(picked) => return Ok(picked.to_string()),
            None => {
                eprintln!(
                    "  (pick 1–6 or type basic/todo/flow/flow-booking/flow-fullstack/production)"
                );
            }
        }
    }
}

pub fn prompt_integration() -> Result<String> {
    println!("\nAdd an integration:");
    println!("  1) sqlx   — SQLite/Postgres via SQLx + migrations");
    println!("  2) turso  — Turso/libSQL edge database");
    loop {
        let choice = read_line("\nIntegration [1]: ")?;
        let picked = match choice.as_str() {
            "" | "1" | "sqlx" => "sqlx",
            "2" | "turso" => "turso",
            _ => {
                eprintln!("  (pick 1–2 or type sqlx/turso)");
                continue;
            }
        };
        return Ok(picked.to_string());
    }
}

/// Parse a yes/no reply. Empty uses `default_yes`. Accepts y/yes/s/si/sí.
pub fn parse_confirm(line: &str, default_yes: bool) -> bool {
    match line.trim().to_ascii_lowercase().as_str() {
        "" => default_yes,
        "y" | "yes" | "s" | "si" | "sí" => true,
        "n" | "no" => false,
        _ => default_yes,
    }
}

/// Interactive yes/no. Returns `default_yes` on empty input.
pub fn confirm(question: &str, default_yes: bool) -> Result<bool> {
    let hint = if default_yes { "Y/n" } else { "y/N" };
    let line = read_line(&format!("{question} [{hint}]: "))?;
    Ok(parse_confirm(&line, default_yes))
}

pub fn missing_arg(hint: &str) -> Result<()> {
    Err(anyhow!(
        "{hint}\n  (pass the argument directly, or run in an interactive terminal)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_confirm_defaults_and_aliases() {
        assert!(!parse_confirm("", false));
        assert!(parse_confirm("", true));
        assert!(parse_confirm("y", false));
        assert!(parse_confirm("YES", false));
        assert!(parse_confirm("si", false));
        assert!(!parse_confirm("n", true));
        assert!(!parse_confirm("no", true));
    }

    #[test]
    fn resolve_template_choice_covers_every_listed_template() {
        assert_eq!(TEMPLATE_CHOICES.len(), 6);
        assert_eq!(resolve_template_choice(""), Some("basic"));
        assert_eq!(resolve_template_choice("1"), Some("basic"));
        assert_eq!(resolve_template_choice("6"), Some("production"));
        assert_eq!(resolve_template_choice("production"), Some("production"));
        assert_eq!(
            resolve_template_choice("flow-booking"),
            Some("flow-booking")
        );
        assert_eq!(resolve_template_choice("7"), None);
        assert_eq!(resolve_template_choice("nope"), None);
    }
}
