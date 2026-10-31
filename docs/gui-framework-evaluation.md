# Desktop GUI Framework Evaluation

**Status:** Initial evaluation complete; framework choice is provisional pending a working proof of concept.

**Scope:** Windows-first desktop application, with Linux portability as a target. The UI language is English and the interface direction is left-to-right (LTR). The existing Rust network core remains independent of the presentation layer.

## Decision summary

Evaluate a web-based presentation layer hosted by **Tauri 2**, using **React + TypeScript + Vite**, as the leading candidate.

This is a deliberate separation of concerns:

- Rust owns network inspection, provider contracts, lifecycle management, and all privileged operations.
- The frontend owns presentation, navigation, accessibility semantics, localization, and user interaction.
- Tauri IPC exposes a small, typed command surface. The frontend must not receive unrestricted system access.
- The GUI must not initiate network-changing operations during startup, rendering, refresh, or navigation. Operations require an explicit user action and truthful provider confirmation.

This is not a claim that Tauri is a native widget toolkit. It is a desktop shell with a web-rendered UI.

## Candidate comparison

| Criterion | Tauri 2 + React/TypeScript | Slint + Rust | egui + Rust |
|---|---|---|---|
| Architecture | Rust backend + web frontend over IPC | Rust-native declarative UI DSL | Rust immediate-mode UI |
| Windows | WebView2; supported on Windows 10+ | Tested on Windows 10/11 | Native desktop support |
| Linux | WebKitGTK 4.1 dependency for Tauri 2 | Supported desktop target | Native desktop support |
| Material 3 direction | Implement tokens/components in frontend; full expressive behavior must be verified | Official Slint Material component set based on Material 3 | Custom implementation required |
| English LTR / technical text | Browser engine provides mature text rendering; test English labels alongside IP/MAC/IPv6 values | Validate font rendering and text editing in a prototype | Custom text and widget behavior still require validation |
| Accessibility | Semantic HTML and browser accessibility tree; test each platform WebView | Explicit accessibility roles/properties are available | AccessKit integration is available |
| Styling flexibility | High; CSS and component-level control | High; custom components and styles | High, but many widgets/states are application-owned |
| Licensing | Tauri MIT/Apache-2.0; frontend dependencies need individual review | GPLv3 or royalty-free desktop license with attribution; paid options also exist | MIT/Apache-2.0 |
| Main risk | Different system WebViews can render differently; frontend/Rust IPC boundary needs strict security | Licensing/attribution and text rendering require decisions and validation | Arabic bidi/text editing and extensive custom M3 work |

### Tauri 2

**Strengths**
- Keeps the current Rust core and places a narrow presentation boundary around it.
- Supports a frontend built with standard HTML/CSS/TypeScript; Tauri recommends Vite for SPA frameworks such as React.
- Semantic HTML gives the UI a strong baseline for keyboard navigation, screen readers, focus handling, and a consistent LTR layout.
- Uses WebView2 on Windows and WebKitGTK on Linux rather than bundling a full browser.
- Tauri's capability model can restrict which commands the frontend is allowed to invoke.

**Costs and risks**
- Windows and Linux use different WebView engines and versions. Font rendering, CSS details, and accessibility behavior must be tested on both.
- Linux builds require WebKitGTK 4.1 and its development packages.
- The frontend/Rust boundary is a security boundary. Never expose generic shell, process, filesystem, or network-control commands to the frontend.
- Current Tauri 2.12.x metadata specifies Rust 1.90; the repository currently declares Rust 1.85. Adopting this version requires an explicit MSRV decision and CI update.

**Assessment:** Best fit for the required English-only LTR UI and the project's detailed, custom Material 3 Expressive design target, provided the proof of concept validates platform behavior and resource use.

### Slint

**Strengths**
- A declarative UI toolkit with an official Rust API and Windows/Linux desktop support.
- The Slint Material component set is based on Material 3 and can accelerate the initial component implementation.
- Accessibility roles and properties are part of the toolkit.
- Suitable when keeping the whole application UI in a Rust-oriented stack is a priority.

**Costs and risks**
- The free licensing choices are not simply MIT/Apache: GPLv3 requires the distributed application to be GPLv3-compatible, while the royalty-free desktop license requires Slint attribution in an About screen or public download page.
- The available Material set is based on Material 3; Material 3 Expressive parity must not be assumed.
- Font rendering, text editing, and mixed technical values need a real prototype before selection.

**Assessment:** Strongest Rust-native alternative and worth keeping as the fallback if its Arabic text behavior and licensing are acceptable.

