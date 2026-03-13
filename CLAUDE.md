# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ClamAV Desktop is a cross-platform desktop GUI for ClamAV antivirus built with **Tauri v2** (Rust backend + React/TypeScript webview). Currently in alpha (v0.x).

## Common Commands

### Development
- `yarn` — install dependencies
- `yarn dev` — full dev mode (Vite webview + Tauri dev server)
- `yarn dev:webview` — frontend only (Vite on :1420)
- `yarn dev:daemon:linux` / `macos` / `windows` — build & install platform daemon

### Building
- `yarn build` — compile Tauri app (`cd src-tauri && cargo build`)
- `yarn build:webview` — production frontend build
- `yarn build:daemon` — compile daemon service
- `yarn bundle:deb:x64` / `bundle:dmg` / `bundle:msi:x64` — platform installers

### Testing
- `yarn test:lint` — Biome linting with auto-fix
- `yarn test:type` — TypeScript type checking
- `yarn test:unit:webview` — Jest for React (watch: `yarn test:unit:webview:watch`)
- `yarn test:unit:core` / `make test` — all Rust tests
- `make test-cli` / `make test-common` / `make test-filer` — individual Rust crate tests
- `yarn test:e2e` — WebdriverIO E2E tests

### Other
- `yarn test:sec` — secret scanning (ggshield)
- `yarn test:perms` — permission audit

## Architecture

### Hybrid Rust/TypeScript via Tauri v2

**Frontend (webview):** React 19 + TypeScript + Styled Components + Vite, in `src/`.

**Backend:** Rust with Tauri, in `src-tauri/`. This is a Cargo workspace with members: `cli`, `common`, `config`, `dev`, `fast-cli`, `filer`.

**Daemon:** Standalone Rust service in `daemon/` that runs as a background process (systemd/launchd/Windows service). Communicates with the Tauri app via WebSocket.

**Sidecars:** Pre-built ClamAV binaries (`clamd`, `clamscan`, `freshclam`) bundled and executed via `tauri-plugin-shell`.

### Tauri IPC Pattern

Webview modules in `src/core/` (Cloud, Scanner, DaemonClient, FileManager, Copilot) are type-safe wrappers around Tauri commands and events defined in the corresponding Rust modules under `src-tauri/src/modules/`.

Each Rust module follows a consistent structure:
```
module/
├── commands.rs   # Tauri command handlers (invoked from webview)
├── state.rs      # Shared state (managed via Tauri's Manager)
├── types.rs      # Type definitions
└── mod.rs        # Public API
```

### State Management

Per-module shared state managed through Tauri's state system: `CopilotSharedState`, `DashboardSharedState`, `CloudSharedState`, `ScannerSharedState`, `SharedSettingsState`.

### Screens

Dashboard (daemon status/control), Scanner (file scanning), Settings (clamd.conf editor), Copilot (setup wizard), Loader (initialization).

## Conventions

- **Commit format:** Conventional Commits — `type(scope): message`. Types: `build`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `style`, `test`. Scopes: `cloud`, `dashboard`, `daemon`, `scanner`, `settings`
- **Tool versions:** Node 24, Rust 1.93 (managed via `mise.toml`)

## Code Style

### Rust

- **Line width:** 120 characters (`.rustfmt.toml`)
- **Tauri commands:** always annotated with both `#[tauri::command]` and `#[cfg(not(tarpaulin_include))]`. Return `Result<(), ()>`. First line is always `debug!("function_name()", "Command call.")`
- **State pattern:** `SharedState` wraps a `State` struct containing `public: Arc<Mutex<PublicState>>` and optional `private` fields. Mutations lock the mutex, update, `drop(guard)`, then call `broadcast_state()`
- **Derives on public state:** `Clone, Debug, Default, Deserialize, Serialize, PartialEq`
- **Error handling:** commands log errors with `error!()` macro and return `Err(())`. `.expect()` with descriptive messages for infallible unwraps
- **Import order:** std → external crates → `crate::` → `super::`
- **Test files:** named `*_test.rs` (not inline `#[cfg(test)]` modules). Uses `jrest` crate with `expect!` macro

### TypeScript / React

- **Biome** for all linting and formatting (not Prettier). Config extends `@ivangabriele/biome-config`
- **Biome assist rules:** `organizeImports`, `useSortedAttributes`, `useSortedInterfaceMembers`, `useSortedKeys`, `useSortedProperties` — all properties, keys, and interface members must be sorted
- **Naming convention:** enforced by Biome — camelCase for variables/functions, PascalCase for components/types, UPPER_SNAKE_CASE for constants. Object literal properties allow camelCase, PascalCase, or snake_case
- **Strict TypeScript:** `strict: true`, `noUnusedLocals`, `noUnusedParameters`, `noUncheckedIndexedAccess`, `useUnknownInCatchVariables`, `verbatimModuleSyntax`
- **Path aliases:** `@screens`, `@components`, `@core`, `@elements`, `@hooks`, `@libs`, `@layouts`, `@utils`
- **Components:** functional only, named exports. Props typed as `Readonly<{...}>`. Styled components defined at end of file with `$` prefix for transient props (e.g. `$isCentered`)
- **Tauri invoke pattern:** core modules export objects with async methods wrapping `invoke<Type>('command_name', {...})`
- **TypeScript namespaces:** used for grouping related types per module (e.g. `namespace Cloud { export interface State {...} }`)
- **Import style:** `import type` for type-only imports, path aliases for cross-directory, relative for same directory

## Release Packaging

### Bundle Commands (local)
- `yarn bundle:deb:x64` — Debian package (x86_64, requires Linux)
- `yarn bundle:dmg:x64` — macOS disk image (x86_64, requires macOS)
- `yarn bundle:dmg:arm64` — macOS disk image (ARM64, requires macOS)
- `yarn bundle:msi:x64` — Windows installer (x86_64, requires Windows)
- `yarn bundle:msi:arm64` — Windows installer (ARM64, requires Windows)
- `yarn bundle:bin` — standalone binary (no installer)

All bundle commands run `yarn prebuild` first, which builds the webview and prepares core (downloads ClamAV sidecars).

### CI/CD Pipelines
- **`bundle.yml`** — runs on PRs and main pushes, builds DEB (x64), DMG (arm64 only currently), MSI (x64 + arm64). Uploads artifacts with 1-day retention. Uses Docker container `ivangabriele/tauri:debian-bookworm-22` for Linux builds.
- **`release.yml`** — triggered via `workflow_dispatch`. Currently only creates a GitHub Release draft via `release-drafter`. The `release` (build) and `publish` jobs are commented out.

### Bundle Configuration
- Tauri bundle config in `src-tauri/tauri.conf.json` under `bundle` — targets all formats, includes icons, polkit policy for Linux, entitlements for macOS
- External sidecars (`clamd`, `clamscan`, `freshclam`) are bundled with the app
- Build preparation scripts live in `scripts/build/` (sidecar download, normalization)
