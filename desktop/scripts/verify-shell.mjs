import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const [html, app, styles, tauriConfig, rustMain] = await Promise.all([
  readFile(new URL("../index.html", import.meta.url), "utf8"),
  readFile(new URL("../src/App.tsx", import.meta.url), "utf8"),
  readFile(new URL("../src/styles.css", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8"),
]);

assert.match(html, /<html lang="en" dir="ltr">/, "The HTML document must declare English and LTR.");
assert.match(app, /document\.documentElement\.dir = "ltr"/, "The application must enforce LTR at runtime.");
assert.match(app, /document\.documentElement\.lang = "en"/, "The application must enforce English at runtime.");

for (const value of [
  "192.0.2.10",
  "2001:db8::1",
  "02:00:5e:10:00:00",
  "Ethernet 2 / ifIndex 12",
  "443",
  "2026-10-31T10:20:30Z INFO route 192.0.2.10 -> 192.0.2.1 via Ethernet 2",
  "C:\\Program Files\\NetShift\\logs\\netshift.log",
]) {
  assert.ok(app.includes(value), `Missing representative LTR value: ${value}`);
}

assert.match(styles, /max-width: 1279px/, "The compact sidebar breakpoint is missing.");
assert.match(styles, /max-width: 1023px/, "The small desktop breakpoint is missing.");
assert.match(styles, /min-width: 960px/, "The minimum supported desktop width is missing.");
assert.match(styles, /prefers-reduced-motion/, "Reduced-motion preferences must be respected.");
assert.match(styles, /:focus-visible/, "Visible keyboard focus styles must be present.");
assert.match(app, /navigator\\.clipboard\\.writeText/, "Copy controls must use the Clipboard API.");
assert.match(app, /aria-live="polite"/, "Copy feedback must be announced to assistive technology.");
assert.match(app, /aria-modal="true"/, "Preview notices must use a modal dialog.");

const config = JSON.parse(tauriConfig);
const windowConfig = config.app.windows[0];
assert.equal(windowConfig.minWidth, 960, "Tauri must enforce the 960px minimum width.");
assert.equal(windowConfig.minHeight, 640, "Tauri must enforce the 640px minimum height.");
assert.equal(config.build.frontendDist, "../dist", "Tauri must load the built frontend.");

assert.doesNotMatch(rustMain, /invoke_handler/, "The prototype must not register backend command handlers.");
assert.doesNotMatch(app, /\b(invoke|fetch|XMLHttpRequest)\s*\(/, "The frontend must not call backend commands or network endpoints.");

console.log("NetShift shell checks passed: English LTR, representative values, responsive guards, and no network commands.");
