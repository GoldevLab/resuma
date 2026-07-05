#[cfg(test)]
#[allow(clippy::await_holding_lock)]
#[allow(clippy::module_inception)]
mod tests {
    use serde_json::json;

    use std::time::Duration;

    use crate::exec::{
        id, plan, FlowEngine, GraphId, GraphSnapshot, GraphStatus, PlannerHints, Resources,
        WorkerMeta, WorkerRegistry,
    };

    fn exec_guard() -> parking_lot::MutexGuard<'static, ()> {
        crate::exec::queue_disk::exec_test_lock().lock()
    }

    fn temp_durable(name: &str) -> std::path::PathBuf {
        let _queue = crate::exec::queue_disk::test_queue_lock().lock();
        let p = std::env::temp_dir().join(format!("resuma-test-{name}-{}", id::next_id()));
        let _ = std::fs::remove_dir_all(&p);
        crate::exec::durable::configure(&p);
        p
    }

    /// Poll until the graph reaches a terminal status or timeout.
    async fn wait_for_terminal_snapshot(graph_id: &GraphId, timeout: Duration) -> GraphSnapshot {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(snap) = FlowEngine::snapshot(graph_id) {
                if matches!(
                    snap.status,
                    GraphStatus::Done | GraphStatus::Failed | GraphStatus::Paused
                ) {
                    return snap;
                }
            }
            if tokio::time::Instant::now() >= deadline {
                panic!(
                    "timeout waiting for graph terminal state, last: {:?}",
                    FlowEngine::snapshot(graph_id)
                );
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[test]
    fn planner_map_reduce_for_heavy_intent() {
        let p = plan(
            "analizar 1000 empresas en Montreal con AI",
            PlannerHints::default(),
        );
        assert_eq!(p.strategy, crate::exec::ExecutionStrategy::MapReduce);
        assert!(p.chunks > 1);
    }

    #[tokio::test]
    async fn flow_engine_runs_registered_worker() {
        let _guard = exec_guard();
        let _root = temp_durable("engine_echo");
        let worker = format!("test_echo_{}", id::next_id());

        WorkerRegistry::new()
            .register(
                worker.as_str(),
                WorkerMeta {
                    intent: "echo input".into(),
                    resources: Resources::auto(),
                },
                |input, ctx| {
                    Box::pin(async move {
                        ctx.log("ok");
                        Ok(input)
                    })
                },
            )
            .install();

        let started = FlowEngine::start(&worker, json!({ "x": 1 }))
            .await
            .expect("start");
        let snap = wait_for_terminal_snapshot(&started.graph_id, Duration::from_secs(5)).await;
        assert_eq!(snap.status, GraphStatus::Done);
        assert_eq!(snap.worker, worker);
        let events = FlowEngine::replay(&started.graph_id).expect("replay");
        assert!(!events.is_empty());
    }

    #[tokio::test]
    async fn map_reduce_runs_parallel_chunks() {
        let _guard = exec_guard();
        let _root = temp_durable("durable");
        let worker = format!("merge_echo_{}", id::next_id());

        WorkerRegistry::new()
            .register(
                worker.as_str(),
                WorkerMeta {
                    intent: "analizar 1000 items con AI".into(),
                    resources: Resources::auto(),
                },
                |_input, ctx| {
                    Box::pin(async move {
                        ctx.log("merged");
                        Ok(json!({ "ok": true }))
                    })
                },
            )
            .install();

        let started = FlowEngine::start(
            &worker,
            json!({ "items": [1, 2, 3, 4, 5], "prompt": "sum" }),
        )
        .await
        .expect("start");

        assert_eq!(
            started.plan.strategy,
            crate::exec::ExecutionStrategy::MapReduce
        );

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let snap = FlowEngine::snapshot(&started.graph_id).expect("snapshot");
        assert!(snap.nodes.iter().any(|n| n.id.0.starts_with("ai-")));
    }

    #[tokio::test]
    async fn pause_cancels_slow_worker() {
        let _guard = exec_guard();
        temp_durable("cancel");
        let worker = format!("slow_{}", id::next_id());

        WorkerRegistry::new()
            .register(
                worker.as_str(),
                WorkerMeta {
                    intent: "slow task".into(),
                    resources: Resources::auto(),
                },
                |_input, ctx| {
                    Box::pin(async move {
                        for _ in 0..50 {
                            ctx.check_cancelled()?;
                            tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                        }
                        Ok(json!({ "done": true }))
                    })
                },
            )
            .install();

        let started = FlowEngine::start(&worker, json!({})).await.expect("start");

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        FlowEngine::pause(&started.graph_id).expect("pause");

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        let snap = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(snap.status, crate::exec::GraphStatus::Paused);

        let events = FlowEngine::replay(&started.graph_id).expect("events");
        assert!(events.iter().any(|e| matches!(
            e,
            crate::exec::WorkerEvent::Log { message, .. }
            if message.contains("paused")
        )));
    }

    #[tokio::test]
    async fn pause_and_resume_roundtrip() {
        let _guard = exec_guard();
        temp_durable("pause");
        let worker = format!("pausable_{}", id::next_id());

        WorkerRegistry::new()
            .register(
                worker.as_str(),
                WorkerMeta {
                    intent: "slow pausable task".into(),
                    resources: Resources::auto(),
                },
                |input, ctx| {
                    Box::pin(async move {
                        for _ in 0..40 {
                            ctx.check_cancelled()?;
                            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                        }
                        ctx.state_set("ran", json!(true));
                        Ok(input)
                    })
                },
            )
            .install();

        let started = FlowEngine::start(&worker, json!({ "n": 1 }))
            .await
            .expect("start");
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;

        FlowEngine::pause(&started.graph_id).expect("pause");
        let paused = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(paused.status, crate::exec::GraphStatus::Paused);

        FlowEngine::resume(&started.graph_id).await.expect("resume");
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        let done = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(done.status, crate::exec::GraphStatus::Done);
        let events = FlowEngine::replay(&started.graph_id).expect("events");
        assert!(events.len() >= 2);
    }

    #[tokio::test]
    async fn cancel_marks_graph_failed_and_blocks_resume() {
        let _guard = exec_guard();
        temp_durable("hard-cancel");
        let worker = format!("cancel_me_{}", id::next_id());

        WorkerRegistry::new()
            .register(
                worker.as_str(),
                WorkerMeta {
                    intent: "slow task".into(),
                    resources: Resources::auto(),
                },
                |_input, ctx| {
                    Box::pin(async move {
                        for _ in 0..20 {
                            ctx.check_cancelled()?;
                            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                        }
                        Ok(json!({ "done": true }))
                    })
                },
            )
            .install();

        let started = FlowEngine::start(&worker, json!({})).await.expect("start");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        FlowEngine::cancel(&started.graph_id).expect("cancel");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let snap = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(snap.status, crate::exec::GraphStatus::Failed);
        assert!(matches!(
            FlowEngine::resume(&started.graph_id).await,
            Err(crate::core::ResumaError::Validation(_))
        ));
    }
}
