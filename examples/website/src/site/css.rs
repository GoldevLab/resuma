//! Qwik-inspired light theme for docs and landing.

pub const SITE_CSS: &str = r#"<style>
:root {
  --bg: #ffffff;
  --bg-subtle: #f6f8fa;
  --bg-card: #ffffff;
  --border: #d8dee4;
  --text: #1f2328;
  --muted: #59636e;
  --primary: #712cf9;
  --primary-hover: #5a1fd4;
  --primary-soft: #f3ecff;
  --accent: #0550ae;
  --success: #116329;
  --danger: #cf222e;
  --mono: ui-monospace, "Cascadia Code", "Fira Code", monospace;
  --sans: "Segoe UI", ui-sans-serif, system-ui, sans-serif;
  --sidebar-w: 17.5rem;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  font-family: var(--sans);
  background: var(--bg);
  color: var(--text);
  margin: 0;
  line-height: 1.65;
  font-size: 16px;
}
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
code, pre { font-family: var(--mono); font-size: 0.9em; }
code {
  background: var(--bg-subtle);
  padding: 0.12rem 0.4rem;
  border-radius: 5px;
  border: 1px solid var(--border);
}
pre.code {
  background: var(--bg-subtle);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem 1.15rem;
  overflow-x: auto;
  margin: 1rem 0;
  line-height: 1.5;
}
pre.code code { background: none; border: 0; padding: 0; }

.site-header {
  position: sticky; top: 0; z-index: 50;
  background: rgba(255, 255, 255, 0.92);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--border);
}
.header-inner {
  max-width: 80rem; margin: 0 auto; padding: 0.75rem 1.25rem;
  display: flex; align-items: center; gap: 1.5rem;
}
.logo {
  font-weight: 700; font-size: 1.15rem; color: var(--text);
  display: flex; align-items: center; gap: 0.5rem;
  text-decoration: none !important;
}
.logo-mark {
  width: 1.85rem; height: 1.85rem; border-radius: 8px;
  background: var(--primary); color: white;
  display: grid; place-items: center; font-size: 0.95rem;
}
.site-nav { display: flex; gap: 1rem; flex: 1; }
.site-nav a { color: var(--muted); font-weight: 500; text-decoration: none; }
.site-nav a.active, .site-nav a:hover { color: var(--text); }
.header-actions { display: flex; gap: 0.5rem; }

.btn {
  display: inline-flex; align-items: center; justify-content: center;
  padding: 0.5rem 1rem; border-radius: 8px; font-weight: 600;
  border: 1px solid transparent; cursor: pointer; text-decoration: none !important;
  font-size: 0.9rem;
}
.btn-primary { background: var(--primary); color: white; }
.btn-primary:hover { background: var(--primary-hover); color: white; text-decoration: none; }
.btn-ghost { background: var(--bg); color: var(--text); border-color: var(--border); }
.btn-ghost:hover { background: var(--bg-subtle); text-decoration: none; }

