/**
 * Resuma Flow widgets — ops dashboard, live graph, event stream.
 */

export interface WorkerEventBase {
  type: string;
  timestamp_ms: number;
}

export interface WorkerEventLog extends WorkerEventBase {
  type: "log";
  message: string;
  node: { 0: string } | string;
}

export type WorkerEvent = WorkerEventLog | Record<string, unknown>;

export interface ExecStatus {
  ok: boolean;
  uptime_ms: number;
  workers: { registered: number; names: string[] };
  graphs: { active: number; running: number; paused: number };
  queues: Array<{
    queue: string;
    pending: number;
    processing: number;
    done: number;
    failed: number;
  }>;
  scheduler: { total: number; enabled: number; due: number };
}


function resumaAction(name: string, args: unknown[]): Promise<unknown> {
  const r = window.__resuma;
  if (!r?.action) throw new Error("Resuma core not loaded");
  return r.action(name, args);
}

// Long-lived resources (poll timers, SSE connections) created by Flow widgets.
// On SPA navigation the page HTML is swapped and widgets are re-mounted, so we
// must tear down the previous page's timers/EventSources — otherwise every
// navigation leaks a running interval and an open SSE stream to detached DOM.
const flowCleanups: Array<() => void> = [];
const eventStreamOwners = new Map<string, () => void>();
const eventStreamMountGen = new Map<string, number>();
const streamSeenKeys = new WeakMap<HTMLElement, Set<string>>();

type SseListener = (ev: WorkerEvent) => void;
interface GraphSseHub {
  listeners: Set<SseListener>;
  es: EventSource | null;
  refs: number;
  terminal: boolean;
}
const graphSseHubs = new Map<string, GraphSseHub>();
/** Graphs that finished — survives hub teardown so remounts do not reopen SSE replay. */
const completedGraphIds = new Set<string>();

function seenForList(list: HTMLElement): Set<string> {
  let seen = streamSeenKeys.get(list);
  if (!seen) {
    seen = new Set();
    streamSeenKeys.set(list, seen);
  }
  return seen;
}

function closeGraphSseHub(graphId: string): void {
  const hub = graphSseHubs.get(graphId);
  if (!hub) return;
  hub.es?.close();
  hub.es = null;
  graphSseHubs.delete(graphId);
}

function markGraphTerminal(graphId: string): void {
  if (graphId) completedGraphIds.add(graphId);
  const hub = graphSseHubs.get(graphId);
  if (hub) {
    hub.terminal = true;
    hub.es?.close();
    hub.es = null;
  }
}

function registerFlowCleanup(fn: () => void): void {
  flowCleanups.push(fn);
}

function flushFlowCleanups(): void {
  while (flowCleanups.length) {
    const fn = flowCleanups.pop();
    try {
      fn?.();
    } catch {
      /* ignore cleanup errors */
    }
  }
}

function graphIdFrom(el: HTMLElement): string {
  return (
    el.getAttribute("data-r-flow-graph") ??
    el.getAttribute("data-r-event-stream") ??
    el.getAttribute("data-r-worker-panel") ??
    ""
  );
}

function graphTokenFrom(el: HTMLElement): string {
  return el.getAttribute("data-r-graph-token") ?? "";
}

function debounce<T extends (...args: never[]) => void>(fn: T, ms: number): T {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return ((...args: never[]) => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  }) as T;
}

function eventKey(ev: WorkerEvent): string {
  const t = ev.type ?? "event";
  const ts = (ev as { timestamp_ms?: number }).timestamp_ms ?? 0;
  switch (t) {
    case "log":
      // Content-keyed — SSE replay after live stream uses new timestamps.
      return `${t}:${(ev as { message?: string }).message ?? ""}`;
    case "progress":
      return `${t}:${(ev as { value?: number }).value ?? ""}`;
    case "result":
      return `${t}:${JSON.stringify((ev as { data?: unknown }).data ?? null)}`;
    case "node_done":
      return `${t}:${(ev as { duration_ms?: number }).duration_ms ?? ts}`;
    case "graph_done":
      return `${t}:done`;
    default:
      return `${t}:${ts}`;
  }
}

