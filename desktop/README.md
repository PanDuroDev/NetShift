# NetShift Desktop Prototype

This directory contains the first Tauri 2 + React + TypeScript + Vite shell for NetShift.

## Scope

- English UI copy only; the document root and every screen use left-to-right (LTR).
- Static sample values only. The UI does not inspect Windows adapters, routes, DNS, provider processes, VPN services, or public IP addresses.
- No Tauri command handlers are registered. No frontend IPC command is invoked.
- Provider and diagnostic actions display a preview notice instead of running an operation.
- Light and dark themes share centralized CSS tokens.
- The minimum window size is 960 × 640 logical pixels.

## Requirements

- Node.js 22.12 or newer.
- Rust 1.90 or newer for Tauri 2.12.x.
- Windows: Microsoft C++ Build Tools and WebView2 Runtime.
- Linux: Tauri's WebKitGTK 4.1 development dependencies.

The Tauri 2.12 line raised its minimum supported Rust version to 1.90. The root NetShift library keeps its existing Rust 1.85 MSRV; the desktop shell is a separate crate so it does not raise the core library's MSRV.

## Run

From this directory:

    npm install
    npm run tauri dev

Build and run the static checks:

    npm run check
    cargo check --manifest-path src-tauri/Cargo.toml

The frontend can also run in a browser for layout review:

    npm run dev

## Current validation boundary

Automated checks cover TypeScript compilation, Vite production bundling, English/LTR declarations, representative technical values, responsive breakpoints, reduced motion, focus styling, the Tauri minimum window size, and the absence of registered network or provider commands.

Actual native window rendering, clipboard round-trip behavior, screen-reader behavior, and the four reference viewport screenshots still require manual verification on Windows and Linux. This prototype is not a production network manager.
