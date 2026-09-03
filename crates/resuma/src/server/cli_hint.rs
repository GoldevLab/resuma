//! Startup hint when an app is launched with `cargo run` instead of `resuma dev`.

use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

static HINTED: AtomicBool = AtomicBool::new(false);

fn env_flag_on(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

fn env_flag_off(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("0") | Ok("false") | Ok("FALSE") | Ok("no") | Ok("NO")
    )
}

fn production_env() -> bool {
    matches!(
        std::env::var("RESUMA_ENV").as_deref(),
        Ok("production") | Ok("prod")
    )
}

fn via_cli() -> bool {
    env_flag_on("RESUMA_VIA_CLI")
}

fn hints_disabled() -> bool {
    env_flag_off("RESUMA_CLI_HINT")
}

fn interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn cli_installed() -> bool {
    Command::new("resuma")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn parse_confirm(line: &str, default_yes: bool) -> bool {
    match line.trim().to_ascii_lowercase().as_str() {
        "" => default_yes,
        "y" | "yes" | "s" | "si" => true,
        "n" | "no" => false,
        _ => default_yes,
    }
}

/// Whether this process should print the cargo-run CLI tip.
pub(crate) fn should_hint() -> bool {
    !production_env() && !via_cli() && !hints_disabled()
}

pub(crate) fn print_cli_commands() {
    println!(
        "\
[resuma] CLI commands:
  resuma dev                 hot reload (watches src/, http://127.0.0.1:3000)
  resuma build               production binary
  resuma routes --generate   rebuild src/pages/_registry.rs
  resuma add sqlx            SQLite / Postgres scaffold
  resuma update              bump resuma in this project
  resuma update --cli        reinstall the global CLI
  resuma doctor              toolchain + project health
  resuma new my-app          scaffold a new app
  resuma --help              full list
Docs: https://resuma-docs.fly.dev/docs"
    );
}

/// One-shot tip (and optional install) when the app is started with `cargo run`.
///
/// Skipped in production, when launched by `resuma dev` (`RESUMA_VIA_CLI=1`),
/// or when `RESUMA_CLI_HINT=0`. Never fails the server.
pub fn maybe_hint_cli() {
    if HINTED.swap(true, Ordering::SeqCst) || !should_hint() {
        return;
    }

    if cli_installed() {
        println!(
            "[resuma] tip: `cargo run` has no hot reload. Prefer `resuma dev` (CLI already installed)."
        );
        return;
    }

    if !interactive() {
        println!(
            "[resuma] tip: install the Resuma CLI for hot reload:\n    cargo install resuma\n    resuma dev"
        );
        return;
    }

    println!(
        "[resuma] you're using `cargo run`. The Resuma CLI adds hot reload, routes, and updates."
    );
    print!("Install the CLI now (`cargo install resuma`)? [y/N]: ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return;
    }
    if !parse_confirm(&line, false) {
        println!("[resuma] skipped — later: cargo install resuma && resuma dev");
        return;
    }

    println!("[resuma] running `cargo install resuma`…");
    match Command::new("cargo").args(["install", "resuma"]).status() {
        Ok(status) if status.success() => {
            println!("[resuma] CLI installed. From this project directory:");
            print_cli_commands();
            println!(
                "[resuma] stop this `cargo run` and start `resuma dev` when you want hot reload."
            );
        }
        Ok(_) => {
            eprintln!(
                "[resuma] cargo install failed — try: cargo install resuma\n         Rust: https://rustup.rs"
            );
        }
        Err(_) => {
            eprintln!("[resuma] cargo not found — install Rust from https://rustup.rs");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_confirm_no_by_default() {
        assert!(!parse_confirm("", false));
        assert!(parse_confirm("y", false));
        assert!(!parse_confirm("n", true));
    }

    #[test]
    fn should_hint_respects_via_cli() {
        let prev = std::env::var_os("RESUMA_VIA_CLI");
        std::env::set_var("RESUMA_VIA_CLI", "1");
        assert!(!should_hint());
        match prev {
            Some(v) => std::env::set_var("RESUMA_VIA_CLI", v),
            None => std::env::remove_var("RESUMA_VIA_CLI"),
        }
    }

    #[test]
    fn should_hint_respects_opt_out() {
        let via = std::env::var_os("RESUMA_VIA_CLI");
        let hint = std::env::var_os("RESUMA_CLI_HINT");
        std::env::remove_var("RESUMA_VIA_CLI");
        std::env::set_var("RESUMA_CLI_HINT", "0");
        assert!(!should_hint());
        match via {
            Some(v) => std::env::set_var("RESUMA_VIA_CLI", v),
            None => std::env::remove_var("RESUMA_VIA_CLI"),
        }
        match hint {
            Some(v) => std::env::set_var("RESUMA_CLI_HINT", v),
            None => std::env::remove_var("RESUMA_CLI_HINT"),
        }
    }
}
