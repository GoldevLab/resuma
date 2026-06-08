#!/usr/bin/env python3
"""Browser interactive audit for resuma-audit."""

import json
import sys
from playwright.sync_api import sync_playwright

BASE = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:3000"

TESTS = [
    (
        "/audit/intro/getting_started",
        """
        async () => {
          await page.click('.demo-box .btn, .btn');
          await page.waitForTimeout(200);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "Resumed",
    ),
    (
        "/audit/components/signals",
        """
        async () => {
          const plus = page.locator('.demo-box .btn').nth(1);
          await plus.click(); await plus.click();
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "2",
    ),
    (
        "/audit/components/control_flow",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(150);
          return await page.evaluate(() => window.__resuma?.signals?.get('s1')?.value);
        }
        """,
        "true",
    ),
    (
        "/audit/components/handlers",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(100);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "1",
    ),
    (
        "/audit/components/server",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(800);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "Echo",
    ),
    (
        "/audit/components/js",
        """
        async () => {
          const input = page.locator('.demo-box input');
          await input.fill('hello');
          await page.waitForTimeout(150);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "hello",
    ),
    (
        "/audit/components/error_boundary",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(150);
          return await page.locator('.pill').textContent();
        }
        """,
        "broke",
    ),
    (
        "/audit/components/islands",
        """
        async () => {
          await page.click('resuma-island .btn');
          await page.waitForTimeout(150);
          return await page.locator('resuma-island resuma-dyn').textContent();
        }
        """,
        "1",
    ),
    (
        "/audit/components/store",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(150);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "Count: 1",
    ),
    (
        "/audit/components/effects",
        """
        async () => {
          await page.locator('.demo-box input').first.fill('Grace');
          await page.locator('.demo-box input').nth(1).fill('Hopper');
          await page.waitForTimeout(150);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "Grace Hopper",
    ),
    (
        "/audit/cookbook/debouncer",
        """
        async () => {
          await page.locator('.demo-box input').fill('test');
          await page.waitForTimeout(450);
          return await page.locator('resuma-dyn').first.textContent();
        }
        """,
        "Debounced",
    ),
    (
        "/audit/cookbook/portals",
        """
        async () => {
          await page.click('.demo-box .btn');
          await page.waitForTimeout(300);
          await page.click('#modals button:has-text("Close")');
          await page.waitForTimeout(300);
          return document.querySelector('#modals')?.childElementCount === 0 ? 'closed' : 'open';
        }
        """,
        "closed",
    ),
    (
        "/audit/cookbook/theme",
        """
        async () => {
          await page.click('[data-audit-theme-panel] .btn-themed');
          await page.waitForTimeout(200);
          return await page.locator('[data-audit-theme-panel] p').textContent();
        }
        """,
        "light",
    ),
    (
        "/audit/components/form",
        """
        async () => {
          await page.locator('input[name="name"]').fill('QA');
          await page.locator('button[type="submit"]').click();
          await page.waitForTimeout(1200);
          return await page.locator('[data-audit-greet-result]').textContent();
        }
        """,
        "Hello, QA",
    ),
    (
        "/audit/flow/actions",
        """
        async () => {
          await page.locator('input[name="name"]').fill('Flow');
          await page.locator('button[type="submit"]').click();
          await page.waitForTimeout(1200);
          return await page.locator('[data-audit-greet-result]').textContent();
        }
        """,
        "Hello, Flow",
    ),
    (
        "/audit/cookbook/prg",
        """
        async () => {
          await page.locator('input[name="item"]').fill('widget');
          await page.locator('button[type="submit"]').click();
          await page.waitForTimeout(1200);
          return document.querySelector('.pill')?.textContent || page.url;
        }
        """,
        "widget",
    ),
    (
        "/audit/security/middleware",
        """
        async () => {
          await page.click('[data-audit-user-btn][data-user="alice"]');
          await page.waitForTimeout(1500);
          const cookies = await page.context().cookies();
          return cookies.find(c => c.name === 'resuma_demo_user')?.value || '';
        }
        """,
        "alice",
    ),
    (
        "/audit/security/todo",
        """
        async () => {
          await page.waitForTimeout(700);
          await page.locator('input[placeholder="New task"]').fill('Audit task');
          await page.click('.demo-box .btn');
          await page.waitForTimeout(900);
          const list = await page.locator('[data-audit-todos]').textContent();
          const cb = page.locator('[data-audit-todos] input[type=checkbox]').first;
          const was = await cb.isChecked();
          await cb.click();
          await page.waitForTimeout(900);
          const now = await cb.isChecked();
          return list + '|toggle:' + (now !== was);
        }
        """,
        "toggle:true",
    ),
    (
        "/audit/components/accessibility",
        """
        async () => {
          await page.locator('[data-audit-a11y-btn]').click();
          await page.locator('[data-audit-a11y-btn]').click();
          await page.waitForTimeout(150);
          return await page.locator('[data-audit-a11y-status]').textContent();
        }
        """,
        "Count: 2",
    ),
    (
        "/audit/flow/platform?platform=mobile",
        """
        async () => {
          return await page.locator('[data-audit-platform-panel]').getAttribute('class');
        }
        """,
        "platform-mobile",
    ),
    (
        "/audit/cookbook/virtual_list",
        """
        async () => {
          await page.waitForTimeout(400);
          const vp = page.locator('[data-audit-vlist-viewport]');
          await vp.evaluate(el => { el.scrollTop = 320; });
          await page.waitForTimeout(200);
          return await page.locator('[data-audit-vlist-meta]').textContent();
        }
        """,
        "Showing rows",
    ),
    (
        "/audit/reference/matrix",
        """
        async () => {
          return await page.locator('.matrix-table').textContent();
        }
        """,
        "Virtual list",
    ),
    (
        "/audit/integrations",
        """
        async () => {
          return await page.locator('.matrix-table').textContent();
        }
        """,
        "SQLx",
    ),
    (
        "/audit/flow/streaming",
        """
        async () => {
          await page.waitForTimeout(1400);
          return await page.textContent('body');
        }
        """,
        "800ms",
    ),
    (
        "/audit/flow/query_params?q=rust",
        """
        async () => {
          return await page.textContent('body');
        }
        """,
        "Result A",
    ),
    (
        "/audit/flow/users/42",
        """
        async () => {
          return await page.textContent('body');
        }
        """,
        "User #42",
    ),
    (
        "/audit/reference/registry",
        """
        async () => {
          await page.locator('[data-reg-search]').fill('virtual');
          await page.waitForTimeout(200);
          return await page.locator('[data-reg-item]:visible').count();
        }
        """,
        "1",
    ),
    (
        "/audit/cookbook/animations",
        """
        async () => {
          await page.click('[data-audit-anim-toggle]');
          await page.waitForTimeout(200);
          return await page.locator('[data-audit-anim-status]').textContent();
        }
        """,
        "running",
    ),
    (
        "/audit/cookbook/drag_drop",
        """
        async () => {
          return await page.locator('[data-audit-dnd-status]').textContent();
        }
        """,
        "Order:",
    ),
    (
        "/audit/cookbook/image_list",
        """
        async () => {
          await page.waitForTimeout(500);
          const vp = page.locator('[data-audit-img-viewport]');
          await vp.evaluate(el => { el.scrollTop = 200; });
          await page.waitForTimeout(200);
          return await page.locator('[data-audit-img-meta]').textContent();
        }
        """,
        "Showing",
    ),
    (
        "/audit/components/clipboard",
        """
        async () => {
          await page.click('[data-audit-clip-copy]');
          await page.waitForTimeout(200);
          return await page.locator('[data-audit-clip-status]').textContent();
        }
        """,
        "Copied",
    ),
    (
        "/audit/components/picker",
        """
        async () => {
          await page.selectOption('[data-audit-picker]', 'remix');
          return await page.locator('[data-audit-picker-out]').textContent();
        }
        """,
        "remix",
    ),
    (
        "/audit/integrations/network",
        """
        async () => {
          return await page.locator('[data-audit-network]').textContent();
        }
        """,
        "Online",
    ),
    (
        "/audit/integrations/storage",
        """
        async () => {
          await page.locator('[data-audit-store-input]').fill('persisted');
          await page.click('[data-audit-store-save]');
          await page.waitForTimeout(150);
          return await page.locator('[data-audit-store-out]').textContent();
        }
        """,
        "Stored",
    ),
    (
        "/audit/flow/gestures",
        """
        async () => {
          await page.locator('[data-audit-gesture-pad]').click();
          await page.waitForTimeout(150);
          return await page.locator('[data-audit-gesture-out]').textContent();
        }
        """,
        "pointer",
    ),
    (
        "/audit/flow/user_presence",
        """
        async () => {
          return await page.locator('[data-audit-presence-vis]').textContent();
        }
        """,
        "visible",
    ),
]


def wait_resuma(page):
    page.wait_for_function(
        "() => window.__resuma?.ready === true || document.querySelector('#resuma-state')",
        timeout=10000,
    )
    page.wait_for_timeout(300)


def main():
    results = []
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        page.set_default_timeout(15000)

        for path, _js, expect in TESTS:
            url = BASE + path
            try:
                if path == "/audit/components/clipboard":
                    context = page.context
                    context.grant_permissions(["clipboard-read", "clipboard-write"], origin=BASE)
                page.goto(url, wait_until="domcontentloaded")
                wait_resuma(page)
                # Run test logic inline per path
                out = run_test(page, path)
                ok = expect.lower() in str(out).lower()
                results.append({"path": path, "ok": ok, "out": str(out)[:150], "expect": expect})
            except Exception as e:
                results.append({"path": path, "ok": False, "error": str(e)})

        browser.close()

    passed = sum(1 for r in results if r.get("ok"))
    print(f"BASE={BASE}")
    print(f"Interactive: {passed}/{len(results)} passed")
    for r in results:
        status = "PASS" if r.get("ok") else "FAIL"
        print(f"  [{status}] {r['path']}")
        if not r.get("ok"):
            print(f"         {r.get('error') or r.get('out')} (expected {r.get('expect')!r})")
    sys.exit(0 if passed == len(results) else 1)


def run_test(page, path):
    if path == "/audit/intro/getting_started":
        page.click(".demo-box .btn")
        page.wait_for_timeout(200)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/components/signals":
        plus = page.locator(".demo-box button", has_text="+")
        plus.click()
        page.wait_for_timeout(100)
        plus.click()
        page.wait_for_timeout(150)
        return page.locator(".demo-box resuma-dyn").first.text_content()
    if path == "/audit/components/control_flow":
        page.click(".demo-box .btn")
        page.wait_for_timeout(150)
        val = page.evaluate("() => window.__resuma?.signals?.get('s1')?.value")
        return str(val).lower()
    if path == "/audit/components/handlers":
        page.click(".demo-box .btn")
        page.wait_for_timeout(100)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/components/server":
        page.click(".demo-box .btn")
        page.wait_for_timeout(800)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/components/js":
        page.locator(".demo-box input").fill("hello")
        page.wait_for_timeout(150)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/components/error_boundary":
        page.click(".demo-box .btn")
        page.wait_for_timeout(150)
        return page.locator(".pill").text_content()
    if path == "/audit/components/islands":
        page.click("resuma-island .btn")
        page.wait_for_timeout(150)
        return page.locator("resuma-island resuma-dyn").text_content()
    if path == "/audit/components/store":
        page.click(".demo-box .btn")
        page.wait_for_timeout(150)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/components/effects":
        first = page.locator('.demo-box input[placeholder="First"]')
        first.click()
        page.wait_for_timeout(150)
        first.fill("")
        first.press_sequentially("Grace", delay=30)
        last = page.locator('.demo-box input[placeholder="Last"]')
        last.click()
        last.fill("")
        last.press_sequentially("Hopper", delay=30)
        page.wait_for_timeout(300)
        return page.locator(".demo-box resuma-dyn").first.text_content()
    if path == "/audit/cookbook/debouncer":
        page.locator(".demo-box input").fill("test")
        page.wait_for_timeout(450)
        return page.locator("resuma-dyn").first.text_content()
    if path == "/audit/cookbook/portals":
        page.click(".demo-box .btn")
        page.wait_for_timeout(300)
        page.click('#modals button:has-text("Close")')
        page.wait_for_timeout(300)
        empty = page.evaluate('() => document.querySelector("#modals")?.childElementCount === 0')
        return "closed" if empty else "open"
    if path == "/audit/cookbook/theme":
        page.click("[data-audit-theme-toggle]")
        page.wait_for_timeout(250)
        return page.locator("[data-audit-theme-mode]").text_content()
    if path == "/audit/components/form":
        page.locator('input[name="name"]').fill("QA")
        page.locator('button[type="submit"]').click()
        page.wait_for_timeout(1200)
        return page.locator("[data-audit-greet-result]").text_content()
    if path == "/audit/flow/actions":
        page.locator('input[name="name"]').fill("Flow")
        page.locator('button[type="submit"]').click()
        page.wait_for_timeout(1200)
        return page.locator("[data-audit-greet-result]").text_content()
    if path == "/audit/cookbook/prg":
        page.locator('input[name="item"]').fill("widget")
        page.locator('button[type="submit"]').click()
        page.wait_for_timeout(1200)
        pill = page.locator(".pill").text_content()
        return pill or page.url
    if path == "/audit/security/middleware":
        page.locator('[data-audit-user-btn][data-user="alice"]').click()
        page.wait_for_timeout(1500)
        cookies = page.context.cookies()
        for c in cookies:
            if c["name"] == "resuma_demo_user":
                return c["value"]
        return ""
    if path == "/audit/security/todo":
        page.wait_for_function(
            "() => (document.querySelector('[data-audit-todos]')?.textContent || '').length > 0",
            timeout=8000,
        )
        inp = page.locator('input[placeholder="New task"]')
        task = f"Audit task {int(page.evaluate('Date.now()'))}"
        inp.fill(task)
        page.locator("[data-audit-add]").click()
        page.wait_for_function(
            f"() => document.querySelector('[data-audit-todos]')?.textContent?.includes('{task}')",
            timeout=8000,
        )
        cb = page.locator(f'[data-audit-todos] li:has-text("{task}") input[type=checkbox]')
        was = cb.is_checked()
        cb.click()
        page.wait_for_timeout(900)
        now = cb.is_checked()
        return f"{task}|toggle:{now != was}"
    if path == "/audit/components/accessibility":
        page.locator("[data-audit-a11y-btn]").click()
        page.locator("[data-audit-a11y-btn]").click()
        page.wait_for_timeout(150)
        return page.locator("[data-audit-a11y-status]").text_content()
    if path == "/audit/flow/platform?platform=mobile":
        return page.locator("[data-audit-platform-panel]").get_attribute("class")
    if path == "/audit/cookbook/virtual_list":
        page.wait_for_timeout(400)
        vp = page.locator("[data-audit-vlist-viewport]")
        vp.evaluate("el => { el.scrollTop = 320; }")
        page.wait_for_timeout(200)
        return page.locator("[data-audit-vlist-meta]").text_content()
    if path == "/audit/reference/matrix":
        return page.locator(".matrix-table").text_content()
    if path == "/audit/integrations":
        return page.locator(".matrix-table").text_content()
    if path == "/audit/flow/streaming":
        page.wait_for_function(
            "() => document.body.textContent.includes('800ms')",
            timeout=5000,
        )
        return page.text_content("body")
    if path == "/audit/flow/query_params?q=rust":
        return page.text_content("body")
    if path == "/audit/flow/users/42":
        return page.text_content("body")
    if path == "/audit/reference/registry":
        page.wait_for_timeout(600)
        page.locator("[data-reg-search]").fill("virtual")
        page.wait_for_timeout(300)
        visible = page.evaluate(
            """() => [...document.querySelectorAll('[data-reg-item]')].filter(el => !el.hidden).length"""
        )
        return str(visible)
    if path == "/audit/cookbook/animations":
        page.click("[data-audit-anim-toggle]")
        page.wait_for_timeout(200)
        return page.locator("[data-audit-anim-status]").text_content()
    if path == "/audit/cookbook/drag_drop":
        return page.locator("[data-audit-dnd-status]").text_content()
    if path == "/audit/cookbook/image_list":
        page.wait_for_timeout(500)
        vp = page.locator("[data-audit-img-viewport]")
        vp.evaluate("el => { el.scrollTop = 200; }")
        page.wait_for_timeout(200)
        return page.locator("[data-audit-img-meta]").text_content()
    if path == "/audit/components/clipboard":
        page.click("[data-audit-clip-copy]")
        page.wait_for_timeout(200)
        return page.locator("[data-audit-clip-status]").text_content()
    if path == "/audit/components/picker":
        page.select_option("[data-audit-picker]", "remix")
        return page.locator("[data-audit-picker-out]").text_content()
    if path == "/audit/integrations/network":
        return page.locator("[data-audit-network]").text_content()
    if path == "/audit/integrations/storage":
        page.locator("[data-audit-store-input]").fill("persisted")
        page.click("[data-audit-store-save]")
        page.wait_for_timeout(150)
        return page.locator("[data-audit-store-out]").text_content()
    if path == "/audit/flow/gestures":
        page.locator("[data-audit-gesture-pad]").click()
        page.wait_for_timeout(150)
        return page.locator("[data-audit-gesture-out]").text_content()
    if path == "/audit/flow/user_presence":
        return page.locator("[data-audit-presence-vis]").text_content()
    raise ValueError(f"unknown test {path}")


if __name__ == "__main__":
    main()
