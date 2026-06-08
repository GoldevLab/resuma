//! Reactive `<Show>` SSR smoke test.

use resuma::prelude::*;

#[test]
fn show_with_signal_get_renders_resuma_show() {
    use resuma::core::context::{with_context, RenderContext, RenderMode};

    #[component]
    fn Demo() -> View {
        let logged_in = signal(false);
        view! {
            <Show when={logged_in.get()}>
                <p>"yes"</p>
            </Show>
        }
    }

    let ctx = RenderContext::new(RenderMode::Ssr);
    let html = with_context(ctx, || {
        resuma::ssr::render_view(&Demo::render(DemoProps::default()))
    });
    assert!(html.contains("<resuma-show"));
    assert!(html.contains("yes"));
}