function graphTerminalStatus(status: string | undefined): boolean {
  return status === "done" || status === "failed" || status === "paused";
}

function nodeLabel(node: unknown): string {
  if (typeof node === "string") return node;
  if (node && typeof node === "object" && "0" in node) {
    return String((node as Record<string, string>)["0"]);
  }
  return "node";
}

function formatEvent(ev: WorkerEvent): string {
  const t = ev.type ?? "event";
  switch (t) {
    case "log":
      return `[log] ${(ev as WorkerEventLog).message}`;
    case "progress":
      return `[progress] ${(ev as { value: number }).value}%`;
    case "ai_thinking":
      return `[ai] ${(ev as { content: string }).content}`;
    case "tool_call":
      return `[tool] ${(ev as { tool: string }).tool}`;
    case "node_start":
      return `[start] ${nodeLabel((ev as { node: unknown }).node)}`;
    case "node_done":
      return `[done] ${nodeLabel((ev as { node: unknown }).node)} (${(ev as { duration_ms: number }).duration_ms}ms)`;
    case "node_failed":
      return `[error] ${(ev as { error: string }).error}`;
    case "result":
      return `[result] ${JSON.stringify((ev as { data: unknown }).data)}`;
    case "graph_done":
      return `[graph] complete`;
    default:
      return `[${t}]`;
  }
}

function withGraphToken(path: string, token: string): string {
  if (!token) return path;
  const sep = path.includes("?") ? "&" : "?";
  return `${path}${sep}token=${encodeURIComponent(token)}`;
}

function graphFetchHeaders(token: string): HeadersInit {
  const headers: Record<string, string> = {};
  if (token) headers["X-Resuma-Graph-Token"] = token;
  return headers;
}

function csrfToken(): string {
  const node = document.getElementById("resuma-state");
  if (!node?.textContent) return "";
  try {
    const payload = JSON.parse(node.textContent) as { csrf_token?: string };
    return payload.csrf_token ?? "";
  } catch {
    return "";
  }
}

function graphMutationHeaders(token: string): HeadersInit {
  const headers = graphFetchHeaders(token) as Record<string, string>;
  const csrf = csrfToken();
  if (csrf) headers["x-resuma-csrf"] = csrf;
  return headers;
}

function graphControlPath(graphId: string, action: "pause" | "resume" | "cancel"): string {
  return `/_resuma/graph/${encodeURIComponent(graphId)}/${action}`;
}

async function controlErrorMessage(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as { error?: string };
    if (body.error) return body.error;
  } catch {
    /* ignore */
  }
  if (res.status === 422) {
    return "Graph already finished — click Run worker again, then Pause while Running.";
  }
  if (res.status === 401 || res.status === 403) {
    return "Unauthorized — refresh the page and run the worker again.";
  }
  return `Control failed (HTTP ${res.status}).`;
}

