use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Bundle benchmark"</h1>
            <p class="lead">
                "Measured JavaScript/WASM for a counter page (SSR heading + increment button) across "
                <strong>"Resuma"</strong>", " <strong>"Qwik"</strong>", " <strong>"Leptos"</strong>", "
                <strong>"Astro"</strong>", " <strong>"Next.js"</strong>", " <strong>"SvelteKit"</strong>", "
                <strong>"SolidStart"</strong>", " <strong>"React (Vite)"</strong>", and "
                <strong>"templ + HTMX"</strong>". "
                "Static Resuma pages ship " <strong>"zero"</strong> " client JS."
            </p>

            <h2>"Summary (gzip)"</h2>
            <table class="compare">
                <thead>
                    <tr>
                        <th>"Framework"</th>
                        <th>"Initial load"</th>
                        <th>"First interaction"</th>
                        <th>"Static page"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><strong>"Resuma"</strong></td>
                        <td class="yes">"901 B"</td>
                        <td class="yes">"4.20 KiB"</td>
                        <td class="yes">"0 B"</td>
                    </tr>
                    <tr>
                        <td>"Qwik 1.20"</td>
                        <td>"1.96 KiB"</td>
                        <td>"22.32 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"templ + HTMX 2"</td>
                        <td>"16.21 KiB"</td>
                        <td>"16.21 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"SolidStart 1.2"</td>
                        <td>"16.75 KiB"</td>
                        <td>"16.75 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"SvelteKit 2.57"</td>
                        <td>"27.71 KiB"</td>
                        <td>"27.71 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"Astro 5.7 (React island)"</td>
                        <td>"57.76 KiB"</td>
                        <td>"57.76 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"React 19 (Vite SPA)"</td>
                        <td>"57.99 KiB"</td>
                        <td>"57.99 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"Leptos 0.7"</td>
                        <td>"79.02 KiB"</td>
                        <td>"79.02 KiB"</td>
                        <td>"—"</td>
                    </tr>
                    <tr>
                        <td>"Next.js 16 (App Router)"</td>
                        <td>"142.43 KiB"</td>
                        <td>"142.43 KiB"</td>
                        <td>"—"</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Methodology"</h2>
            <ol>
                <li>"Same UX: SSR heading + one interactive counter button."</li>
                <li>"Production builds in " <code>"benchmark/*-counter"</code> " plus " <code>"runtime/"</code> " for Resuma."</li>
                <li>"Report minified raw + gzip + brotli from build artifacts (simulated compression in " <code>"run.mjs"</code> ")."</li>
                <li>"Initial load = JS required before the page can resume/hydrate interactivity."</li>
                <li>"First interaction = total JS transferred when the user clicks " <code>"+"</code> " (includes lazy chunks where applicable)."</li>
                <li>"Regenerate anytime: " <code>"node benchmark/run.mjs"</code> " → " <code>"benchmark/results.json"</code>"."</li>
            </ol>

            <h2>"What each framework ships"</h2>
            <table class="compare">
                <thead>
                    <tr>
                        <th>"Framework"</th>
                        <th>"Initial"</th>
                        <th>"First click"</th>
                        <th>"Notes"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><strong>"Resuma"</strong></td>
                        <td><code>"loader.js"</code></td>
                        <td><code>"loader.js + core.js"</code></td>
                        <td>"Rust SSR + resumability; static pages = 0 B"</td>
                    </tr>
                    <tr>
                        <td>"Qwik"</td>
                        <td><code>"preloader"</code></td>
                        <td>"preloader + core + route + onClick chunk"</td>
                        <td>"Resumability (JS)"</td>
                    </tr>
                    <tr>
                        <td>"templ + HTMX"</td>
                        <td><code>"htmx.min.js"</code></td>
                        <td>"same (server round-trip on click)"</td>
                        <td>"Go SSR + HTMX; no client app bundle"</td>
                    </tr>
                    <tr>
                        <td>"SolidStart"</td>
                        <td>"client + web + index chunks"</td>
                        <td>"same (full hydration on load)"</td>
                        <td>"Solid SSR + hydration"</td>
                    </tr>
                    <tr>
                        <td>"SvelteKit"</td>
                        <td>"entry + app + runtime chunks"</td>
                        <td>"same"</td>
                        <td>"SSR + client hydration (adapter-static)"</td>
                    </tr>
                    <tr>
                        <td>"Astro"</td>
                        <td>"React island + client runtime"</td>
                        <td>"same"</td>
                        <td><code>"client:load"</code> " React island"</td>
                    </tr>
                    <tr>
                        <td>"React (Vite)"</td>
                        <td>"single SPA bundle"</td>
                        <td>"same"</td>
                        <td>"Client-rendered baseline"</td>
                    </tr>
                    <tr>
                        <td>"Leptos"</td>
                        <td><code>".wasm + glue"</code></td>
                        <td>"same"</td>
                        <td>"Rust SSR + WASM hydrate"</td>
                    </tr>
                    <tr>
                        <td>"Next.js"</td>
                        <td>"firstLoadChunkPaths (App Router)"</td>
                        <td>"same"</td>
                        <td>"React SSR + hydration; default create-next-app scaffold"</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Resuma (split runtime)"</h2>
            <table class="compare">
                <thead>
                    <tr>
                        <th>"Bundle"</th>
                        <th>"When loaded"</th>
                        <th>"Raw"</th>
                        <th>"Gzip"</th>
                        <th>"Brotli"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>"loader.js"</code></td>
                        <td>"Interactive pages only"</td>
                        <td>"1.82 KiB"</td>
                        <td>"901 B"</td>
                        <td>"746 B"</td>
                    </tr>
                    <tr>
                        <td><code>"core.js"</code></td>
                        <td>"First interaction"</td>
                        <td>"8.68 KiB"</td>
                        <td>"3.32 KiB"</td>
                        <td>"2.93 KiB"</td>
                    </tr>
                    <tr>
                        <td><strong>"Static docs page"</strong></td>
                        <td>"Never"</td>
                        <td class="yes">"0 B"</td>
                        <td class="yes">"0 B"</td>
                        <td class="yes">"0 B"</td>
                    </tr>
                </tbody>
            </table>

            <h2>"Reproduce locally"</h2>
            {code_block(r#"node benchmark/run.mjs

# Or individually:
cd runtime && npm run build && npm run size
cd benchmark/qwik-counter && npm run build
cd benchmark/leptos-counter && wasm-pack build --target web --release
cd benchmark/astro-counter && npm run build
cd benchmark/next-counter && npm run build
cd benchmark/sveltekit-counter && npm run build
cd benchmark/solidstart-counter && npm run build
cd benchmark/react-counter && npm run build

curl -H "Accept-Encoding: gzip" http://127.0.0.1:3000/_resuma/benchmark.json
cargo run -p example-counter"#)}

            <h2>"Takeaways"</h2>
            <ul>
                <li><strong>"Resuma:"</strong>" ~901 B gzip initial, ~4 KiB gzip to full interactivity — no WASM, lazy core on first click."</li>
                <li><strong>"Qwik:"</strong>" smallest resumable JS preloader (~2 KiB gzip), core chunk adds ~20 KiB gzip on first click."</li>
                <li><strong>"templ + HTMX:"</strong>" ~16 KiB gzip for HTMX alone; interactivity is a server round-trip, not client hydration."</li>
                <li><strong>"SolidStart / SvelteKit:"</strong>" mid-tier hydration bundles (~17–28 KiB gzip) for a minimal counter."</li>
                <li><strong>"Astro / React:"</strong>" ~58 KiB gzip — React runtime dominates whether island or SPA."</li>
                <li><strong>"Leptos:"</strong>" WASM hydration bundle ~79 KiB gzip even for a minimal counter."</li>
                <li><strong>"Next.js:"</strong>" ~142 KiB gzip first-load JS on default App Router scaffold (includes framework + React runtime)."</li>
                <li><strong>"Static-first:"</strong>" only Resuma skips all client JS on non-interactive pages."</li>
            </ul>
        </>
    }
}