### egui

**Strengths**
- Mature Rust ecosystem, cross-platform desktop support, and a straightforward path to custom dashboards.
- AccessKit support provides a foundation for native accessibility APIs.
- MIT/Apache-2.0 licensing.

**Costs and risks**
- The application would need to implement most of the specified Material component styling and interaction states itself.
- The project has known open text-rendering issues that should be checked in the prototype.
- Immediate-mode UI is viable, but it is less aligned with the design-system/component-first implementation approach than the leading candidates.

**Assessment:** Good for developer tools and diagnostic panels, but not the preferred foundation for this design-system-driven GUI.

## Proposed frontend/backend boundary

Keep the current Rust library as the source of truth. The GUI must consume stable application-facing types rather than directly accessing Windows APIs or provider implementations.

Initial boundary proposal:

1. Read-only command: request a fresh network snapshot.
2. Read-only command: enumerate providers and inspect their metadata, capabilities, availability, and status.
3. Explicit connect/disconnect commands only after a confirmation flow and only through the existing provider manager.
4. Typed, sanitized errors and operation reports returned to the UI.
5. No arbitrary command execution, direct route/DNS/adapter/firewall mutation, background IP rotation, or implicit operations.

The first proof of concept must implement only a static shell and mock data. It must not call real provider operations or modify network configuration.

## Proof-of-concept acceptance tests

The Tauri candidate is not final until a minimal shell demonstrates all of the following:

- Builds and opens on Windows; Linux build/runtime is checked separately.
- Navigation sidebar, top app bar, and responsive content area follow the Issue #7 measurements.
- Light/dark tokens and a cyan/teal accent can be changed centrally.
- All visible UI copy is English (US) only in this phase; no Arabic translations or language switcher are required.
- The entire application uses LTR direction, left-aligned navigation/content, and LTR keyboard/focus order.
- Keyboard-only navigation, visible focus, semantic names, and screen-reader exposure are verified.
- Buttons, fields, cards, dialogs, tooltips, status indicators, and disabled/loading/error states are represented.
- Window sizes 1440×900, 1280×720, 1024×700, and 960×640 are checked.
- The shell uses mock network/provider data only; no real network mutation is possible.
- The existing Rust tests and Windows CI remain intact.

## Version and dependency notes

- Candidate shell: Tauri 2.12.x.
- Candidate frontend: React + TypeScript + Vite.
- Do not add a Material component dependency until its maintenance status, license, accessibility, LTR behavior, and Material 3 Expressive coverage have been checked.
- Keep design tokens in one frontend theme module and avoid per-screen hardcoded values.
- Re-check current upstream versions and Rust MSRV when the proof-of-concept branch is created.

## Sources

- Tauri architecture: https://v2.tauri.app/concept/architecture/
- Tauri frontend configuration: https://v2.tauri.app/start/frontend/
- Tauri security model: https://v2.tauri.app/security/
- Tauri WebView versions: https://v2.tauri.app/reference/webview-versions/
- Tauri project and license: https://github.com/tauri-apps/tauri
- Slint desktop support and styles: https://slint.dev/desktop
- Slint Material components: https://material.slint.dev/
- Slint license terms: https://slint.dev/terms-and-conditions
- Slint accessibility properties: https://docs.slint.dev/latest/docs/slint/reference/common/
- egui accessibility: https://github.com/emilk/egui/blob/main/docs/accessibility.md
- egui bidi tracking issue: https://github.com/emilk/egui/issues/1016
- iced RTL FAQ: https://book.iced.rs/faq

## Current conclusion

Proceed with a **Tauri 2 + React/TypeScript/Vite proof of concept**, but do not treat the framework as permanently selected until the English-only LTR, accessibility, Windows/Linux, sizing, and resource-use tests above pass. Slint remains the Rust-native fallback. No production GUI or network operation is included in this evaluation.


## Language and direction requirement (confirmed)

- The application UI is English-only for the current project scope. All navigation labels, buttons, headings, dialogs, tooltips, validation messages, status text, and documentation-facing UI copy must be written in English.
- The application root and all screens use left-to-right direction (`dir="ltr"`). Layout alignment, navigation placement, reading order, and keyboard/focus order must be LTR.
- Do not add Arabic translations, Arabic-first typography, an RTL mode, or a language switcher in this phase.
- Network identifiers (IPv4, IPv6, MAC addresses, ports, interface IDs, file paths, and logs) remain selectable and copyable; render code-like values in an LTR-safe style.
