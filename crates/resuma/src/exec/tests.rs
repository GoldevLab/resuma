#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::exec::{
        id, plan, FlowEngine, PlannerHints, Resources, WorkerMeta, WorkerRegistry,
    };

    fn temp_durable(name: &str) -> std::path::PathBuf {
        let _guard = crate::exec::queue_disk::test_queue_lock().lock();
        let p = std::env::temp_dir().join(format!("resuma-test-{name}-{}", id::next_id()));
        let _ = std::fs::remove_dir_all(&p);
        crate::exec::durable::configure(&p);
        p
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
        WorkerRegistry::new()
            .register(
                "test_echo",
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

        let started = FlowEngine::start("test_echo", json!({ "x": 1 }))
            .await
            .expect("start");
        // Allow background task to finish.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let snap = FlowEngine::snapshot(&started.graph_id).expect("snapshot");
        assert_eq!(snap.worker, "test_echo");
        let events = FlowEngine::replay(&started.graph_id).expect("replay");
        assert!(!events.is_empty());
    }

    #[tokio::test]
    async fn map_reduce_runs_parallel_chunks() {
        let _root = temp_durable("durable");

        WorkerRegistry::new()
            .register(
                "merge_echo",
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
            "merge_echo",
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
        temp_durable("cancel");

        WorkerRegistry::new()
            .register(
                "slow",
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

        let started = FlowEngine::start("slow", json!({}))
            .await
            .expect("start");

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
        temp_durable("pause");

        WorkerRegistry::new()
            .register(
                "pausable",
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

        let started = FlowEngine::start("pausable", json!({ "n": 1 }))
            .await
            .expect("start");
        tokio::time::sleep(std::time::Duration::from_millis(60)).await;

        FlowEngine::pause(&started.graph_id).expect("pause");
        let paused = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(paused.status, crate::exec::GraphStatus::Paused);

        FlowEngine::resume(&started.graph_id)
            .await
            .expect("resume");
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        let done = FlowEngine::snapshot(&started.graph_id).expect("snap");
        assert_eq!(done.status, crate::exec::GraphStatus::Done);
        let events = FlowEngine::replay(&started.graph_id).expect("events");
        assert!(events.len() >= 2);
    }

    #[tokio::test]
    async fn cancel_marks_graph_failed_and_blocks_resume() {
        temp_durable("hard-cancel");

        WorkerRegistry::new()
            .register(
                "cancel_me",
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

        let started = FlowEngine::start("cancel_me", json!({}))
            .await
            .expect("start");
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
