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
    ("flow-fullstack", "Flow + SQLx SQLite sample"),
];

pub fn prompt_template() -> Result<String> {
    println!("\nChoose a template:");
    for (i, (id, desc)) in TEMPLATE_CHOICES.iter().enumerate() {
        println!("  {}) {:<16} — {}", i + 1, id, desc);
    }
    loop {
        let choice = read_line("\nTemplate [1]: ")?;
        let picked = match choice.as_str() {
            "" | "1" => "basic",
            "2" => "todo",
            "3" => "flow",
            "4" => "flow-fullstack",
            other if TEMPLATE_CHOICES.iter().any(|(id, _)| *id == other) => other,
            _ => {
                eprintln!("  (pick 1–4 or type basic/todo/flow/flow-fullstack)");
                continue;
            }
        };
        return Ok(picked.to_string());
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

pub fn missing_arg(hint: &str) -> Result<()> {
    Err(anyhow!(
        "{hint}\n  (pass the argument directly, or run in an interactive terminal)"
    ))
}
