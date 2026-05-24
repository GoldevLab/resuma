//! `resuma update` — bump project dependencies or reinstall the CLI.

use std::process::Command;

use anyhow::{anyhow, Context, Result};

use super::ensure_rust_toolchain;

pub(crate) const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn update_command(cli: bool, check: bool, version: Option<&str>) -> Result<()> {
    if cli {
        return update_cli(check, version);
    }
    update_project(check, version)
}

fn update_cli(check: bool, version: Option<&str>) -> Result<()> {
    let target = version.unwrap_or(CLI_VERSION);
    let installed = installed_cli_version();

    if check {
        print_version_line("resuma CLI (installed)", installed.as_deref());
        print_version_line("resuma CLI (this binary)", Some(CLI_VERSION));
        if let Some(latest) = latest_crates_io_version("resuma") {
            print_version_line("resuma (crates.io latest)", Some(&latest));
        }
        return Ok(());
    }

    if installed.as_deref() == Some(target) {
        println!("[resuma] CLI already at v{target}");
        return Ok(());
    }

    ensure_rust_toolchain()?;
    println!("[resuma] installing resuma v{target}…");
    let args = vec!["install", "resuma", "--force", "--version", target];
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("failed to run cargo install resuma")?;
    if !status.success() {
        return Err(anyhow!(
            "cargo install resuma failed — try: cargo install resuma --force --version {target}"
        ));
    }
    println!("[resuma] CLI updated to v{target}");
    Ok(())
}

fn update_project(check: bool, version: Option<&str>) -> Result<()> {
    let root = std::env::current_dir()?;
    let cargo_path = root.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(anyhow!(
            "run `resuma update` from a Rust project root (Cargo.toml not found)"
        ));
    }

    let cargo = std::fs::read_to_string(&cargo_path)?;
    let dep = parse_resuma_dependency(&cargo).ok_or_else(|| {
        anyhow!("no `resuma` dependency in Cargo.toml — is this a Resuma project?")
    })?;

    let current = match &dep {
        ResumaDependency::Version(v) => Some(v.as_str()),
        ResumaDependency::Path(path) => {
            if check {
                println!("  resuma (project Cargo.toml): path ({path})");
                print_version_line("resuma (CLI bundled version)", Some(CLI_VERSION));
                if let Some(latest) = latest_crates_io_version("resuma") {
                    print_version_line("resuma (crates.io latest)", Some(&latest));
                }
                println!("\n[resuma] path dependency — edit Cargo.toml or use a published version to run `resuma update`");
                return Ok(());
            }
            return Err(anyhow!(
                "project uses a path dependency on resuma ({path}) — switch to `resuma = \"{CLI_VERSION}\"` to use `resuma update`, or pull latest source in the monorepo"
            ));
        }
    };
    let target = version.unwrap_or(CLI_VERSION);

    if check {
        if let Some(v) = parse_resuma_version(&cargo) {
            print_version_line("resuma (project Cargo.toml)", Some(&v));
        }
        print_version_line("resuma (CLI bundled version)", Some(CLI_VERSION));
        if let Some(latest) = latest_crates_io_version("resuma") {
            print_version_line("resuma (crates.io latest)", Some(&latest));
        }
        if let Some(cur) = current {
            if versions_compatible(cur, target) {
                println!("\n[resuma] project dependency looks up to date");
            } else {
                println!("\n[resuma] update available — run: resuma update");
            }
        }
        return Ok(());
    }

    if current.is_some_and(|c| versions_compatible(c, target)) {
        println!("[resuma] project already uses resuma {target}");
        println!("[resuma] refreshing lockfile…");
    } else {
        println!("[resuma] updating resuma → {target} in Cargo.toml…");
        ensure_rust_toolchain()?;
        let spec = format!("resuma@{target}");
        let status = Command::new("cargo")
            .args(["add", &spec])
            .current_dir(&root)
            .status()
            .context("failed to run cargo add")?;
        if !status.success() {
            return Err(anyhow!(
                "cargo add {spec} failed — ensure cargo >= 1.62 or edit Cargo.toml manually"
            ));
        }
    }

    let status = Command::new("cargo")
        .args(["update", "-p", "resuma", "-p", "resuma-macros"])
        .current_dir(&root)
        .status()
        .context("failed to run cargo update")?;
    if !status.success() {
        return Err(anyhow!("cargo update failed"));
    }

    println!("[resuma] dependencies updated — run `cargo build` to verify");
    Ok(())
}

fn print_version_line(label: &str, version: Option<&str>) {
    match version {
        Some(v) => println!("  {label}: v{v}"),
        None => println!("  {label}: (not found)"),
    }
}

pub(crate) fn parse_resuma_version(cargo: &str) -> Option<String> {
    match parse_resuma_dependency(cargo)? {
        ResumaDependency::Version(v) => Some(v),
        ResumaDependency::Path(_) => None,
    }
}

#[derive(Debug)]
pub(crate) enum ResumaDependency {
    Version(String),
    Path(String),
}

pub(crate) fn parse_resuma_dependency(cargo: &str) -> Option<ResumaDependency> {
    for line in cargo.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("resuma") {
            let rest = rest.trim_start();
            if rest.starts_with('=') {
                let value = rest.trim_start_matches('=').trim();
                if let Some(v) = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    return Some(ResumaDependency::Version(v.to_string()));
                }
                if value.starts_with('{') {
                    let inner = value.trim_start_matches('{').trim_end_matches('}');
                    for part in inner.split(',') {
                        let part = part.trim();
                        if let Some(path) = part.strip_prefix("path").map(str::trim) {
                            let path = path.trim_start_matches('=').trim();
                            if let Some(p) = path.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
                            {
                                return Some(ResumaDependency::Path(p.to_string()));
                            }
                        }
                        if let Some(ver) = part.strip_prefix("version").map(str::trim) {
                            let ver = ver.trim_start_matches('=').trim();
                            if let Some(q) = ver.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
                            {
                                return Some(ResumaDependency::Version(q.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn versions_compatible(current: &str, target: &str) -> bool {
    current == target || current.starts_with(&format!("{target}.")) || target.starts_with(current)
}

fn installed_cli_version() -> Option<String> {
    Command::new("resuma")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .and_then(|s| s.split_whitespace().last().map(str::to_string))
}

pub(crate) fn latest_crates_io_version(crate_name: &str) -> Option<String> {
    let output = Command::new("cargo")
        .args(["search", crate_name, "--limit", "1"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    // cargo search output: `resuma = "0.3.1" # description`
    let line = text.lines().next()?;
    let after_eq = line.split('=').nth(1)?.trim();
    let start = after_eq.find('"')? + 1;
    let rest = &after_eq[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_version() {
        let cargo = r#"
[dependencies]
resuma = "0.3"
tokio = "1"
"#;
        assert_eq!(parse_resuma_version(cargo).as_deref(), Some("0.3"));
    }

    #[test]
    fn parse_inline_table() {
        let cargo = r#"
[dependencies]
resuma = { version = "0.3.1", default-features = false }
"#;
        assert_eq!(parse_resuma_version(cargo).as_deref(), Some("0.3.1"));
    }

    #[test]
    fn parse_path_dependency() {
        let cargo = r#"
[dependencies]
resuma = { path = "../../crates/resuma" }
"#;
        match parse_resuma_dependency(cargo) {
            Some(ResumaDependency::Path(p)) => assert!(p.contains("crates/resuma")),
            other => panic!("expected path dep, got {other:?}"),
        }
    }
}
