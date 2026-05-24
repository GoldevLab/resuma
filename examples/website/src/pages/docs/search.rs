use resuma::prelude::*;

use crate::site::search as search_docs;

pub fn page(req: FlowRequest) -> View {
    let q = req.query_param("q").unwrap_or("").to_string();
    let results = search_docs(&q);

    view! {
        <>
            <h1>"Search docs"</h1>
            <p class="lead">"Server-side search — no client JavaScript required."</p>
            <form method="get" action="/docs/search" class="docs-search-form">
                <input
                    type="search"
                    name="q"
                    value={q.clone()}
                    placeholder="Search integrations, Flow, security..."
                />
                <button type="submit">"Search"</button>
            </form>
            {if q.is_empty() {
                view! { <p>"Enter a term or browse " {results.len().to_string()} " indexed pages."</p> }
            } else {
                view! {
                    <p>{format!("{} result(s) for \"{}\"", results.len(), q)}</p>
                }
            }}
            <ul class="docs-search-results">
                {results.iter().map(|e| view! {
                    <li key={e.href.to_string()}>
                        <a href={e.href.to_string()}>
                            <strong>{e.title.to_string()}</strong>
                            <span>{e.href.to_string()}</span>
                        </a>
                    </li>
                }).collect::<Vec<_>>()}
            </ul>
        </>
    }
}