function formatUptime(ms: number): string {
  const s = Math.floor(ms / 1000);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m ${sec}s`;
  return `${sec}s`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

async function fetchExecStatus(): Promise<ExecStatus | null> {
  try {
    const data = await resumaAction("exec_status", []);
    return data as ExecStatus;
  } catch {
    /* fall through */
  }
  try {
    const res = await fetch("/_resuma/status", { credentials: "same-origin" });
    if (!res.ok) return null;
    return (await res.json()) as ExecStatus;
  } catch {
    return null;
  }
}

function renderDashboard(root: HTMLElement, status: ExecStatus): void {
  const pending = status.queues.reduce((n, q) => n + q.pending, 0);
  const processing = status.queues.reduce((n, q) => n + q.processing, 0);
  const badgeClass =
    pending > 10
      ? "r-flow-dash__badge r-flow-dash__badge--warn"
      : status.ok
        ? "r-flow-dash__badge"
        : "r-flow-dash__badge r-flow-dash__badge--err";

  const workerChips =
    status.workers.names.length > 0
      ? status.workers.names
          .map((n) => `<span class="r-flow-dash__chip">${escapeHtml(n)}</span>`)
          .join("")
      : '<span class="r-flow-dash__chip">none registered</span>';

  const queueRows = status.queues
    .map((q) => {
      const total = q.pending + q.processing + q.done + q.failed || 1;
      const pct = Math.round((q.processing / total) * 100);
      return `<tr>
        <td>${escapeHtml(q.queue)}</td>
        <td>${q.pending}</td>
        <td>${q.processing}</td>
        <td>${q.done}</td>
        <td>${q.failed}</td>
        <td><div class="r-flow-dash__bar"><svg viewBox="0 0 100 6" preserveAspectRatio="none" aria-hidden="true"><rect class="r-flow-dash__bar-fill" width="${pct}" height="6" rx="3"/></svg></div></td>
      </tr>`;
    })
    .join("");

  root.innerHTML = `
    <header class="r-flow-dash__header">
      <h2 class="r-flow-dash__title">Resuma OS</h2>
      <span class="${badgeClass}">${status.ok ? "healthy" : "degraded"} · uptime ${formatUptime(status.uptime_ms)}</span>
    </header>
    <div class="r-flow-dash__grid">
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Workers</p><p class="r-flow-dash__stat-value">${status.workers.registered}</p></div>
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Graphs running</p><p class="r-flow-dash__stat-value">${status.graphs.running}</p></div>
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Graphs paused</p><p class="r-flow-dash__stat-value">${status.graphs.paused}</p></div>
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Queue pending</p><p class="r-flow-dash__stat-value">${pending}</p></div>
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Processing</p><p class="r-flow-dash__stat-value">${processing}</p></div>
      <div class="r-flow-dash__stat"><p class="r-flow-dash__stat-label">Scheduler due</p><p class="r-flow-dash__stat-value">${status.scheduler.due}</p></div>
    </div>
    <section class="r-flow-dash__section">
      <h3>Workers</h3>
      <div class="r-flow-dash__chips">${workerChips}</div>
    </section>
    <section class="r-flow-dash__section">
      <h3>Queues</h3>
      <table class="r-flow-dash__table">
        <thead><tr><th>Name</th><th>Pending</th><th>Active</th><th>Done</th><th>Failed</th><th>Load</th></tr></thead>
        <tbody>${queueRows}</tbody>
      </table>
    </section>
    <section class="r-flow-dash__section">
      <h3>Scheduler</h3>
      <p class="r-flow-dash__meta">${status.scheduler.enabled} enabled · ${status.scheduler.total} total · ${status.scheduler.due} due now</p>
    </section>`;
}

function mountFlowDashboard(el: HTMLElement): void {
  const root = el.querySelector<HTMLElement>("[data-r-flow-dashboard-root]") ?? el;
  const pollMs = Number(el.getAttribute("data-r-flow-dashboard-poll") ?? "5000") || 5000;
  const initRaw = el.getAttribute("data-r-flow-dashboard-init");

  const refresh = async () => {
    const status = await fetchExecStatus();
    if (status) {
      renderDashboard(root, status);
      return;
    }
    if (!root.querySelector(".r-flow-dash__header")) {
      root.innerHTML =
        '<p class="r-flow-dash__meta">Could not refresh ops status. The SSR snapshot above may be stale — reload the page.</p>';
    }
  };

  if (initRaw) {
    try {
      renderDashboard(root, JSON.parse(initRaw) as ExecStatus);
    } catch {
      /* ignore */
    }
  }
  void refresh();
  const timer = window.setInterval(() => void refresh(), pollMs);
  const stop = () => clearInterval(timer);
  el.addEventListener("resuma:disconnect", stop, { once: true });
  registerFlowCleanup(stop);
}

function subscribeGraphEvents(
  graphId: string,
  token: string,
  onEvent: (ev: WorkerEvent) => void,
  opts?: { closeOnError?: boolean },
): () => void {
  if (!graphId || typeof EventSource === "undefined") return () => {};
  if (completedGraphIds.has(graphId)) return () => {};

  let hub = graphSseHubs.get(graphId);
  if (!hub) {
    hub = { listeners: new Set(), es: null, refs: 0, terminal: false };
    graphSseHubs.set(graphId, hub);
  }
  if (hub.terminal) return () => {};

  hub.listeners.add(onEvent);
  hub.refs += 1;

  const open = (): void => {
    if (hub!.terminal || hub!.es) return;
    const url = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}/events`, token);
    const es = new EventSource(url);
    hub!.es = es;
    es.onmessage = (msg) => {
      try {
        const ev = JSON.parse(msg.data) as WorkerEvent;
        for (const fn of hub!.listeners) fn(ev);
        if (ev.type === "graph_done") {
          hub!.terminal = true;
          es.close();
          hub!.es = null;
        }
      } catch {
        /* ignore */
      }
    };
    es.onerror = () => {
      es.close();
      hub!.es = null;
    };
  };

  open();

  let closed = false;
  return () => {
    if (closed) return;
    closed = true;
    hub!.listeners.delete(onEvent);
    hub!.refs -= 1;
    if (hub!.refs <= 0) {
      hub!.es?.close();
      graphSseHubs.delete(graphId);
    }
  };
}

