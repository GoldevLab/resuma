use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Cookbook"</h1>
            <p class="lead">"Practical recipes for common Resuma patterns — copy, adapt, and ship."</p>

            <h2>"Recipes"</h2>
            <div class="grid-3">
                <a href="/docs/cookbook/debouncer" class="card" style="text-decoration: none;">
                    <h3>"Debouncer"</h3>
                    <p>"Rate-limit search input with use_debounce."</p>
                </a>
                <a href="/docs/cookbook/portals" class="card" style="text-decoration: none;">
                    <h3>"Portals"</h3>
                    <p>"Render modals into remote DOM targets."</p>
                </a>
                <a href="/docs/cookbook/view_transitions" class="card" style="text-decoration: none;">
                    <h3>"View transitions"</h3>
                    <p>"Animated route changes with the View Transitions API."</p>
                </a>
                <a href="/docs/cookbook/theme" class="card" style="text-decoration: none;">
                    <h3>"Theme"</h3>
                    <p>"Dark/light tokens via provide_theme."</p>
                </a>
                <a href="/docs/cookbook/streaming_loaders" class="card" style="text-decoration: none;">
                    <h3>"Streaming loaders"</h3>
                    <p>"Deferred SSR for slow data."</p>
                </a>
                <a href="/docs/cookbook/docker" class="card" style="text-decoration: none;">
                    <h3>"Docker deploy"</h3>
                    <p>"Minimal container image for production."</p>
                </a>
                <a href="/docs/cookbook/prg" class="card" style="text-decoration: none;">
                    <h3>"PRG pattern"</h3>
                    <p>"Post/Redirect/Get after form submits."</p>
                </a>
                <a href="/docs/cookbook/loader_invalidation" class="card" style="text-decoration: none;">
                    <h3>"Loader invalidation"</h3>
                    <p>"Refresh stale #[load] data after mutations."</p>
                </a>
            </div>
        </>
    }
}
