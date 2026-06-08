#!/usr/bin/env python3
"""HTTP smoke test for resuma-audit — all routes + key API endpoints."""

import http.cookiejar
import json
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:3000"

REGISTRY = open(
    "/home/golfredo/Documentos/apps/resuma/examples/resuma-audit/src/pages/_registry.rs"
).read()
modules = re.findall(r'"([^"]+)" => Some', REGISTRY)
paths = sorted(
    set(
        "/"
        if m == "index"
        else "/audit/" + m.replace("audit::", "").replace("::", "/").replace("_id_", "42")
        for m in modules
    )
)

failures = []
ok = 0

for p in paths:
    url = BASE + p
    try:
        r = urllib.request.urlopen(url, timeout=10)
        body = r.read().decode("utf-8", errors="replace")
        if r.status != 200:
            failures.append((p, f"status {r.status}"))
        elif "Error 404" in body and "Page not found" in body:
            failures.append((p, "404 page content"))
        else:
            ok += 1
    except Exception as e:
        failures.append((p, str(e)))

# Dynamic user 404 should show error page (still 200 from error_page)
try:
    r = urllib.request.urlopen(BASE + "/audit/flow/users/404", timeout=10)
    body = r.read().decode("utf-8", errors="replace")
    if "not found" not in body.lower() and "404" not in body.lower():
        failures.append(("/audit/flow/users/404", "missing error message"))
    else:
        ok += 1
except Exception as e:
    failures.append(("/audit/flow/users/404", str(e)))

# Static asset
try:
    r = urllib.request.urlopen(BASE + "/audit-badge.svg", timeout=5)
    if r.status == 200:
        ok += 1
    else:
        failures.append(("/audit-badge.svg", f"status {r.status}"))
except Exception as e:
    failures.append(("/audit-badge.svg", str(e)))

# Server action (CSRF double-submit: cookie + header)
try:
    jar = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
    page = opener.open(BASE + "/audit/components/server", timeout=10)
    html = page.read().decode()
    csrf_m = re.search(r'id="resuma-state"[^>]*>(\{.*?\})</script>', html, re.S)
    token = ""
    if csrf_m:
        payload = json.loads(csrf_m.group(1))
        token = payload.get("csrf_token", "") or ""
    req = urllib.request.Request(
        BASE + "/_resuma/action/audit_echo",
        data=b'{"args":["smoke"]}',
        headers={
            "Content-Type": "application/json",
            "X-Resuma-CSRF": token,
        },
        method="POST",
    )
    resp = opener.open(req, timeout=10)
    data = resp.read().decode()
    if "smoke" in data:
        ok += 1
    else:
        failures.append(("POST audit_echo", data[:200]))
except Exception as e:
    failures.append(("POST audit_echo", str(e)))

print(f"BASE={BASE}")
print(f"Routes OK: {ok}")
print(f"Failures: {len(failures)}")
for p, err in failures:
    print(f"  FAIL {p}: {err}")

sys.exit(1 if failures else 0)
