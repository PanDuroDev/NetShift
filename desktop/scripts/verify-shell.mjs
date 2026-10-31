import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const [html, app, styles, tauriConfig, rustMain, buildScript] = await Promise.all([
  readFile(new URL("../index.html", import.meta.url), "utf8"),
  readFile(new URL("../src/App.tsx", import.meta.url), "utf8"),
  readFile(new URL("../src/styles.css", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8"),
  readFile(new URL("../src-tauri/build.rs", import.meta.url), "utf8"),
]);

assert.match(html, /<html lang="en" dir="ltr">/, "The HTML document must declare English and LTR.");
assert.match(app, /document\.documentElement\.dir = "ltr"/, "The application must enforce LTR at runtime.");
assert.match(app, /document\.documentElement\.lang = "en"/, "The application must enforce English at runtime.");
assert.doesNotMatch(`${html}\n${app}`, /[\u0600-\u06FF]/, "The UI source must not contain Arabic copy in this English-only phase.");

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

assert.match(styles, /--md-sys-color-surface: #111111/, "The dark surface must use the requested #111111 base.");
assert.match(styles, /--md-sys-color-on-surface: #eaeaea/, "The dark text must use the requested #eaeaea base.");
assert.match(styles, /--md-sys-color-primary: #eaeaea/, "The dark primary role must use the contrasting inverse tone.");
assert.match(styles, /--md-sys-color-surface: #eaeaea/, "The light surface must invert to #eaeaea.");
assert.match(styles, /--md-sys-color-on-surface: #111111/, "The light text must invert to #111111.");
assert.match(styles, /--md-sys-color-primary: #111111/, "The light primary role must invert to #111111.");
assert.doesNotMatch(styles, /#8de5ee|#54cbd8|#087e8b|#101518|#e8f1f3/, "The previous cyan palette must not remain.");
assert.match(styles, /--md-sys-state-hover-opacity: 0\.08/, "The MD3 hover state layer must be 8%.");
assert.match(styles, /--md-sys-state-focus-opacity: 0\.10/, "The MD3 focus state layer must be 10%.");
assert.match(styles, /--md-sys-shape-corner-extra-large: 28px/, "The MD3 dialog corner token is missing.");
assert.match(styles, /--md-sys-motion-duration-short-2: 100ms/, "The MD3 motion duration token is missing.");
assert.match(styles, /\.button--primary,[\s\S]*?min-height: 48px/, "The prominent primary button must be 48px high.");
assert.match(styles, /\.action-dialog \{[\s\S]*?border-radius: var\(--md-sys-shape-corner-extra-large\)/, "The dialog must use the 28px MD3 extra-large corner.");
assert.match(styles, /\.provider-capabilities span \{[\s\S]*?min-height: 32px/, "Capability chips must use the 32px MD3 height.");
assert.match(app, /data-tooltip=/, "Icon-only actions must expose a tooltip.");
assert.doesNotMatch(styles, /nav-item--active::before/, "The navigation active state must use a tonal container, not a side bar.");
for (const match of styles.matchAll(/font-size:\s*(\d+)px/g)) {
  assert.ok(Number(match[1]) >= 12, `Essential text must not use ${match[1]}px.`);
}
assert.match(styles, /max-width: 1279px/, "The compact sidebar breakpoint is missing.");
assert.match(styles, /max-width: 1023px/, "The small desktop breakpoint is missing.");
assert.match(styles, /@media \(max-width: 1023px\)/, "The compact desktop layout breakpoint is missing.");
assert.match(styles, /prefers-reduced-motion/, "Reduced-motion preferences must be respected.");
assert.match(styles, /:focus-visible/, "Visible keyboard focus styles must be present.");
assert.match(app, /await navigator\.clipboard\.writeText\(value\)/, "Copy controls must write the exact original value to the clipboard.");
assert.match(app, /onCopy\(copyLabel, value\)/, "Metric copy controls must pass the unmodified value.");
assert.match(styles, /overflow-wrap: anywhere/, "Long technical values must wrap instead of being hidden.");
assert.match(app, /aria-live="polite"/, "Copy feedback must be announced to assistive technology.");
assert.match(app, /aria-modal="true"/, "Preview notices must use a modal dialog.");
assert.match(app, /disabled title="Connection actions are disabled in this prototype"/, "The disabled connection action must be represented.");
assert.match(app, /role="status"/, "The loading state must be exposed as status.");
assert.match(app, /role="alert"/, "The error state must be announced as an alert.");

assert.ok(buildScript.includes('icon_directory.join("icon.png")'), "The build must generate the default PNG window icon.");
assert.ok(buildScript.includes('icon_directory.join("icon.ico")'), "The build must generate the required Windows ICO resource.");
assert.match(buildScript, /\[16_u32, 24, 32, 48, 64, 256\]/, "The Windows ICO must contain all required desktop icon sizes.");
assert.ok(buildScript.includes("ico::IconDirEntry::encode"), "The Windows icon must be encoded as a valid ICO collection.");
const config = JSON.parse(tauriConfig);
const windowConfig = config.app.windows[0];
assert.equal(windowConfig.minWidth, 960, "Tauri must enforce the 960px minimum width.");
assert.equal(windowConfig.minHeight, 640, "Tauri must enforce the 640px minimum height.");
assert.equal(config.build.frontendDist, "../dist", "Tauri must load the built frontend.");

assert.doesNotMatch(rustMain, /invoke_handler/, "The prototype must not register backend command handlers.");
assert.doesNotMatch(app, /\b(invoke|fetch|XMLHttpRequest)\s*\(/, "The frontend must not call backend commands or network endpoints.");

console.log("NetShift shell checks passed: English LTR, representative values, responsive guards, and no network commands.");