function renderGraph(
  el: HTMLElement,
  snapshot: {
    nodes?: Array<{ id: unknown; label: string; status: string }>;
    status?: string;
    worker?: string;
  },
): void {
  const track = el.querySelector<HTMLElement>("[data-r-flow-graph-track]");
  const statusEl = el.querySelector<HTMLElement>("[data-r-flow-graph-status]");
  const nodes = snapshot.nodes ?? [];

  if (track) {
    if (!nodes.length) {
      track.textContent = "No nodes";
    } else {
      track.innerHTML = "";
      nodes.forEach((n, i) => {
        if (i > 0) {
          const arrow = document.createElement("span");
          arrow.className = "r-flow-graph__arrow";
          arrow.textContent = "→";
          track.appendChild(arrow);
        }
        const pill = document.createElement("span");
        pill.className = `r-flow-graph__node r-flow-graph__node--${n.status}`;
        const sym =
          n.status === "done"
            ? "✓"
            : n.status === "running"
              ? "●"
              : n.status === "failed"
                ? "✗"
                : n.status === "paused"
                  ? "‖"
                  : "○";
        pill.textContent = `${sym} ${n.label}`;
        track.appendChild(pill);
      });
    }
  }

  if (statusEl) {
    const worker = snapshot.worker ? ` · ${snapshot.worker}` : "";
    const st = snapshot.status ?? "unknown";
    statusEl.textContent = `Status: ${st}${worker}`;
  }
}

