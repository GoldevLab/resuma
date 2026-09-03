//! Same-project `resuma dev` reclaim: pidfile + leftover `cargo-watch` trees.
//!
//! `cargo watch` is long-lived. If the CLI exits without reaping it (sandbox,
//! SIGKILL, a dropped SSH session), the next `resuma dev` used to hop ports
//! while the orphaned app kept serving stale binaries.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const PIDFILE_REL: &str = ".resuma/dev.pid";

fn pidfile_path() -> PathBuf {
    PathBuf::from(PIDFILE_REL)
}

pub fn parse_dev_pidfile(text: &str) -> Option<(u32, String)> {
    let mut lines = text.lines();
    let pid = lines.next()?.trim().parse().ok()?;
    let cwd = lines.next().unwrap_or("").trim().to_string();
    Some((pid, cwd))
}

fn project_cwd() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    fs::canonicalize(&cwd).ok().or(Some(cwd))
}

fn write_pidfile(pid: u32, cwd: &Path) -> std::io::Result<()> {
    if let Some(parent) = pidfile_path().parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(pidfile_path())?;
    writeln!(f, "{pid}")?;
    writeln!(f, "{}", cwd.display())?;
    Ok(())
}

pub fn remove_pidfile() {
    let _ = fs::remove_file(pidfile_path());
}

/// Record the `cargo watch` pid so a later `resuma dev` in this directory can stop it.
pub fn record_watch_pid(pid: u32) {
    let Some(cwd) = project_cwd() else { return };
    if let Err(err) = write_pidfile(pid, &cwd) {
        eprintln!("[resuma] could not write {PIDFILE_REL}: {err}");
    }
}

/// Stop leftover watchers for *this* project, then bind. Other directories are left alone.
pub fn reclaim_same_project_dev() {
    let Some(cwd) = project_cwd() else { return };
    let self_pid = std::process::id();
    let mut pids: Vec<u32> = Vec::new();

    if let Ok(text) = fs::read_to_string(pidfile_path()) {
        if let Some((pid, recorded)) = parse_dev_pidfile(&text) {
            if recorded.is_empty() || Path::new(&recorded) == cwd.as_path() {
                pids.push(pid);
            }
        }
    }

    pids.extend(scan_watch_pids(&cwd, self_pid));
    pids.sort_unstable();
    pids.dedup();
    pids.retain(|&pid| pid != self_pid && proc_alive(pid));

    let mut stopped = !pids.is_empty();
    for pid in &pids {
        eprintln!("[resuma] stopping leftover `resuma dev` watcher (pid {pid})");
        stop_pid_tree(*pid);
    }

    // Orphaned app binaries whose parent watch already died (PPID 1 / systemd).
    for pid in scan_dev_bin_pids(&cwd, self_pid) {
        eprintln!("[resuma] stopping leftover dev binary (pid {pid})");
        stop_pid_tree(pid);
        stopped = true;
    }

    if stopped {
        std::thread::sleep(Duration::from_millis(150));
    }
    remove_pidfile();
}

/// Spawn `cargo watch`, keep a pidfile for the life of the child, then wait.
pub fn spawn_and_wait_watch(mut cmd: Command) -> std::io::Result<std::process::ExitStatus> {
    let mut child = cmd.spawn()?;
    record_watch_pid(child.id());
    let status = child.wait();
    remove_pidfile();
    status
}

fn proc_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

fn stop_pid_tree(pid: u32) {
    let kids = descendants(pid);
    for child in kids.iter().rev() {
        terminate(*child);
    }
    terminate(pid);
    std::thread::sleep(Duration::from_millis(80));
    for child in kids.iter().rev() {
        if proc_alive(*child) {
            kill_force(*child);
        }
    }
    if proc_alive(pid) {
        kill_force(pid);
    }
}

fn terminate(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T"])
            .status();
    }
}

fn kill_force(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string(), "/T"])
            .status();
    }
}

fn descendants(pid: u32) -> Vec<u32> {
    let mut acc = Vec::new();
    let mut stack = vec![pid];
    while let Some(p) = stack.pop() {
        for c in children_of(p) {
            if !acc.contains(&c) {
                acc.push(c);
                stack.push(c);
            }
        }
    }
    acc
}

