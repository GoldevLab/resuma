use crate::site::code_block;
use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <>
            <h1>"E2E testing"</h1>
            <p class="lead">"End-to-end tests for Resuma apps — comparable to Qwik's Playwright integration."</p>

            <h2>"Approach"</h2>
            <ul>
                <li>"Spin up the app with " <code>"cargo run"</code> " or " <code>"resuma dev"</code> " in CI"</li>
                <li>"Use " <a href="https://playwright.dev" target="_blank">"Playwright"</a> " (Node) or " <code>"fantoccini"</code> " (Rust WebDriver) against real HTTP"</li>
                <li>"Assert SSR HTML, form POSTs to " <code>"/_resuma/submit/:name"</code>", and resumability markers"</li>
            </ul>

            <h2>"Playwright example"</h2>
            {code_block(r#"// tests/e2e/home.spec.ts
import { test, expect } from '@playwright/test';

test('landing renders h1', async ({ page }) => {
  await page.goto('http://127.0.0.1:3000/');
  await expect(page.locator('h1')).toContainText('Resuma');
});

test('form submit without JS', async ({ page }) => {
  await page.goto('http://127.0.0.1:3000/contact');
  await page.fill('input[name=email]', 'a@b.co');
  await page.click('button[type=submit]');
  await expect(page).toHaveURL(/thanks/);
});"#)}

            <h2>"CI"</h2>
            {code_block(r#"# GitHub Actions
- run: cargo build -p my-app
- run: cargo run -p my-app &
- run: npx playwright test"#)}
        </>
    }
}
