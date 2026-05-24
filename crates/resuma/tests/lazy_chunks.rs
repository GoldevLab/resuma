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
    let client = full.for_client();
    assert!(!client.handlers.contains_key("Clicker"));
    assert!(client.lazy_chunks.iter().any(|c| c == "Clicker"));
}
