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
    let full = resuma::core::context::with_context(ctx.clone(), || {
        let view = Clicker::render(ClickerProps::default());
        resuma::ssr::render_view(&view);
        ctx.snapshot_full()
    });

    assert!(full.handlers.contains_key("Clicker"));
    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["Clicker"]);
    assert!(module.contains("export const h_"));
    assert!(!module.contains("export async ("));

    let client = full.for_client();
    assert!(!client.handlers.contains_key("Clicker"));
    assert!(client.lazy_chunks.iter().any(|c| c == "Clicker"));
}

#[tokio::test]
async fn oversized_page_handlers_marked_lazy_and_served_as_chunk() {
    use resuma::core::context::{with_context, RenderContext, RenderMode, INLINE_HANDLER_MAX_BYTES};

    let ctx = RenderContext::new(RenderMode::Ssr);
    let full = with_context(ctx.clone(), || {
        ctx.register_handler("__page__", "h_big", &"x".repeat(INLINE_HANDLER_MAX_BYTES + 1));
        ctx.snapshot_full()
    });

    assert!(full.handlers["__page__"].contains_key("h_big"));
    let client = full.for_client();
    assert!(client.lazy_chunks.iter().any(|c| c == "__page__"));
    assert!(
        client
            .handlers
            .get("__page__")
            .and_then(|m| m.get("h_big"))
            .is_none()
    );

    let module = resuma::server::handler_assets::handler_chunk_module(&full.handlers["__page__"]);
    assert!(module.contains("export function h_big("));
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
