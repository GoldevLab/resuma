use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"Theme"</h1>
            <p class="lead">"Provide theme tokens via context and expose CSS variables for consistent styling."</p>

            <h2>"Provide theme"</h2>
            {code_block(r##"#[layout("/")]
fn AppLayout() -> View {
    provide_theme(Theme {
        mode: "dark".into(),
        primary: "#6366f1".into(),
        background: "#0b1020".into(),
        foreground: "#e6e8ee".into(),
    });

    view! {
        <div class="app" style={theme_css_vars(&use_theme())}>
            <Slot />
        </div>
    }
}"##)}

            <h2>"Consume in components"</h2>
            {code_block(r#"#[component]
fn ThemedButton() -> View {
    let theme = use_theme();
    view! {
        <button style={format!("background: {}", theme.primary)}>
            "Click"
        </button>
    }
}"#)}

            <h2>"Toggle mode"</h2>
            {code_block(r#"let dark = use_signal(true);

view! {
    <button onClick={ move |_| dark.update(|d| *d = !*d) }>
        "Toggle theme"
    </button>
}"#)}
        </>
    }
}
