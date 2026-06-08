#!/usr/bin/env python3
"""Interactive audit checklist — run in browser console after loading each page.

Usage: open audit app, paste sections, or use with browser automation.
"""

INTERACTIVE = [
    ("/audit/intro/getting_started", "click .btn", "Resumed! Interactivity works."),
    ("/audit/components/signals", "document.querySelectorAll('.btn')[1].click(); document.querySelectorAll('.btn')[1].click(); document.querySelector('resuma-dyn')?.textContent", "2"),
    ("/audit/components/control_flow", "document.querySelector('.btn').click(); JSON.stringify({s1:window.__resuma?.signals?.get('s1')?.value, logout:document.querySelector('.btn')?.textContent?.includes('Logout')})", "true"),
    ("/audit/components/handlers", "document.querySelector('.btn').click(); document.querySelector('resuma-dyn')?.textContent", "1"),
    ("/audit/components/server", "await (async()=>{document.querySelector('.btn').click(); await new Promise(r=>setTimeout(r,600)); return document.querySelector('resuma-dyn')?.textContent;})()", "Echo:"),
    ("/audit/components/js", "const i=document.querySelector('input'); i.value='hi'; i.dispatchEvent(new Event('input',{bubbles:true})); document.querySelector('resuma-dyn')?.textContent", "hi"),
    ("/audit/components/error_boundary", "document.querySelector('.btn').click(); document.querySelector('.pill')?.textContent", "broke"),
    ("/audit/components/islands", "document.querySelector('resuma-island .btn')?.click(); document.querySelector('resuma-island resuma-dyn')?.textContent", "1"),
    ("/audit/components/store", "document.querySelector('.btn').click(); document.querySelector('resuma-dyn')?.textContent", "Count: 1"),
    ("/audit/cookbook/debouncer", "await (async()=>{const i=document.querySelector('input'); i.value='abc'; i.dispatchEvent(new Event('input',{bubbles:true})); await new Promise(r=>setTimeout(r,400)); return document.querySelector('resuma-dyn')?.textContent;})()", "Debounced"),
    ("/audit/cookbook/portals", "document.querySelector('.btn').click(); setTimeout(()=>document.querySelector('#modals .modal')?.textContent,0)", "Portal"),
    ("/audit/security/todo", "await (async()=>{await new Promise(r=>setTimeout(r,500)); const i=document.querySelector('input'); i.value='Audit task'; i.dispatchEvent(new Event('input',{bubbles:true})); document.querySelector('.btn').click(); await new Promise(r=>setTimeout(r,800)); return document.querySelector('[data-audit-todos]')?.textContent;})()", "Audit task"),
    ("/audit/flow/streaming", "await (async()=>{await new Promise(r=>setTimeout(r,1200)); return document.body.innerText.includes('800ms');})()", "true"),
]

if __name__ == "__main__":
    for path, js, expect in INTERACTIVE:
        print(f"{path}\n  expect contains: {expect!r}\n  js: {js}\n")
