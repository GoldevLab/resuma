use resuma::prelude::*;

mod exec;
mod workers;

#[data]
struct SaveCountInput {
    value: i32,
}

#[data]
struct SaveCountResult {
    message: String,
}

#[data]
struct ContactForm {
    name: String,
    email: String,
}

#[server]
async fn save_count(input: SaveCountInput) -> Result<SaveCountResult> {
    if input.value < 0 {
        return Err(ResumaError::validation("count cannot be negative"));
    }

    Ok(SaveCountResult {
        message: format!("Saved {}", input.value),
    })
}

#[submit]
async fn contact(data: ContactForm) -> std::result::Result<Redirect, SubmitError> {
    if data.name.trim().is_empty() {
        return Err(SubmitError::new("Fix the form").field("name", "Name is required"));
    }
    if !data.email.contains('@') {
        return Err(SubmitError::new("Fix the form").field("email", "Email is invalid"));
    }

    Ok(Redirect::to("/thanks"))
}

#[data]
struct TodoItem {
    id: i32,
    title: String,
}

#[component]
fn Counter() {
    let count = signal(0_i32);
    let status = signal(String::new());
    let open = signal(true);
    let items = signal(vec![
        TodoItem {
            id: 1,
            title: "Alpha".into(),
        },
        TodoItem {
            id: 2,
            title: "Beta".into(),
        },
    ]);

    view! {
        <section data-testid="counter">
            <p data-testid="count">"Count: " {count}</p>
            <button type="button" data-testid="increment" onClick={count.update(|c| *c += 1)}>
                "Increment"
            </button>
            <button
                type="button"
                data-testid="save-count"
                onClick={js! {
                    const result = await __resuma.action("save_count", [{ value: state.count.value }]);
                    state.status.set(result.message);
                }}
            >
                "Save Count"
            </button>
            <p data-testid="save-status">{status}</p>

            <button
                type="button"
                data-testid="toggle-show"
                onClick={open.update(|v| *v = !*v)}
            >
                "Toggle panel"
            </button>
            <Show when={open}>
                <p data-testid="show-panel">"Panel visible"</p>
            </Show>

            <button
                type="button"
                data-testid="add-item"
                onClick={js! {
                    const list = [...state.items.value];
                    const nextId = list.reduce((m, t) => Math.max(m, t.id), 0) + 1;
                    list.push({ id: nextId, title: "Item " + nextId });
                    state.items.set(list);
                }}
            >
                "Add item"
            </button>
            <ul data-testid="item-list">
                <For each={items} key="id" let:item>
                    <li data-testid="list-item">{item.title.clone()}</li>
                </For>
            </ul>
        </section>
    }
}

#[component]
fn ContactCard() {
    view! {
        <section data-testid="contact-card">
            <Form submit={contact} data-testid="contact-form">
                <label>
                    "Name"
                    <input name="name" aria-label="Name" />
                </label>
                <label>
                    "Email"
                    <input name="email" aria-label="Email" />
                </label>
                <button type="submit">"Send"</button>
            </Form>
        </section>
    }
}

fn nav() -> View {
    view! {
        <nav>
            <NavLink href="/" activeClass="active" class="nav-link">"Home"</NavLink>
            <NavLink href="/about" activeClass="active" class="nav-link">"About"</NavLink>
        </nav>
    }
}

#[component]
fn HomePage() {
    view! {
        <main>
            {nav()}
            <h1>"Resuma E2E Home"</h1>
            <Counter />
            <ContactCard />
        </main>
    }
}

#[component]
fn AboutPage() {
    let n = signal(0_i32);
    let doubled = signal(0_i32);
    // Client-replayable effect (rs2js). It only re-runs if the full mount
    // pipeline (initEffects) runs after SPA navigation — guards the regression
    // where SPA nav re-bound text/attrs but skipped effects/tasks/lazy chunks.
    effect!([n, doubled], move || {
        doubled.set(n.get() * 2);
    });

    view! {
        <main>
            {nav()}
            <h1>"About Resuma E2E"</h1>
            <p data-testid="about-copy">"SPA navigation rendered this page."</p>
            <section data-testid="about-effect">
                <button
                    type="button"
                    data-testid="about-bump"
                    onClick={n.update(|c| *c += 1)}
                >
                    "Bump"
                </button>
                <p data-testid="about-doubled">"Doubled: " {doubled}</p>
            </section>
        </main>
    }
}

#[component]
fn ThanksPage() {
    view! {
        <main>
            {nav()}
            <h1>"Thanks"</h1>
            <p data-testid="thanks-copy">"Form submitted successfully."</p>
        </main>
    }
}

const INLINE_CSS: &str = r#"<style>
body { font-family: system-ui, sans-serif; margin: 2rem auto; max-width: 42rem; line-height: 1.5; }
nav { display: flex; gap: .75rem; margin-bottom: 1rem; }
.active { font-weight: 700; }
section { border: 1px solid #d0d7de; border-radius: 8px; margin: 1rem 0; padding: 1rem; }
button { margin: .35rem .35rem .35rem 0; }
label { display: block; margin: .5rem 0; }
input { margin-left: .5rem; }
.resuma-field-error { color: #b42318; display: block; margin-top: .25rem; }
</style>"#;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    FlowApp::new()
        .with_title("Resuma E2E")
        .with_head(INLINE_CSS)
        .component("/", HomePage)
        .component("/about", AboutPage)
        .component("/thanks", ThanksPage)
        .component("/exec", exec::ExecPage)
        .serve(FlowServeOptions::default())
        .await
}
