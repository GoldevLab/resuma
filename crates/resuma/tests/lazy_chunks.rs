#[tokio::test]
async fn component_handlers_register_as_lazy_chunks() {
    use resuma::prelude::*;

    #[component]
    fn Clicker() -> View {
        let n = use_signal(0_i32);
        view! {
            <button onClick={move |_| n.update(|v| *v += 1)}>"+"</button>
        }
    }

    let ctx = resuma::core::RenderContext::new(resuma::core::RenderMode::Ssr);
    let (html, full) = resuma::core::context::with_context(ctx.clone(), || {
        let view = Clicker::render(ClickerProps::default());
        let html = resuma::ssr::render_view(&view);
        (html, ctx.snapshot_full())
    });

    assert!(!html.contains("data-r-boundary-anchor"));
    assert!(full.handlers.contains_key("Clicker"));
    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["Clicker"]);
    assert!(module.contains("export const h_"));
    assert!(!module.contains("export async ("));

    let client = full.for_client();
    assert!(!client.handlers.contains_key("Clicker"));
    assert!(client.lazy_chunks.iter().any(|c| c == "Clicker"));
}

#[test]
fn visible_tasks_are_bound_to_their_component_instances() {
    use resuma::prelude::*;

    #[component]
    fn LazyTask() -> View {
        let _ = use_visible_task("(state, __resuma) => {}".to_string());
        view! { <p>"task owner"</p> }
    }

    let ctx = resuma::core::RenderContext::new(resuma::core::RenderMode::Ssr);
    let (html, payload) = resuma::core::context::with_context(ctx.clone(), || {
        let view = View::fragment(vec![
            LazyTask::render(LazyTaskProps::default()).into(),
            LazyTask::render(LazyTaskProps::default()).into(),
        ]);
        let html = resuma::ssr::render_view(&view);
        (html, ctx.snapshot_full())
    });

    let mut task_boundaries: Vec<u32> = payload
        .visible_tasks
        .values()
        .map(|task| task.boundary_id.expect("component task has an owner"))
        .collect();
    task_boundaries.sort_unstable();
    assert_eq!(task_boundaries, vec![1, 2]);
    assert!(html.contains(r#"data-r-chunk="LazyTask" data-r-boundary-id="1""#));
    assert!(html.contains(r#"data-r-chunk="LazyTask" data-r-boundary-id="2""#));
    assert_eq!(html.matches("data-r-boundary-anchor").count(), 2);
}

#[tokio::test]
async fn page_handlers_are_always_served_as_chunks() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};

    let ctx = RenderContext::new(RenderMode::Ssr);
    let full = with_context(ctx.clone(), || {
        ctx.register_handler("__page__", "h_page", "return state.count.value;");
        ctx.snapshot_full()
    });

    assert!(full.handlers["__page__"].contains_key("h_page"));
    let client = full.for_client();
    assert!(client.lazy_chunks.iter().any(|c| c == "__page__"));
    assert!(client.handlers.is_empty());

    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["__page__"]);
    assert!(module.contains("export function h_page("));
}

#[test]
fn lifecycle_sources_are_externalized_from_client_payload() {
    use resuma::core::context::{with_context, ClientEffectSpec, RenderContext, RenderMode};
    use std::collections::BTreeMap;

    let ctx = RenderContext::new(RenderMode::Ssr);
    let full = with_context(ctx.clone(), || {
        ctx.register_client_effect(ClientEffectSpec {
            id: 1,
            deps: vec![],
            captures: BTreeMap::new(),
            kind: "effect".into(),
            symbol: "effect_test".into(),
            body: "(state, __resuma) => { state.count.set(2); }".into(),
            target: None,
            debounce_ms: None,
        });
        ctx.register_visible_task(
            "(async (state, __resuma) => { state.count.set(1); })",
            BTreeMap::new(),
        );
        ctx.snapshot_full()
    });
    let client = full.for_client();
    let json = serde_json::to_string(&client).unwrap();
    assert!(client
        .lazy_chunks
        .iter()
        .any(|chunk| chunk == "__effects__"));
    assert!(client
        .lazy_chunks
        .iter()
        .any(|chunk| chunk == "__visible_tasks__"));
    assert!(json.contains("task_"));
    assert!(json.contains("effect_test"));
    assert!(!json.contains("state.count.set"));

    let effects = resuma::server::handler_assets::effect_chunk_module(&full);
    assert!(effects.contains("export const effect_test"));
    assert!(effects.contains("state.count.set(2)"));
    let tasks = resuma::server::handler_assets::visible_task_chunk_module(&full);
    assert!(tasks.contains("export const task_"));
    assert!(tasks.contains("state.count.set(1)"));

    let handler_chunks =
        std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new()));
    let island_chunks =
        std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new()));
    resuma::server::handler_assets::merge_payload_handlers(&handler_chunks, &island_chunks, &full);
    let registered = handler_chunks.read();
    assert!(registered["__effects__"].contains("export const effect_test"));
    assert!(registered["__visible_tasks__"].contains("export const task_"));
}

#[tokio::test]
async fn event_handlers_accept_direct_signal_expressions() {
    use resuma::prelude::*;

    #[component]
    fn Clicker() {
        let n = signal(0_i32);
        view! {
            <button onClick={n.update(|v| *v += 1)}>"+"</button>
        }
    }

    let ctx = resuma::core::RenderContext::new(resuma::core::RenderMode::Ssr);
    let full = resuma::core::context::with_context(ctx.clone(), || {
        let view = Clicker::render(ClickerProps::default());
        resuma::ssr::render_view(&view);
        ctx.snapshot_full()
    });

    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["Clicker"]);
    assert!(module.contains("state.n.update"));
    assert!(module.contains("async (_event, state, __resuma)"));
    assert!(!module.contains("move"));
}

/// Writes a representative counter handler chunk for `benchmark/run.mjs`.
#[test]
fn write_benchmark_counter_handler() {
    use resuma::prelude::*;

    #[component]
    fn Counter() -> View {
        let count = use_signal(0_i32);
        view! {
            <button onClick={move |_| count.update(|v| *v += 1)}>"+"</button>
        }
    }

    let ctx = resuma::core::RenderContext::new(resuma::core::RenderMode::Ssr);
    let full = resuma::core::context::with_context(ctx.clone(), || {
        let view = Counter::render(CounterProps::default());
        resuma::ssr::render_view(&view);
        ctx.snapshot_full()
    });

    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["Counter"]);
    assert!(!module.is_empty());

    if std::env::var_os("RESUMA_WRITE_BENCHMARK_HANDLER").is_some() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmark/.resuma-counter-handler.js");
        std::fs::write(&path, module).expect("write benchmark handler sample");
    }
}