async function refreshGraph(el: HTMLElement, graphId: string, token: string): Promise<void> {
  const statusEl = el.querySelector<HTMLElement>("[data-r-flow-graph-status]");
  if (completedGraphIds.has(graphId)) {
    if (statusEl) statusEl.textContent = "Graph finished.";
    return;
  }
  const path = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}`, token);
  try {
    const res = await fetch(path, { headers: graphFetchHeaders(token), credentials: "same-origin" });
    if (!res.ok) {
      if (statusEl) {
        if (res.status === 401 || res.status === 403) {
          markGraphTerminal(graphId);
          statusEl.textContent = completedGraphIds.has(graphId)
            ? "Graph finished."
            : "Unauthorized — refresh the page and run the worker again.";
        } else {
          statusEl.textContent = `Failed to load graph (HTTP ${res.status}).`;
        }
      }
      return;
    }
    const snap = (await res.json()) as {
      nodes?: Array<{ id: unknown; label: string; status: string }>;
      status?: string;
      worker?: string;
    };
    renderGraph(el, snap);
    if (graphTerminalStatus(snap.status)) {
      markGraphTerminal(graphId);
    }
  } catch {
    if (statusEl) statusEl.textContent = "Network error loading graph.";
  }
}

function mountFlowGraph(el: HTMLElement): void {
  const graphId = graphIdFrom(el);
  const token = graphTokenFrom(el);
  if (!graphId) return;

  if (el.dataset.rFlowMounted === "1") {
    el.dispatchEvent(new Event("resuma:disconnect"));
  }
  el.dataset.rFlowMounted = "1";

  let aborted = false;
  let closeSse: (() => void) | null = null;
  const stop = () => {
    aborted = true;
    closeSse?.();
    closeSse = null;
    delete el.dataset.rFlowMounted;
  };
  el.addEventListener("resuma:disconnect", stop, { once: true });
  registerFlowCleanup(stop);

  const scheduleRefresh = debounce(() => {
    if (!aborted) void refreshGraph(el, graphId, token);
  }, 400);

  void (async () => {
    if (aborted) return;
    await refreshGraph(el, graphId, token);
    if (aborted) return;
    const live = el.getAttribute("data-r-flow-graph-live") === "true";
    if (!live) return;

    try {
      const path = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}`, token);
      const res = await fetch(path, {
        headers: graphFetchHeaders(token),
        credentials: "same-origin",
      });
      if (aborted) return;
      if (res.ok) {
        const snap = (await res.json()) as { status?: string };
        if (graphTerminalStatus(snap.status)) return;
      }
    } catch {
      /* ignore */
    }

    if (aborted) return;

    closeSse = subscribeGraphEvents(graphId, token, (ev) => {
      if (aborted) return;
      if (ev.type === "progress" || ev.type === "log" || ev.type === "ai_thinking") return;
      if (
        ev.type === "graph_done" ||
        ev.type === "node_done" ||
        ev.type === "node_start" ||
        ev.type === "node_failed"
      ) {
        void refreshGraph(el, graphId, token);
        return;
      }
      scheduleRefresh();
    });
  })();
}

function eventStreamViewport(el: HTMLElement): HTMLElement {
  return (
    el.querySelector<HTMLElement>("[data-r-event-stream-viewport]") ??
    el.querySelector<HTMLElement>(".r-event-stream__viewport") ??
    el.querySelector<HTMLElement>(".r-event-stream-list") ??
    el
  );
}

function scrollStreamToEnd(viewport: HTMLElement, smooth = true): void {
  const run = () => {
    // Assign a large value — more reliable than scrollHeight - clientHeight after DOM updates.
    const top = viewport.scrollHeight;
    if (smooth && "scrollTo" in viewport) {
      viewport.scrollTo({ top, behavior: "smooth" });
    } else {
      viewport.scrollTop = top;
    }
  };
  // Flex/backdrop-filter layouts may measure one frame late.
  requestAnimationFrame(() => {
    run();
    requestAnimationFrame(run);
  });
}

function bindStreamAutoScroll(viewport: HTMLElement, list: HTMLElement): () => void {
  if (typeof ResizeObserver === "undefined") return () => {};
  const scroll = () => {
    viewport.scrollTop = viewport.scrollHeight;
  };
  const ro = new ResizeObserver(scroll);
  ro.observe(list);
  return () => ro.disconnect();
}

