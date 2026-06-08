//! Resuma Extensions — optional CLI-scaffolded integrations (core stays lean).

use crate::audit_shell::AuditStatus;

#[derive(Clone, Copy)]
pub struct ResumaExtension {
    pub id: &'static str,
    pub name: &'static str,
    pub cli: &'static str,
    pub env: Option<&'static str>,
    pub audit_href: &'static str,
    pub status: AuditStatus,
    pub summary: &'static str,
}

pub const EXTENSIONS: &[ResumaExtension] = &[
    ResumaExtension {
        id: "sqlx",
        name: "SQLx",
        cli: "resuma add sqlx",
        env: Some("DATABASE_URL"),
        audit_href: "/audit/integrations/sqlx",
        status: AuditStatus::Pass,
        summary: "SQLite/Postgres pool, migrations, db.rs scaffold",
    },
    ResumaExtension {
        id: "turso",
        name: "Turso / libSQL",
        cli: "resuma add turso",
        env: Some("TURSO_DATABASE_URL"),
        audit_href: "/audit/integrations/turso",
        status: AuditStatus::Info,
        summary: "Edge SQLite via libsql client",
    },
    ResumaExtension {
        id: "tailwind",
        name: "Tailwind CSS",
        cli: "resuma add tailwind",
        env: None,
        audit_href: "/audit/integrations/tailwind",
        status: AuditStatus::Info,
        summary: "PostCSS + Tailwind config scaffold",
    },
    ResumaExtension {
        id: "virtual_list",
        name: "Virtual List",
        cli: "cookbook demo",
        env: None,
        audit_href: "/audit/cookbook/virtual_list",
        status: AuditStatus::Pass,
        summary: "Windowed list rendering for large datasets",
    },
    ResumaExtension {
        id: "e2e",
        name: "E2E Testing",
        cli: "see example-e2e",
        env: None,
        audit_href: "/audit/integrations/e2e",
        status: AuditStatus::Info,
        summary: "Playwright-style browser tests against a running app",
    },
    ResumaExtension {
        id: "auth",
        name: "Auth patterns",
        cli: "manual / security module",
        env: Some("RESUMA_TODO_ADMINS"),
        audit_href: "/audit/integrations/auth",
        status: AuditStatus::Pass,
        summary: "Session cookie + middleware guards (demo)",
    },
    ResumaExtension {
        id: "validator",
        name: "Validation",
        cli: "SubmitError + DTO pipes",
        env: None,
        audit_href: "/audit/integrations/validator",
        status: AuditStatus::Pass,
        summary: "Field errors on #[submit] and server DTOs",
    },
];

pub fn extension_by_id(id: &str) -> Option<&'static ResumaExtension> {
    EXTENSIONS.iter().find(|e| e.id == id)
}