.hero {
  max-width: 80rem; margin: 0 auto; padding: 3.5rem 1.25rem 2.5rem;
  display: grid; gap: 2rem;
}
@media (min-width: 900px) {
  .hero { grid-template-columns: 1.05fr 0.95fr; align-items: center; }
}
.hero-badge {
  display: inline-block; padding: 0.25rem 0.7rem; border-radius: 999px;
  background: var(--primary-soft); border: 1px solid #e9d5ff;
  color: var(--primary); font-size: 0.8rem; font-weight: 600; margin-bottom: 0.85rem;
}
.hero h1 {
  font-size: clamp(2rem, 4.5vw, 3rem); line-height: 1.15;
  margin: 0 0 0.85rem; letter-spacing: -0.03em; color: var(--text);
}
.hero h1 span { color: var(--primary); }
.hero-lead { color: var(--muted); font-size: 1.1rem; max-width: 36rem; margin: 0 0 1.25rem; }
.hero-actions { display: flex; flex-wrap: wrap; gap: 0.65rem; margin-bottom: 1.5rem; }
.hero-stats { display: flex; gap: 1.75rem; color: var(--muted); font-size: 0.85rem; }
.hero-stats strong { display: block; color: var(--text); font-size: 1rem; }
.hero-panel {
  background: var(--bg-subtle); border: 1px solid var(--border);
  border-radius: 12px; padding: 1rem;
}
.hero-panel-header { display: flex; gap: 0.35rem; margin-bottom: 0.85rem; }
.hero-panel-header span {
  width: 0.6rem; height: 0.6rem; border-radius: 50%; background: #bbb;
}
.hero-panel-header span:first-child { background: #ff5f57; }
.hero-panel-header span:nth-child(2) { background: #febc2e; }
.hero-panel-header span:nth-child(3) { background: #28c840; }

.section { max-width: 80rem; margin: 0 auto; padding: 1.75rem 1.25rem 2.5rem; }
.section-title { font-size: 1.65rem; margin: 0 0 0.5rem; letter-spacing: -0.02em; }
.section-sub { color: var(--muted); margin: 0 0 1.5rem; max-width: 40rem; }
.grid-3 {
  display: grid; gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
}
.card {
  background: var(--bg-card); border: 1px solid var(--border);
  border-radius: 10px; padding: 1.15rem;
}
.card h3 { margin: 0 0 0.4rem; font-size: 1rem; color: var(--text); }
.card p { margin: 0; color: var(--muted); font-size: 0.9rem; }
.card-icon { font-size: 1.35rem; margin-bottom: 0.5rem; }

.compare, .docs-table {
  width: 100%; border-collapse: collapse; font-size: 0.9rem;
  border: 1px solid var(--border); border-radius: 8px; overflow: hidden;
}
.compare th, .compare td, .docs-table th, .docs-table td {
  padding: 0.65rem 0.85rem; text-align: left; border-bottom: 1px solid var(--border);
}
.compare th, .docs-table th { background: var(--bg-subtle); color: var(--muted); font-weight: 600; }
.compare tr:last-child td, .docs-table tr:last-child td { border-bottom: 0; }
.compare .yes { color: var(--success); font-weight: 600; }

.package-diagram {
  display: grid; gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
}
.package-box {
  border: 1px solid var(--border); border-radius: 10px; padding: 1.25rem;
  background: var(--bg-subtle);
}
.package-box h3 { margin: 0 0 0.3rem; }
.package-box .tag { color: var(--primary); font-size: 0.78rem; font-weight: 700; }
.package-box ul { margin: 0.65rem 0 0; padding-left: 1.15rem; color: var(--muted); font-size: 0.88rem; }
.package-plus {
  display: flex; align-items: center; justify-content: center;
  font-size: 1.75rem; color: var(--muted);
}

.docs-shell {
  max-width: 80rem; margin: 0 auto; padding: 1.25rem 1.25rem 3rem;
  display: grid; gap: 2rem;
}
@media (min-width: 960px) {
  .docs-shell { grid-template-columns: var(--sidebar-w) 1fr; align-items: start; }
}
.docs-sidebar {
  position: sticky; top: 4.5rem;
  max-height: calc(100vh - 5.5rem); overflow-y: auto;
  padding-right: 0.5rem;
}
.docs-sidebar h4 {
  margin: 1.25rem 0 0.5rem; font-size: 0.68rem; text-transform: uppercase;
  letter-spacing: 0.07em; color: var(--muted); font-weight: 700;
}
.docs-sidebar h4:first-child { margin-top: 0; }
.docs-sidebar nav { display: flex; flex-direction: column; gap: 0.05rem; margin-bottom: 0.25rem; }
.docs-sidebar a {
  padding: 0.32rem 0.55rem; border-radius: 6px; color: var(--muted);
  font-size: 0.875rem; text-decoration: none; line-height: 1.35;
}
.docs-sidebar a.active {
  background: var(--primary-soft); color: var(--primary); font-weight: 600;
}
.docs-sidebar a:hover { background: var(--bg-subtle); color: var(--text); text-decoration: none; }

.docs-main { min-width: 0; max-width: 48rem; }
.docs-main h1 { font-size: 2rem; margin: 0 0 0.5rem; letter-spacing: -0.02em; color: var(--text); }
.docs-main h2 {
  font-size: 1.25rem; margin: 2rem 0 0.65rem; padding-top: 0.5rem;
  border-top: 1px solid var(--border); color: var(--text);
}
.docs-main h2:first-of-type { border-top: 0; margin-top: 1.25rem; }
.docs-main h3 { font-size: 1.05rem; margin: 1.25rem 0 0.5rem; color: var(--text); }
.docs-main p, .docs-main li { color: var(--muted); }
.docs-main strong { color: var(--text); }
.docs-main ul, .docs-main ol { padding-left: 1.25rem; }
.docs-main .lead { font-size: 1.08rem; color: var(--muted); margin-bottom: 1.25rem; }
.docs-callout {
  border-left: 3px solid var(--primary); background: var(--primary-soft);
  padding: 0.75rem 1rem; border-radius: 0 8px 8px 0; margin: 1rem 0;
  font-size: 0.92rem; color: var(--muted);
}

.site-footer {
  border-top: 1px solid var(--border); padding: 1.75rem 1.25rem;
  text-align: center; color: var(--muted); font-size: 0.85rem;
  background: var(--bg-subtle);
}
.site-footer a { color: var(--muted); }

.playground-grid {
  display: grid; gap: 0.85rem;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  margin: 1rem 0 1.5rem;
}
.playground-card {
  display: block; background: var(--bg); border: 1px solid var(--border);
  border-radius: 10px; padding: 1rem; text-decoration: none !important;
}
.playground-card:hover { border-color: var(--primary); box-shadow: 0 2px 12px rgba(113,44,249,0.08); }
.playground-card h3 { margin: 0 0 0.3rem; color: var(--text); font-size: 1rem; }
.playground-card p { margin: 0 0 0.65rem; color: var(--muted); font-size: 0.88rem; }
.playground-card code {
  display: block; font-size: 0.8rem; color: var(--primary); background: var(--bg-subtle);
  padding: 0.45rem 0.55rem; border-radius: 6px; border: 1px solid var(--border);
}
.template-grid {
  display: grid; gap: 0.65rem; margin: 0.85rem 0 1.25rem;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
}
.template-pill {
  background: var(--bg-subtle); border: 1px solid var(--border); border-radius: 8px;
  padding: 0.65rem 0.85rem; font-size: 0.88rem;
}
.template-pill strong { display: block; color: var(--text); margin-bottom: 0.2rem; }
.template-pill span { color: var(--muted); font-size: 0.82rem; }
</style>"#;