#[cfg(target_os = "linux")]
fn children_of(pid: u32) -> Vec<u32> {
    fs::read_to_string(format!("/proc/{pid}/task/{pid}/children"))
        .ok()
        .map(|s| {
            s.split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(target_os = "linux"))]
fn children_of(_pid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn proc_cmdline(pid: u32) -> String {
    fs::read(format!("/proc/{pid}/cmdline"))
        .map(|b| String::from_utf8_lossy(&b).replace('\0', " "))
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn proc_cwd(pid: u32) -> Option<PathBuf> {
    fs::read_link(format!("/proc/{pid}/cwd")).ok()
}

#[cfg(target_os = "linux")]
fn scan_watch_pids(cwd: &Path, self_pid: u32) -> Vec<u32> {
    scan_proc(cwd, self_pid, is_resuma_dev_watch)
}

#[cfg(not(target_os = "linux"))]
fn scan_watch_pids(_cwd: &Path, _self_pid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn scan_dev_bin_pids(cwd: &Path, self_pid: u32) -> Vec<u32> {
    scan_proc(cwd, self_pid, is_orphaned_dev_bin)
}

#[cfg(not(target_os = "linux"))]
fn scan_dev_bin_pids(_cwd: &Path, _self_pid: u32) -> Vec<u32> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn scan_proc(cwd: &Path, self_pid: u32, pred: fn(&str) -> bool) -> Vec<u32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ent in entries.flatten() {
        let pid: u32 = match ent.file_name().to_str().and_then(|s| s.parse().ok()) {
            Some(p) => p,
            None => continue,
        };
        if pid == self_pid {
            continue;
        }
        let Some(pcwd) = proc_cwd(pid) else { continue };
        if pcwd != cwd {
            continue;
        }
        let cmd = proc_cmdline(pid);
        if pred(&cmd) {
            out.push(pid);
        }
    }
    out
}

pub fn is_resuma_dev_watch(cmdline: &str) -> bool {
    let c = cmdline;
    if c.contains("cargo-watch") {
        return true;
    }
    let tokens: Vec<&str> = c.split_whitespace().collect();
    let has_cargo = tokens
        .iter()
        .any(|w| *w == "cargo" || w.ends_with("/cargo"));
    let has_watch = tokens.contains(&"watch");
    has_cargo && has_watch && (c.contains("watch-prepare") || c.contains("--watch-when-idle"))
}

pub fn is_orphaned_dev_bin(cmdline: &str) -> bool {
    if is_resuma_cli_invocation(cmdline) {
        return false;
    }
    if cmdline.contains("target/debug/build") || cmdline.contains("target/debug/deps") {
        return false;
    }
    if cmdline.contains("rustc") || cmdline.contains("clippy") {
        return false;
    }
    cmdline.contains("/target/debug/") || cmdline.contains("\\target\\debug\\")
}

pub fn is_resuma_cli_invocation(cmdline: &str) -> bool {
    let tokens: Vec<&str> = cmdline.split_whitespace().collect();
    let has_cli = tokens
        .iter()
        .any(|w| *w == "resuma" || w.ends_with("/resuma") || w.ends_with("\\resuma.exe"));
    if !has_cli {
        return false;
    }
    tokens.iter().any(|w| {
        matches!(
            *w,
            "dev"
                | "watch-prepare"
                | "build"
                | "new"
                | "routes"
                | "doctor"
                | "install"
                | "update"
                | "add"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pid_and_cwd() {
        assert_eq!(
            parse_dev_pidfile("42\n/tmp/app\n"),
            Some((42, "/tmp/app".into()))
        );
        assert_eq!(parse_dev_pidfile("nope"), None);
        assert_eq!(parse_dev_pidfile("7\n"), Some((7, String::new())));
    }

    #[test]
    fn watch_cmdline_detects_cargo_watch() {
        assert!(is_resuma_dev_watch(
            "/home/me/.cargo/bin/cargo-watch -q -s resuma watch-prepare && cargo run"
        ));
        assert!(is_resuma_dev_watch(
            "cargo watch -q --watch-when-idle -s 'resuma watch-prepare && cargo run'"
        ));
        assert!(!is_resuma_dev_watch("cargo test"));
        assert!(!is_resuma_dev_watch("cargo build"));
    }

    #[test]
    fn dev_bin_skips_compiler_and_deps() {
        assert!(is_orphaned_dev_bin("/proj/target/debug/website"));
        assert!(!is_orphaned_dev_bin("/proj/target/debug/deps/foo-abc"));
        assert!(!is_orphaned_dev_bin(
            "/proj/target/debug/build/bar/build-script-build"
        ));
        assert!(!is_orphaned_dev_bin("rustc --crate-name website"));
        assert!(!is_orphaned_dev_bin(
            "/proj/target/debug/resuma dev --addr 127.0.0.1:3000"
        ));
    }
}
