use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Context"</h1>
            <p class="lead">"Context passes serializable values down the component tree without prop drilling."</p>

            <h2>"Define a context"</h2>
            {code_block(r#"#[derive(Clone, Serialize, Deserialize)]
struct Locale {
    lang: String,
}

static LOCALE: ContextId<Locale> = ContextId::new();"#)}

            <h2>"Provide and consume"</h2>
            {code_block(r#"#[component]
fn App() -> View {
    provide_context(&LOCALE, Locale { lang: "en".into() });
    view! { <Page /> }
}

#[component]
fn Page() -> View {
    let locale = use_context(&LOCALE);
    view! {
        <p>"Language: " {locale.lang.clone()}</p>
    }
}"#)}

            <h2>"Resumability"</h2>
            <p>"Context values are serialized in the resumability payload. Descendants can read them on the client after resume without re-fetching from the server."</p>

            <h2>"Theme helper"</h2>
            <p>"For theming, see " <a href="/docs/cookbook/theme">"Theme cookbook"</a> " which wraps context with " <code>"provide_theme"</code> " / " <code>"use_theme"</code>"."</p>
        </>
    }
}
