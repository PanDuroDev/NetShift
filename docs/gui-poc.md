# Desktop GUI Proof of Concept

## Status

The first Tauri desktop shell is under implementation. It is a UI-only prototype, not a functioning network manager.

## Framework and version baseline

- Tauri 2.12.5 (stable 2.x line).
- React 19.3.1.
- TypeScript 7.0.2.
- Vite 8.3.5 with `@vitejs/plugin-react` 6.1.2.
- Rust 1.90 for the separate Tauri crate.
- Node.js 22.12 or newer (CI currently uses Node 24).

Tauri 2.12 raised its MSRV to Rust 1.90. The root `netshift` library keeps `rust-version = "1.85"`; the desktop crate is intentionally separate so the core's MSRV is not raised just to host the GUI.

## UX scope

- English-only copy and left-to-right layout on every screen.
- Main areas: Overview, Providers, Diagnostics, and Settings.
- Desktop reference sizes: 1440×900, 1280×720, 1024×700, and 960×640.
- Expanded navigation at widths of 1280 px and above; compact icon navigation below 1280 px.
- Centralized light/dark color tokens and a restrained cyan accent.
- Keyboard-visible focus, semantic landmarks and labels, a skip link, live announcements for copy feedback, and reduced-motion support.
- Sample network values are selected from documentation-only ranges where applicable.

## Deliberately disabled behavior

- No live adapter, route, DNS, gateway, or public IP inspection.
- No provider discovery, installation, connection, disconnection, or process execution.
- No network probes or DNS requests.
- No file reads or log export.
- No Tauri IPC command handlers, shell plugin, filesystem plugin, or network-control APIs.
- Refresh only updates the sample timestamp. Provider and diagnostic actions display a notice.

The frontend does not import or call `@tauri-apps/api/core`. The Tauri backend uses only `tauri::Builder::default()` and does not register an `invoke_handler`.

## Automated checks

`npm run check` performs:

1. TypeScript strict compilation.
2. A Vite production build.
3. A static shell guard for English/LTR declarations, representative IPv4/IPv6/MAC/interface/path values, responsive breakpoints, minimum window size, reduced motion, focus styling, and the absence of backend or network calls.

CI additionally runs `cargo check` for the isolated Tauri crate on Windows and Linux. The existing root Rust formatting, Clippy, tests, and release build remain in place.

## Manual validation still required

- Launch the native Tauri window on Windows and Linux.
- Inspect all four reference viewport sizes and check for clipping/overlap.
- Copy representative IPv4, IPv6, MAC, port, interface ID, Windows path, and log values; paste into a plain-text editor and compare exact characters.
- Verify keyboard traversal and focus order, dialog focus trapping, screen-reader names/announcements, and contrast using native platform accessibility tools.
- Measure startup and idle/interaction resource use for information only; these values are not a framework-selection gate in this stage.

Do not close GUI Stage 1 until manual checks and the remaining acceptance criteria have been demonstrated.