function mountEventStream(el: HTMLElement): void {
  const graphId = graphIdFrom(el);
  const token = graphTokenFrom(el);
  if (!graphId) return;
  const viewport = eventStreamViewport(el);
  const list = el.querySelector("ul") ?? el;
  list.innerHTML = "";
  const max = 1000;
  const gen = (eventStreamMountGen.get(graphId) ?? 0) + 1;
  eventStreamMountGen.set(graphId, gen);
  const isActive = () => eventStreamMountGen.get(graphId) === gen;
  const seen = seenForList(list as HTMLElement);
  const stopAutoScroll = bindStreamAutoScroll(viewport, list as HTMLElement);

  const append = (line: string, smooth = false) => {
    if (!isActive()) return;
    const li = document.createElement("li");
    li.textContent = line;
    list.appendChild(li);
    while (list.children.length > max) {
      list.removeChild(list.firstChild!);
    }
    scrollStreamToEnd(viewport, smooth);
  };
  const appendEvent = (ev: WorkerEvent, smooth = false) => {
    if (!isActive()) return;
    if (el.dataset.rStreamTerminal === "1" && ev.type !== "graph_done") return;
    const key = eventKey(ev);
    if (seen.has(key)) return;
    seen.add(key);
    append(formatEvent(ev), smooth);
    if (ev.type === "graph_done") {
      el.dataset.rStreamTerminal = "1";
      markGraphTerminal(graphId);
    }
  };

  eventStreamOwners.get(graphId)?.();
  eventStreamOwners.delete(graphId);

  if (el.dataset.rFlowMounted === "1") {
    el.dispatchEvent(new Event("resuma:disconnect"));
  }
  el.dataset.rFlowMounted = "1";

  let aborted = false;
  let closeSse: (() => void) | null = null;
  const stop = () => {
    aborted = true;
    stopAutoScroll();
    closeSse?.();
    closeSse = null;
    delete el.dataset.rFlowMounted;
    if (eventStreamMountGen.get(graphId) === gen) {
      eventStreamMountGen.delete(graphId);
    }
    if (eventStreamOwners.get(graphId) === stop) {
      eventStreamOwners.delete(graphId);
    }
  };
  eventStreamOwners.set(graphId, stop);
  el.addEventListener("resuma:disconnect", stop, { once: true });
  registerFlowCleanup(stop);

  void (async () => {
    const replayPath = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}/replay`, token);
    const headers = graphFetchHeaders(token);

    let terminal = false;
    try {
      const res = await fetch(replayPath, { headers, credentials: "same-origin" });
      if (!isActive() || aborted) return;
      if (res.ok) {
        const events = (await res.json()) as WorkerEvent[];
        list.innerHTML = "";
        seen.clear();
        for (const ev of events) appendEvent(ev, false);
        scrollStreamToEnd(viewport, false);
        terminal = events.some((ev) => ev.type === "graph_done");
        if (terminal) {
          el.dataset.rStreamTerminal = "1";
          markGraphTerminal(graphId);
        }
      }
    } catch {
      /* ignore */
    }

    if (!isActive() || aborted || terminal || completedGraphIds.has(graphId)) return;

    try {
      const graphPath = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}`, token);
      const snapRes = await fetch(graphPath, { headers, credentials: "same-origin" });
      if (!isActive() || aborted) return;
      if (snapRes.ok) {
        const snap = (await snapRes.json()) as { status?: string };
        if (graphTerminalStatus(snap.status)) {
          el.dataset.rStreamTerminal = "1";
          markGraphTerminal(graphId);
          return;
        }
      }
    } catch {
      /* ignore */
    }

    if (!isActive() || aborted) return;

    closeSse = subscribeGraphEvents(graphId, token, (ev) => {
      if (!isActive() || aborted) return;
      appendEvent(ev);
    });
  })();
}

