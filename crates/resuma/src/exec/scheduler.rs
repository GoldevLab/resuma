//! Disk-backed cron scheduler — enqueues workers on a schedule (no external cron).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::task;
use tracing::{info, warn};

use crate::core::{Result, ResumaError};

use super::cron;
use super::id;
use super::queue;
use super::security::{validate_resource_name, validate_schedule_id};

static ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);
static STARTED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

/// A scheduled job persisted under `{RESUMA_DATA_DIR}/scheduler/jobs/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleJob {
    pub id: String,
    pub name: String,
    pub cron: String,
    pub worker: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default = "default_queue")]
    pub queue: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub last_run_ms: Option<u64>,
    pub next_run_ms: u64,
    pub created_ms: u64,
    pub run_count: u64,
}

fn default_queue() -> String {
    "default".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleBody {
    pub name: String,
    pub cron: String,
    pub worker: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default = "default_queue")]
    pub queue: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleListResponse {
    pub jobs: Vec<ScheduleJob>,
    pub total: usize,
}

/// Configure scheduler storage root.
pub fn configure(root: impl AsRef<Path>) {
    let p = root.as_ref().to_path_buf();
    let _ = fs::create_dir_all(p.join("jobs"));
    *ROOT.write() = Some(p);
}

fn root_dir() -> PathBuf {
    ROOT.read()
        .clone()
        .unwrap_or_else(|| PathBuf::from(".resuma/scheduler"))
}

fn jobs_dir() -> PathBuf {
    root_dir().join("jobs")
}

fn job_path(id: &str) -> Result<PathBuf> {
    validate_schedule_id(id)?;
    Ok(jobs_dir().join(format!("{id}.json")))
}

fn firing_path(id: &str) -> Result<PathBuf> {
    validate_schedule_id(id)?;
    Ok(jobs_dir().join("firing").join(format!("{id}.json")))
}

/// Atomically claim a due job for firing (multi-process safe).
fn try_claim_job(id: &str) -> Result<ScheduleJob> {
    let src = job_path(id)?;
    if !src.exists() {
        return Err(ResumaError::validation("job not found"));
    }
    let firing_dir = jobs_dir().join("firing");
    fs::create_dir_all(&firing_dir).map_err(ResumaError::Io)?;
    let dst = firing_path(id)?;
    if dst.exists() {
        return Err(ResumaError::validation("job already claimed"));
    }
    fs::rename(&src, &dst).map_err(ResumaError::Io)?;
    let data = fs::read_to_string(&dst).map_err(ResumaError::Io)?;
    serde_json::from_str(&data).map_err(ResumaError::Serde)
}

fn release_firing_claim(id: &str) -> Result<()> {
    let dst = firing_path(id)?;
    if dst.exists() {
        fs::remove_file(dst).map_err(ResumaError::Io)?;
    }
    Ok(())
}

/// Move a claimed job from `firing/` back to `jobs/` after a failed fire attempt.
fn restore_firing_claim(job: &ScheduleJob) -> Result<()> {
    let dst = firing_path(&job.id)?;
    let src = job_path(&job.id)?;
    if dst.exists() {
        fs::rename(&dst, &src).map_err(ResumaError::Io)?;
    }
    Ok(())
}

