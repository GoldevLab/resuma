//! Full API coverage matrix — SSR × client × interactive audit.

use crate::audit_shell::{audit_page, demo_box, AuditStatus};
use resuma::prelude::*;

#[derive(Clone, Copy)]
pub struct MatrixRow {
    pub api: &'static str,
    pub route: &'static str,
    pub ssr: bool,
    pub client: bool,
    pub interactive: bool,
    pub status: AuditStatus,
}

pub const MATRIX: &[MatrixRow] = &[
    row(
        "signal()",
        "/audit/components/signals",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Show",
        "/audit/components/control_flow",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "effect! / computed",
        "/audit/components/effects",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "onClick handlers",
        "/audit/components/handlers",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "js! handlers",
        "/audit/components/js",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "#[server] actions",
        "/audit/components/server",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "#[island]",
        "/audit/components/islands",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "use_store",
        "/audit/components/store",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "error_boundary",
        "/audit/components/error_boundary",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Form #[submit]",
        "/audit/components/form",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Slot",
        "/audit/components/slots",
        true,
        false,
        false,
        AuditStatus::Pass,
    ),
    row(
        "NavLink",
        "/audit/components/nav_link",
        true,
        true,
        false,
        AuditStatus::Pass,
    ),
    row(
        "Context",
        "/audit/components/context",
        true,
        false,
        false,
        AuditStatus::Pass,
    ),
    row(
        "use_visible_task",
        "/audit/components/tasks",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Accessibility",
        "/audit/components/accessibility",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "portal()",
        "/audit/cookbook/portals",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Theme / provide_theme",
        "/audit/cookbook/theme",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Debouncer",
        "/audit/cookbook/debouncer",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "PRG redirect",
        "/audit/cookbook/prg",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Virtual list",
        "/audit/cookbook/virtual_list",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "CSS animations",
        "/audit/cookbook/animations",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Drag & drop",
        "/audit/cookbook/drag_drop",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Image list + loader",
        "/audit/cookbook/image_list",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Clipboard API",
        "/audit/components/clipboard",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Native select",
        "/audit/components/picker",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Network status",
        "/audit/integrations/network",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "localStorage",
        "/audit/integrations/storage",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Pointer / touch",
        "/audit/flow/gestures",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "User presence",
        "/audit/flow/user_presence",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Test registry",
        "/audit/reference/registry",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "#[load]",
        "/audit/flow/loaders",
        true,
        false,
        false,
        AuditStatus::Pass,
    ),
    row(
        "#[load(stream)]",
        "/audit/flow/streaming",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Query params + loader",
        "/audit/flow/query_params",
        true,
        false,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Dynamic route",
        "/audit/flow/users/42",
        true,
        false,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Platform select",
        "/audit/flow/platform",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "PWA static",
        "/audit/flow/pwa",
        true,
        false,
        false,
        AuditStatus::Pass,
    ),
    row(
        "Todo + SQLx RLS",
        "/audit/security/todo",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "Auth middleware",
        "/audit/security/middleware",
        true,
        true,
        true,
        AuditStatus::Pass,
    ),
    row(
        "SQLx extension",
        "/audit/integrations/sqlx",
        true,
        false,
        false,
        AuditStatus::Pass,
    ),
];

const fn row(
    api: &'static str,
    route: &'static str,
    ssr: bool,
    client: bool,
    interactive: bool,
    status: AuditStatus,
) -> MatrixRow {
    MatrixRow {
        api,
        route,
        ssr,
        client,
        interactive,
        status,
    }
}

fn matrix_table() -> Child {
    let pass = MATRIX
        .iter()
        .filter(|r| matches!(r.status, AuditStatus::Pass))
        .count();
    Child::View(view! {
        <>
            <p class="pill">
                {format!("{pass}/{} APIs verified", MATRIX.len())}
            </p>
            <div class="matrix-wrap">
                <table class="matrix-table">
                    <thead>
                        <tr>
                            <th>"API"</th>
                            <th>"SSR"</th>
                            <th>"Client"</th>
                            <th>"Interactive"</th>
                            <th>"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {MATRIX.iter().map(|r| {
                            view! {
                                <tr>
                                    <td>
                                        <NavLink href={r.route.to_string()} activeClass="active">
                                            {r.api.to_string()}
                                        </NavLink>
                                    </td>
                                    <td>{tick(r.ssr)}</td>
                                    <td>{tick(r.client)}</td>
                                    <td>{tick(r.interactive)}</td>
                                    <td>
                                        <span class={format!("badge {}", r.status.class())}>
                                            {r.status.label()}
                                        </span>
                                    </td>
                                </tr>
                            }
                        }).collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </>
    })
}

fn tick(on: bool) -> View {
    view! { <span class={if on { "tick yes" } else { "tick no" }}>{if on { "✓" } else { "—" }}</span> }
}

pub fn matrix(_req: FlowRequest) -> View {
    audit_page(
        "Audit Matrix",
        AuditStatus::Pass,
        "https://resuma-docs.fly.dev/docs/components/testing",
        vec![demo_box(
            "API × SSR × Client × Interactive",
            vec![matrix_table()],
        )],
    )
}