async function syncWorkerControls(
  el: HTMLElement,
  graphId: string,
  token: string,
  hint?: string,
): Promise<void> {
  const statusEl = el.querySelector<HTMLElement>("[data-r-worker-status]");
  const pauseBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-pause]");
  const resumeBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-resume]");
  const cancelBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-cancel]");
  const replayBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-replay]");

  const path = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}`, token);
  try {
    const res = await fetch(path, { headers: graphFetchHeaders(token), credentials: "same-origin" });
    if (!res.ok) {
      if (statusEl) statusEl.textContent = hint ?? `Could not load graph (HTTP ${res.status}).`;
      return;
    }
    const snap = (await res.json()) as { status?: string };
    const st = snap.status ?? "unknown";
    const terminal = st === "done" || st === "failed";
    const paused = st === "paused";

    if (pauseBtn) pauseBtn.disabled = terminal || paused;
    if (resumeBtn) resumeBtn.disabled = !paused;
    if (cancelBtn) cancelBtn.disabled = terminal;
    if (replayBtn) replayBtn.disabled = false;

    if (statusEl) {
      if (hint) statusEl.textContent = hint;
      else if (terminal)
        statusEl.textContent =
          st === "done"
            ? "Done — controls are off. Click Run worker above to try Pause/Cancel again."
            : `Graph ${st} — click Run worker above to start a new run.`;
      else if (paused)
        statusEl.textContent = "Paused — click Resume to continue, or Cancel to abort.";
      else
        statusEl.textContent =
          "Running — click Pause or Cancel now (you have ~25s before this graph finishes).";
    }
  } catch {
    if (statusEl && hint) statusEl.textContent = hint;
  }
}

function mountWorkerPanel(el: HTMLElement): void {
  const graphId = graphIdFrom(el);
  const token = graphTokenFrom(el);
  if (!graphId) return;

  if (el.dataset.rFlowMounted === "1") {
    el.dispatchEvent(new Event("resuma:disconnect"));
  }
  el.dataset.rFlowMounted = "1";

  let aborted = false;
  let closeSse: (() => void) | null = null;
  const stop = () => {
    aborted = true;
    closeSse?.();
    closeSse = null;
    delete el.dataset.rFlowMounted;
  };
  el.addEventListener("resuma:disconnect", stop, { once: true });
  registerFlowCleanup(stop);

  void syncWorkerControls(el, graphId, token);

  const scheduleControlSync = debounce(() => {
    if (!aborted) void syncWorkerControls(el, graphId, token);
  }, 300);

  void (async () => {
    try {
      const path = withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}`, token);
      const res = await fetch(path, { headers: graphFetchHeaders(token), credentials: "same-origin" });
      if (aborted) return;
      if (res.ok) {
        const snap = (await res.json()) as { status?: string };
        if (graphTerminalStatus(snap.status)) {
          markGraphTerminal(graphId);
          await syncWorkerControls(el, graphId, token);
          return;
        }
      }
    } catch {
      /* ignore */
    }
    if (aborted) return;

    closeSse = subscribeGraphEvents(graphId, token, (ev) => {
      if (aborted) return;
      if (ev.type === "graph_done") {
        markGraphTerminal(graphId);
        void syncWorkerControls(el, graphId, token);
        return;
      }
      if (
        ev.type === "node_done" ||
        ev.type === "node_failed" ||
        ev.type === "progress"
      ) {
        scheduleControlSync();
      }
    });
  })();

  const postOpts = (): RequestInit => ({
    method: "POST",
    credentials: "same-origin",
    headers: graphMutationHeaders(token),
  });
  const graphRoot = el.closest("[data-r-flow-execution]");
  const postControl = async (path: "pause" | "resume" | "cancel") => {
    const statusEl = el.querySelector<HTMLElement>("[data-r-worker-status]");
    const pauseBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-pause]");
    const resumeBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-resume]");
    const cancelBtn = el.querySelector<HTMLButtonElement>("[data-r-worker-cancel]");
    if (completedGraphIds.has(graphId)) {
      await syncWorkerControls(
        el,
        graphId,
        token,
        "Graph finished — run the worker again to try Pause/Cancel.",
      );
      return;
    }
    if (pauseBtn) pauseBtn.disabled = true;
    if (resumeBtn) resumeBtn.disabled = true;
    if (cancelBtn) cancelBtn.disabled = true;
    if (statusEl) statusEl.textContent = `${path}…`;
    const res = await fetch(graphControlPath(graphId, path), postOpts());
    if (aborted) return;
    if (!res.ok) {
      const msg = await controlErrorMessage(res);
      await syncWorkerControls(el, graphId, token, msg);
      return;
    }
    const graphEl = graphRoot?.querySelector<HTMLElement>("[data-r-flow-graph]");
    if (graphEl) void refreshGraph(graphEl, graphId, token);
    const labels = { pause: "Paused", resume: "Resumed", cancel: "Cancelled" } as const;
    await syncWorkerControls(el, graphId, token, `${labels[path]} — refreshing graph…`);
  };
  el.querySelector("[data-r-worker-pause]")?.addEventListener("click", () => {
    void postControl("pause");
  });
  el.querySelector("[data-r-worker-resume]")?.addEventListener("click", () => {
    void postControl("resume");
  });
  el.querySelector("[data-r-worker-cancel]")?.addEventListener("click", () => {
    void postControl("cancel");
  });
  el.querySelector("[data-r-worker-replay]")?.addEventListener("click", async () => {
    const res = await fetch(
      withGraphToken(`/_resuma/graph/${encodeURIComponent(graphId)}/replay`, token),
      { headers: graphFetchHeaders(token), credentials: "same-origin" },
    );
    if (!res.ok) return;
    const events = (await res.json()) as WorkerEvent[];
    const stream = el.closest("[data-r-flow-execution]")?.querySelector("[data-r-event-stream]");
    const listEl = stream?.querySelector("ul");
    const viewportEl = stream ? eventStreamViewport(stream as HTMLElement) : null;
    if (!listEl) return;
    const seen = seenForList(listEl as HTMLElement);
    seen.clear();
    listEl.innerHTML = "";
    delete (stream as HTMLElement).dataset.rStreamTerminal;
    for (const ev of events) {
      const key = eventKey(ev);
      if (seen.has(key)) continue;
      seen.add(key);
      const li = document.createElement("li");
      li.textContent = formatEvent(ev);
      listEl.appendChild(li);
    }
    if (viewportEl) scrollStreamToEnd(viewportEl, false);
    await syncWorkerControls(el, graphId, token, "Replay loaded.");
  });
}