fn tick_secs() -> u64 {
    std::env::var("RESUMA_SCHEDULER_TICK_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

/// Start the background scheduler tick loop (idempotent).
pub async fn start() {
    let mut started = STARTED.lock().await;
    if *started {
        return;
    }
    *started = true;
    recover_firing_jobs();
    recompute_all_next_runs();
    task::spawn(async move {
        scheduler_loop().await;
    });
    info!("resuma disk scheduler started");
}

async fn scheduler_loop() {
    let interval = std::time::Duration::from_secs(tick_secs());
    loop {
        if let Err(e) = tick().await {
            warn!(error = %e, "scheduler tick error");
        }
        tokio::time::sleep(interval).await;
    }
}

/// Run due jobs once (also called from tick loop).
pub async fn tick() -> Result<usize> {
    let now = id::now_ms();
    let mut fired = 0;
    for job in list_jobs()? {
        if !job.enabled || job.next_run_ms > now {
            continue;
        }
        let Ok(mut claimed) = try_claim_job(&job.id) else {
            continue;
        };
        match fire_job(&mut claimed).await {
            Ok(()) => {
                fired += 1;
                if let Err(e) = release_firing_claim(&claimed.id) {
                    warn!(job = %claimed.id, error = %e, "failed to release firing claim");
                }
            }
            Err(e) => {
                warn!(job = %claimed.id, error = %e, "scheduled job failed");
                if let Err(re) = restore_firing_claim(&claimed) {
                    warn!(job = %claimed.id, error = %re, "failed to restore firing job");
                } else if let Err(pe) = persist_job(&claimed) {
                    warn!(job = %claimed.id, error = %pe, "failed to persist restored job");
                }
            }
        }
    }
    Ok(fired)
}

async fn fire_job(job: &mut ScheduleJob) -> Result<()> {
    validate_resource_name(&job.worker)?;
    validate_resource_name(&job.queue)?;
    let now = id::now_ms();
    let schedule = cron::parse(&job.cron)?;
    match queue::enqueue(&job.queue, &job.worker, job.input.clone()).await {
        Ok(_) => {
            job.last_run_ms = Some(now);
            job.run_count += 1;
            job.next_run_ms = schedule.next_after_ms(now);
            persist_job(job)?;
            info!(
                job = %job.id,
                worker = %job.worker,
                queue = %job.queue,
                next_run_ms = job.next_run_ms,
                "scheduled job enqueued"
            );
            Ok(())
        }
        Err(e) => {
            job.next_run_ms = schedule.next_after_ms(now);
            persist_job(job)?;
            Err(e)
        }
    }
}

/// Create and persist a new scheduled job.
pub fn create(body: CreateScheduleBody) -> Result<ScheduleJob> {
    if body.name.is_empty() || body.name.len() > 128 {
        return Err(ResumaError::validation("invalid schedule name length"));
    }
    validate_resource_name(&body.worker)?;
    validate_resource_name(&body.queue)?;
    super::security::validate_input(&body.input)?;
    let schedule = cron::parse(&body.cron)?;
    let now = id::now_ms();
    let job = ScheduleJob {
        id: format!("s_{}", crate::server::security::random_token()),
        name: body.name,
        cron: body.cron,
        worker: body.worker,
        input: body.input,
        queue: body.queue,
        enabled: body.enabled,
        last_run_ms: None,
        next_run_ms: schedule.next_after_ms(now.saturating_sub(60_000)),
        created_ms: now,
        run_count: 0,
    };
    persist_job(&job)?;
    Ok(job)
}

/// Remove a scheduled job by id.
pub fn remove(id: &str) -> Result<bool> {
    validate_schedule_id(id)?;
    let path = job_path(id)?;
    if path.exists() {
        fs::remove_file(path).map_err(ResumaError::Io)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get a job by id.
pub fn get(id: &str) -> Option<ScheduleJob> {
    let path = job_path(id).ok()?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// List all scheduled jobs.
pub fn list_jobs() -> Result<Vec<ScheduleJob>> {
    let dir = jobs_dir();
    let _ = fs::create_dir_all(&dir);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut jobs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(job) = serde_json::from_str::<ScheduleJob>(&data) {
                jobs.push(job);
            }
        }
    }
    jobs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(jobs)
}

pub fn list_response() -> Result<ScheduleListResponse> {
    let jobs = list_jobs()?;
    let total = jobs.len();
    Ok(ScheduleListResponse { jobs, total })
}

pub fn stats() -> SchedulerStats {
    let jobs = list_jobs().unwrap_or_default();
    let enabled = jobs.iter().filter(|j| j.enabled).count();
    let due = jobs
        .iter()
        .filter(|j| j.enabled && j.next_run_ms <= id::now_ms())
        .count();
    SchedulerStats {
        total: jobs.len(),
        enabled,
        due,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SchedulerStats {
    pub total: usize,
    pub enabled: usize,
    pub due: usize,
}

fn persist_job(job: &ScheduleJob) -> Result<()> {
    let dir = jobs_dir();
    fs::create_dir_all(&dir).map_err(ResumaError::Io)?;
    let path = job_path(&job.id)?;
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_string_pretty(job)?;
    {
        let mut f = fs::File::create(&tmp).map_err(ResumaError::Io)?;
        f.write_all(data.as_bytes()).map_err(ResumaError::Io)?;
        f.sync_all().map_err(ResumaError::Io)?;
    }
    fs::rename(&tmp, &path).map_err(ResumaError::Io)?;
    Ok(())
}

fn recover_firing_jobs() {
    let firing_dir = jobs_dir().join("firing");
    let Ok(entries) = fs::read_dir(&firing_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            let Ok(dest) = job_path(stem) else {
                continue;
            };
            if dest.exists() {
                // Stale firing claim after successful persist — drop orphan.
                let _ = fs::remove_file(&path);
            } else {
                let _ = fs::rename(&path, &dest);
            }
        }
    }
}

fn recompute_all_next_runs() {
    let now = id::now_ms();
    let Ok(jobs) = list_jobs() else {
        return;
    };
    for mut job in jobs {
        if let Ok(schedule) = cron::parse(&job.cron) {
            if job.next_run_ms < now.saturating_sub(60_000) {
                job.next_run_ms = schedule.next_after_ms(now.saturating_sub(60_000));
                let _ = persist_job(&job);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_scheduler() -> PathBuf {
        let p = std::env::temp_dir().join(format!("resuma-sched-{}", id::next_id()));
        let _ = std::fs::remove_dir_all(&p);
        configure(&p);
        p
    }

    #[test]
    fn create_persists_job() {
        let _guard = super::super::queue_disk::test_queue_lock().lock();
        let _root = temp_scheduler();
        let job = create(CreateScheduleBody {
            name: "nightly".into(),
            cron: "@hourly".into(),
            worker: "test_worker".into(),
            input: json!({}),
            queue: "default".into(),
            enabled: true,
        })
        .expect("create");
        assert!(get(&job.id).is_some());
        assert!(list_jobs().unwrap().iter().any(|j| j.id == job.id));
    }

    #[test]
    fn remove_job() {
        let _guard = super::super::queue_disk::test_queue_lock().lock();
        let _root = temp_scheduler();
        let job = create(CreateScheduleBody {
            name: "x".into(),
            cron: "@daily".into(),
            worker: "w".into(),
            input: json!({}),
            queue: "default".into(),
            enabled: true,
        })
        .unwrap();
        assert!(remove(&job.id).unwrap());
        assert!(get(&job.id).is_none());
    }

    #[test]
    fn remove_rejects_path_traversal() {
        let _guard = super::super::queue_disk::test_queue_lock().lock();
        let _root = temp_scheduler();
        assert!(remove("../../etc/passwd").is_err());
    }
}
