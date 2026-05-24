//! Light theme for docs and landing.

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

.landing { overflow-x: clip; }

.hero-wrap {
  position: relative;
  background:
    radial-gradient(ellipse 80% 60% at 50% -10%, rgba(113, 44, 249, 0.14), transparent 55%),
    radial-gradient(ellipse 50% 40% at 100% 0%, rgba(5, 80, 174, 0.08), transparent 50%),
    linear-gradient(180deg, var(--bg-subtle) 0%, var(--bg) 100%);
  border-bottom: 1px solid var(--border);
}
.hero {
  max-width: 80rem; margin: 0 auto; padding: 4rem 1.25rem 3.5rem;
  display: grid; gap: 2.5rem; position: relative;
}
@media (min-width: 900px) {
  .hero { grid-template-columns: 1.05fr 0.95fr; align-items: center; padding: 5rem 1.25rem 4rem; }
}
.hero-badge {
  display: inline-flex; align-items: center; gap: 0.45rem;
  padding: 0.3rem 0.75rem; border-radius: 999px;
  background: var(--primary-soft); border: 1px solid #e9d5ff;
  color: var(--primary); font-size: 0.78rem; font-weight: 600; margin-bottom: 1rem;
}
.hero-badge-dot {
  width: 0.45rem; height: 0.45rem; border-radius: 50%;
  background: var(--success); box-shadow: 0 0 0 3px rgba(17, 99, 41, 0.15);
}
.hero h1 {
  font-size: clamp(2.35rem, 5vw, 3.45rem); line-height: 1.08;
  margin: 0 0 1rem; letter-spacing: -0.035em; color: var(--text); font-weight: 800;
}
.hero h1 .accent { color: var(--primary); }
.hero-tagline {
  font-size: clamp(1.15rem, 2.5vw, 1.45rem); font-weight: 600;
  color: var(--muted); margin: -0.35rem 0 1.1rem; line-height: 1.4;
  letter-spacing: -0.02em; max-width: 34rem;
}
.hero-lead {
  color: var(--muted); font-size: 1.15rem; max-width: 34rem; margin: 0 0 1.5rem; line-height: 1.6;
}
.hero-actions { display: flex; flex-wrap: wrap; gap: 0.65rem; margin-bottom: 0; }
.hero-actions .btn-primary { padding: 0.65rem 1.25rem; font-size: 0.95rem; }
.hero-note {
  margin: 1.25rem 0 0; font-size: 0.82rem; color: var(--muted);
}
.hero-note code { font-size: 0.78rem; }
.hero-panel {
  background: var(--bg); border: 1px solid var(--border);
  border-radius: 14px; padding: 0; overflow: hidden;
  box-shadow: 0 24px 48px rgba(31, 35, 40, 0.08), 0 4px 12px rgba(113, 44, 249, 0.06);
}
.hero-panel-top {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.65rem 0.85rem; background: var(--bg-subtle); border-bottom: 1px solid var(--border);
}
.hero-panel-dots { display: flex; gap: 0.35rem; }
.hero-panel-dots span {
  width: 0.55rem; height: 0.55rem; border-radius: 50%;
}
.hero-panel-dots span:first-child { background: #ff5f57; }
.hero-panel-dots span:nth-child(2) { background: #febc2e; }
.hero-panel-dots span:nth-child(3) { background: #28c840; }
.hero-panel-label {
  font-size: 0.72rem; color: var(--muted); font-family: var(--mono);
}
.hero-panel-body { padding: 1rem 1.1rem 1.15rem; }
.hero-panel-body pre.code { margin: 0; border: 0; background: transparent; padding: 0; font-size: 0.82rem; }
.hero-panel-caption {
  margin-top: 0.85rem; padding-top: 0.85rem; border-top: 1px dashed var(--border);
  font-size: 0.8rem; color: var(--muted);
}
.hero-panel-caption strong { color: var(--text); }

.metrics-bar {
  max-width: 80rem; margin: 0 auto; padding: 0 1.25rem 2.5rem;
  display: grid; gap: 0.75rem;
  grid-template-columns: repeat(2, 1fr);
}
@media (min-width: 640px) {
  .metrics-bar { grid-template-columns: repeat(4, 1fr); }
}
.metric-item {
  background: var(--bg); border: 1px solid var(--border); border-radius: 12px;
  padding: 1rem 1.1rem; text-align: center;
}
.metric-item strong {
  display: block; font-size: 1.65rem; letter-spacing: -0.03em;
  color: var(--primary); line-height: 1.1; margin-bottom: 0.25rem;
}
.metric-item span { font-size: 0.78rem; color: var(--muted); font-weight: 500; }

.section { max-width: 80rem; margin: 0 auto; padding: 3rem 1.25rem; }
.section-alt { background: var(--bg-subtle); border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); }
.section-title {
  font-size: clamp(1.65rem, 3vw, 2.15rem); margin: 0 0 0.55rem;
  letter-spacing: -0.03em; font-weight: 800; line-height: 1.15;
}
.section-sub {
  color: var(--muted); margin: 0 0 2rem; max-width: 42rem; font-size: 1.05rem; line-height: 1.55;
}
.section-eyebrow {
  display: inline-block; font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em;
  text-transform: uppercase; color: var(--primary); margin-bottom: 0.5rem;
}

.pillars {
  display: grid; gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}
.pillar {
  background: var(--bg); border: 1px solid var(--border); border-radius: 12px;
  padding: 1.35rem 1.25rem; transition: border-color 0.15s, box-shadow 0.15s;
}
.pillar:hover {
  border-color: #d4b8ff; box-shadow: 0 8px 24px rgba(113, 44, 249, 0.07);
}
.pillar-icon {
  width: 2.5rem; height: 2.5rem; border-radius: 10px;
  background: var(--primary-soft); color: var(--primary);
  display: grid; place-items: center; font-size: 1.15rem; margin-bottom: 0.85rem;
}
.pillar h3 { margin: 0 0 0.45rem; font-size: 1.02rem; color: var(--text); }
.pillar p { margin: 0; color: var(--muted); font-size: 0.88rem; line-height: 1.55; }

.pipeline {
  display: grid; gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}
.pipeline-step {
  position: relative; background: var(--bg); border: 1px solid var(--border);
  border-radius: 12px; padding: 1.25rem 1.15rem 1.15rem;
}
.section-alt .pipeline-step { background: var(--bg); }
.pipeline-num {
  display: inline-flex; align-items: center; justify-content: center;
  width: 1.75rem; height: 1.75rem; border-radius: 8px;
  background: var(--primary); color: white; font-size: 0.82rem; font-weight: 700;
  margin-bottom: 0.75rem;
}
.pipeline-step h3 { margin: 0 0 0.4rem; font-size: 1rem; }
.pipeline-step p { margin: 0; color: var(--muted); font-size: 0.88rem; line-height: 1.5; }

.showcase {
  display: grid; gap: 2rem; align-items: center;
}
@media (min-width: 900px) {
  .showcase { grid-template-columns: 1fr 1fr; gap: 3rem; }
  .showcase-reverse .showcase-copy { order: 2; }
  .showcase-reverse .showcase-code { order: 1; }
}
.showcase-copy h3 {
  font-size: 1.35rem; margin: 0 0 0.65rem; letter-spacing: -0.02em; font-weight: 700;
}
.showcase-copy p { color: var(--muted); margin: 0 0 1rem; line-height: 1.6; }
.showcase-list {
  margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 0.55rem;
}
.showcase-list li {
  display: flex; gap: 0.55rem; align-items: flex-start; color: var(--muted); font-size: 0.92rem;
}
.showcase-list li::before {
  content: "✓"; color: var(--success); font-weight: 700; flex-shrink: 0; margin-top: 0.05rem;
}
.showcase-code .code-window {
  background: var(--bg); border: 1px solid var(--border); border-radius: 12px; overflow: hidden;
  box-shadow: 0 12px 32px rgba(31, 35, 40, 0.06);
}
.showcase-code pre.code { margin: 0; border: 0; border-radius: 0; font-size: 0.8rem; }

.card {
  transition: border-color 0.15s, box-shadow 0.15s;
}
.card:hover {
  border-color: #d4b8ff; box-shadow: 0 6px 20px rgba(113, 44, 249, 0.06);
}
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

.package-diagram {
  display: grid; gap: 1rem; align-items: stretch;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
}
.package-box {
  border: 1px solid var(--border); border-radius: 14px; padding: 1.5rem 1.35rem;
  background: var(--bg);
  box-shadow: 0 4px 16px rgba(31, 35, 40, 0.04);
}
.package-box h3 { margin: 0 0 0.3rem; font-size: 1.1rem; }
.package-box .tag { color: var(--primary); font-size: 0.72rem; font-weight: 700; letter-spacing: 0.06em; }
.package-box ul { margin: 0.85rem 0 0; padding-left: 1.15rem; color: var(--muted); font-size: 0.88rem; line-height: 1.65; }
.package-plus {
  display: none;
}
@media (min-width: 768px) {
  .package-diagram {
    grid-template-columns: 1fr auto 1fr;
  }
  .package-plus {
    display: flex; align-items: center; justify-content: center;
    font-size: 2rem; color: var(--primary); font-weight: 300;
  }
}

.compare-wrap { overflow-x: auto; border-radius: 12px; border: 1px solid var(--border); }
.compare { width: 100%; border-collapse: collapse; font-size: 0.9rem; border: 0; margin: 0; }
.compare th, .compare td {
  padding: 0.85rem 1rem; text-align: left; border-bottom: 1px solid var(--border);
}
.compare th { background: var(--bg-subtle); color: var(--muted); font-weight: 600; font-size: 0.82rem; }
.compare th:last-child { color: var(--primary); }
.compare tr:last-child td { border-bottom: 0; }
.compare td:first-child { font-weight: 600; color: var(--text); white-space: nowrap; }
.compare .yes { color: var(--success); font-weight: 600; }

.cta-section {
  max-width: 80rem; margin: 0 auto; padding: 0 1.25rem 4rem;
}
.cta-banner {
  background: linear-gradient(135deg, var(--primary) 0%, #5a1fd4 55%, var(--accent) 100%);
  border-radius: 16px; padding: 2.5rem 2rem; text-align: center; color: white;
  box-shadow: 0 20px 40px rgba(113, 44, 249, 0.25);
}
.cta-banner h2 {
  margin: 0 0 0.65rem; font-size: clamp(1.5rem, 3vw, 2rem);
  letter-spacing: -0.03em; font-weight: 800; color: white;
}
.cta-banner p { margin: 0 auto 1.5rem; max-width: 32rem; opacity: 0.92; line-height: 1.55; }
.cta-banner .btn-primary {
  background: white; color: var(--primary); font-size: 1rem; padding: 0.7rem 1.5rem;
}
.cta-banner .btn-primary:hover { background: #f8f4ff; color: var(--primary-hover); }
.cta-install {
  margin-top: 1.25rem; font-family: var(--mono); font-size: 0.82rem;
  background: rgba(0, 0, 0, 0.2); display: inline-block; padding: 0.5rem 1rem; border-radius: 8px;
}

.site-footer {
  border-top: 1px solid var(--border); padding: 2rem 1.25rem;
  text-align: center; color: var(--muted); font-size: 0.85rem;
  background: var(--bg-subtle);
}
.site-footer-links { display: flex; flex-wrap: wrap; justify-content: center; gap: 0.35rem 1rem; margin-top: 0.65rem; }
.site-footer a { color: var(--muted); }
.site-footer a:hover { color: var(--primary); }

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
.docs-search-form { display: flex; gap: 0.35rem; margin-bottom: 1rem; }
.docs-search-form input[type="search"] {
  flex: 1; padding: 0.4rem 0.55rem; border: 1px solid var(--border); border-radius: 6px;
  font: inherit; background: var(--bg); color: var(--text);
}
.docs-search-form button {
  padding: 0.4rem 0.65rem; border: 1px solid var(--border); border-radius: 6px;
  background: var(--bg-subtle); cursor: pointer; font: inherit;
}
.docs-search-results { list-style: none; padding: 0; margin: 1rem 0; }
.docs-search-results li { margin-bottom: 0.65rem; }
.docs-search-results a { display: flex; flex-direction: column; gap: 0.15rem; text-decoration: none; }
.docs-search-results span { color: var(--muted); font-size: 0.85rem; }
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

.docs-table {
  width: 100%; border-collapse: collapse; font-size: 0.9rem;
  border: 1px solid var(--border); border-radius: 8px; overflow: hidden;
}
.docs-table th, .docs-table td {
  padding: 0.65rem 0.85rem; text-align: left; border-bottom: 1px solid var(--border);
}
.docs-table th { background: var(--bg-subtle); color: var(--muted); font-weight: 600; }
.docs-table tr:last-child td { border-bottom: 0; }

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