/** Disconnect Flow widgets inside `scope` (timers, SSE) without touching siblings. */
export function disconnectFlowWidgets(scope: ParentNode = document): void {
  const graphIds = new Set<string>();
  scope
    .querySelectorAll<HTMLElement>(
      "[data-r-flow-dashboard], [data-r-flow-graph], [data-r-event-stream], [data-r-worker-panel]",
    )
    .forEach((el) => {
      const id = graphIdFrom(el);
      if (id) graphIds.add(id);
      el.dispatchEvent(new Event("resuma:disconnect"));
    });
  for (const id of graphIds) closeGraphSseHub(id);
}

/** Mount all Flow widgets (dashboard, graph, events, controls). */
export function initFlowWidgets(scope: ParentNode = document, opts?: { flush?: boolean }): void {
  if (opts?.flush !== false) {
    // Full page mount — tear down every widget from the prior navigation.
    flushFlowCleanups();
  } else {
    // Scoped mount (e.g. dynamic exec panel) — only disconnect widgets in this subtree.
    disconnectFlowWidgets(scope);
  }
  scope.querySelectorAll<HTMLElement>("[data-r-flow-dashboard]").forEach(mountFlowDashboard);
  const mounted = new Set<string>();
  const mountGraphWidget = (
    kind: string,
    el: HTMLElement,
    fn: (node: HTMLElement) => void,
  ) => {
    const graphId = graphIdFrom(el);
    if (!graphId) {
      fn(el);
      return;
    }
    const key = `${kind}:${graphId}`;
    if (mounted.has(key)) return;
    mounted.add(key);
    fn(el);
  };
  scope.querySelectorAll<HTMLElement>("[data-r-flow-graph]").forEach((el) => {
    mountGraphWidget("graph", el, mountFlowGraph);
  });
  scope.querySelectorAll<HTMLElement>("[data-r-event-stream]").forEach((el) => {
    mountGraphWidget("stream", el, mountEventStream);
  });
  scope.querySelectorAll<HTMLElement>("[data-r-worker-panel]").forEach((el) => {
    mountGraphWidget("panel", el, mountWorkerPanel);
  });
}
