# Resuma Flow cookbook

Recipes for the next-generation Flow APIs (query navigation, `public/`, PWA, booking).

## SPA navigation and `#[load]`

Server loaders run on every full navigation. To refresh data when query params change (e.g. `?fecha=`), use SPA nav instead of mutating the DOM only:

```rust
on:change={js! {
    const input = event.target;
    if (!(input instanceof HTMLInputElement) || !input.value) return;
    await __resuma.navigate(__resuma.buildUrl("/reservar", {
        fecha: input.value,
        servicio: new URLSearchParams(location.search).get("servicio"),
    }));
}}
```

**Important:** In `js!` handlers always read form values from `event.target`, not `event.currentTarget` (the latter is often `null` in async handlers).

Built-in helpers:

| API | Use |
|-----|-----|
| `loader_refresh_input(path, param, value, preserve, type, attrs)` | `<input>` / date picker that navigates on change |
| `loader_refresh_form(path, preserve, children)` | GET form → SPA navigate |
| `loader_poll(path, interval_ms)` | Interval SPA `__resuma.invalidate` (live dashboards; skips hidden tabs) |
| `load_all2` / `load_all3` | One boundary for 2–3 `#[load]` values (no nested `load_boundary`) |
| `query_nav_link(path, &[("servicio","corte")], …)` | Nav link with query string |
| `build_query_href` | Build href in Rust |

### Multi-loader pages

```rust
load_all3(
    use_settings_load(),
    use_jobs_load(),
    use_events_load(),
    |settings, jobs, events| render(settings, jobs, events),
    |err| error_page(&FlowError::Loader(err)),
    || View::empty(),
)
```

### Live dashboards (`loader_poll`)

Prefer `loader_poll` over `<meta http-equiv="refresh">`. It registers a visible task that
calls `__resuma.invalidate(path)` on an interval, pauses while the tab is hidden, and
clears the timer on SPA remount:

```rust
view! {
    <div>
        {loader_poll("/", 8_000)}
        // queue / activity that comes from #[load]
    </div>
}
```

## `public/` directory

`FlowApp` serves `{CARGO_MANIFEST_DIR}/public` automatically:

```
public/
  images/hero.jpg   →  GET /images/hero.jpg
  icon-192.png      →  GET /icons/icon-192.png (overrides generated PWA SVG)
```

Opt-in explicit path:

```rust
FlowApp::new().with_public_dir("public")
```

Files are precached when PWA is enabled.

## Theme → PWA colors

```rust
let theme = Theme {
    primary: "#c9a962".into(),
    background: "#0a0908".into(),
    ..Default::default()
};
FlowApp::new()
    .with_theme_pwa(theme.clone())
    // layout still uses provide_theme(theme)
```

## PWA defaults

PWA is on by default (`RESUMA_PWA=0` to disable). Route list + `public/` paths are precached. Override with `.with_pwa(FlowPwaConfig::…)` or `.without_pwa()`.

## Scaffold: booking template

```bash
resuma new my-salon --template flow-booking
```

Demonstrates `#[load]` + query refresh + `#[submit]` redirect.

## Dev UX

```bash
resuma dev                # :3000, or next free port if busy (multi-app safe)
resuma dev --kill-stale   # replace whatever holds the preferred port (Linux)
resuma dev --addr 127.0.0.1:3010
```

Multiple projects can run `resuma dev` at once — the second binds `:3001`, etc.
`cargo watch` also watches `public/` for static file changes during dev.
