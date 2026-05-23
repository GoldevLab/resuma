# Backend patterns (NestJS + Next.js → Resuma)

Every pattern is implemented in **`examples/todo`**. Run:

```bash
cargo run -p example-todo
```

## Mapping

| NestJS / Next.js | Resuma (Rust) | Todo location |
|------------------|---------------|---------------|
| Next.js Server Actions | `#[server]` | `main.rs` |
| Next.js `revalidatePath` / refetch | `list_todos` action | `main.rs` + island |
| Next.js `middleware.ts` | `set_action_middleware` / `#[middleware]` | `security.rs` |
| NestJS Controller | Thin `#[server]` fn | `main.rs` |
| NestJS Service | Domain module | `todo_store.rs` |
| NestJS Guard | `attach_session()` | `security.rs` |
| NestJS ValidationPipe | DTO + `validate()` | `AddTodoInput` |
| NestJS Interceptor | Request id + audit log | `security.rs` |
| NestJS ExceptionFilter | `ResumaError` → HTTP | `Result<T>` actions |
| NestJS ThrottlerModule | `SecurityConfig` rate limits | `serve_options()` |
| Helmet | `SecurityConfig` headers | Framework |

## Controller → Service

```rust
// main.rs — controller
#[server]
async fn add_todo(title: String, req: &FlowRequest) -> Result<Vec<Todo>> {
    todo_store::add(title, req)
}

// todo_store.rs — service
pub fn add(title: String, req: &FlowRequest) -> Result<Vec<Todo>> { ... }
```

## Server Action + revalidate

Island loads fresh data via `list_todos` on mount (Next.js refetch pattern):

```rust
use_visible_task(|| {
    js! {
        (async () => {
            const next = await __resuma.action("list_todos", []);
            state.todos.set(next);
        })();
    }
});
```

## Defense in depth

1. CDN / WAF rate limits
2. HTTPS + `RESUMA_TRUST_PROXY=1`
3. Resuma `SecurityConfig` (CSRF, headers)
4. Action middleware (guard + interceptor + API key)
5. Service-layer validation + authorization
6. Postgres RLS when you add a database

See [SECURITY.md](./SECURITY.md) and `/docs/security/todo`.
