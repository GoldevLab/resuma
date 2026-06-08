//! `resuma install skill` — copy the Resuma agent skill into editor skill directories.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

const SKILL_MD: &str = include_str!("../../templates/install/skill/SKILL.md");

/// Supported install targets (editor / agent runtimes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    /// Cursor personal skills: `~/.cursor/skills/resuma/`
    Cursor,
    /// Cursor project skills: `.cursor/skills/resuma/`
    CursorProject,
    /// Open agents path (Codex-style): `~/.agents/skills/resuma/`
    Agents,
}

impl InstallTarget {
    fn label(self) -> &'static str {
        match self {
            Self::Cursor => "Cursor (global)",
            Self::CursorProject => "Cursor (project)",
            Self::Agents => "Agents (global)",
        }
    }

    fn resolve(self, cwd: &Path) -> Result<PathBuf> {
        let dir = match self {
            Self::Cursor => home_dir()?.join(".cursor/skills/resuma"),
            Self::CursorProject => cwd.join(".cursor/skills/resuma"),
            Self::Agents => home_dir()?.join(".agents/skills/resuma"),
        };
        Ok(dir)
    }
}

fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("HOME not set — cannot resolve global skill path"))
}

/// Install the Resuma skill to one or more targets.
pub fn install_skill(targets: &[InstallTarget], force: bool, list_only: bool) -> Result<()> {
    if targets.is_empty() {
        return Err(anyhow!("no install target selected"));
    }

    let cwd = std::env::current_dir()?;

    if list_only {
        println!("[resuma] available skill targets:");
        for t in [
            InstallTarget::Cursor,
            InstallTarget::CursorProject,
            InstallTarget::Agents,
        ] {
            let path = t.resolve(&cwd)?;
            println!("  {} → {}", t.label(), path.display());
        }
        return Ok(());
    }

    for target in targets {
        let dest_dir = target.resolve(&cwd)?;
        let dest_file = dest_dir.join("SKILL.md");

        if dest_file.exists() && !force {
            println!(
                "[resuma] skip {} (already at {}) — use --force to overwrite",
                target.label(),
                dest_file.display()
            );
            continue;
        }

        fs::create_dir_all(&dest_dir).with_context(|| format!("create {}", dest_dir.display()))?;
        fs::write(&dest_file, SKILL_MD)
            .with_context(|| format!("write {}", dest_file.display()))?;

        println!(
            "[resuma] installed skill → {} ({})",
            dest_file.display(),
            target.label()
        );
    }

    println!();
    println!("The agent will load this skill when you work on Resuma apps.");
    println!("Docs: https://resuma-docs.fly.dev/docs/integrations/ai_assistant");
    Ok(())
}

/// Parse `--target` flag values.
pub fn parse_targets(raw: &[String], project: bool) -> Result<Vec<InstallTarget>> {
    if project {
        return Ok(vec![InstallTarget::CursorProject]);
    }
    if raw.is_empty() {
        return Ok(vec![InstallTarget::Cursor]);
    }

    let mut out = Vec::new();
    for t in raw {
        match t.as_str() {
            "cursor" => out.push(InstallTarget::Cursor),
            "project" | "cursor-project" => out.push(InstallTarget::CursorProject),
            "agents" | "codex" => out.push(InstallTarget::Agents),
            "all" => {
                out.push(InstallTarget::Cursor);
                out.push(InstallTarget::Agents);
            }
            other => {
                return Err(anyhow!(
                    "unknown target `{other}` — try: cursor, project, agents, all"
                ));
            }
        }
    }
    out.sort_by_key(|t| match t {
        InstallTarget::Cursor => 0,
        InstallTarget::Agents => 1,
        InstallTarget::CursorProject => 2,
    });
    out.dedup();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_target_is_cursor() {
        let t = parse_targets(&[], false).unwrap();
        assert_eq!(t, vec![InstallTarget::Cursor]);
    }

    #[test]
    fn project_flag_overrides() {
        let t = parse_targets(&["agents".into()], true).unwrap();
        assert_eq!(t, vec![InstallTarget::CursorProject]);
    }
}
