# Architecture — Nurbit (beta)

Status: **approved 2026-11-14** (Rust-first desktop). Supersedes the earlier
browser/Node plan, which was never implemented.

Nurbit is an installable **desktop application** (Windows first; macOS and
Linux possible later). It works without a server; a separate Rust server
receives synchronized records and serves medical specialists.

## 1. Technologies and why

| Technology | Role | Why this one |
|------------|------|--------------|
| **Rust** | All logic: medical rules, validation, storage, sync, server | Memory-safe, fast, one language for desktop core and server; strict types catch mistakes in medical records at compile time. |
| **Tauri 2** | Desktop shell: window, installer, Rust ↔ UI bridge | Small installers (uses the system WebView2 instead of bundling a browser), Rust backend by design, permission system (capabilities). |
| **Tauri commands** | UI calls Rust with `invoke("name", args)` | In-process IPC (message passing inside the app), not HTTP; no localhost server in the installed app. |
| **Capabilities** | Allow-list of what the UI may call | The UI gets only `core:default`; files, network and database are reached only through Nurbit's own commands. |
| **React + TypeScript + Vite** | Screens, forms, navigation, charts | Established UI stack. Vite builds static files that are embedded into the `.exe`. |
| **SQLite** | Local database on the astronaut's computer; server DB for the demo | One file, no installation, works offline. |
| **SQLx** | Database access in desktop and server (Stages 4, 10) | One library for both; async (does not block while waiting for the DB); built-in migrations; supports PostgreSQL later. Queries are plain runtime-checked functions so builds do not need a live database. |
| **Axum + Tokio** | Server HTTP API (Stage 10) | Mainstream Rust web framework from the Tokio team. |
| **reqwest (rustls)** | HTTPS client inside the desktop app for sync (Stage 10) | Standard Rust HTTP client; rustls = TLS written in Rust. |
| **serde, uuid v7** | JSON (de)serialization; record ids | uuid v7 ids are unique without a server and sort by time. |
| **argon2** | Password hashing (local login, server login) | Current recommended password hash. |
| **ts-rs** | Generates TypeScript types from Rust structs (from Stage 4) | Rust stays the single source of truth for data shapes. |

## 2. Repository layout

```
nurbit/
├── Cargo.toml                  Cargo workspace: all Rust crates build/test together
├── crates/
│   ├── shared/                 data types: RecordId, DataOrigin, (later) records, sync DTOs
│   ├── medical-core/           PURE logic: daily questions, rules, baselines, risk modules,
│   │                           cross-risk analysis. No database, files, network or clock.
│   └── local-store/            desktop SQLite: migrations, repositories, sync outbox (Stage 4)
├── apps/
│   ├── desktop/
│   │   ├── src/                React UI; src/api.ts is the only file that calls Rust
│   │   └── src-tauri/          Tauri app: commands, setup, (later) sync service
│   │       ├── capabilities/   UI permissions
│   │       ├── icons/          app icon (generated from icon-source.png)
│   │       └── tauri.conf.json window, CSP, bundle (NSIS installer)
│   └── server/                 Axum + SQLx server (Stage 10; placeholder now)
├── knowledge-base/             verified excerpts for the assistant (Stage 11)
└── docs/
```

**Layering rule.** `medical-core` depends only on `shared`. Storage and apps
depend on `medical-core`, never the other way round. Medical calculations never
live in TypeScript or SQL.

## 3. Communication

```
[React UI] ──invoke()──► [src-tauri commands] ──► medical-core   (analysis, pure)
                                   │          └──► local-store    (SQLite file in the user's app-data folder,
                                   │                               records + outbox)
                     background task: server reachable?
                                   ▼
                       HTTPS + JSON (DTOs from crates/shared)
                                   ▼
                         [Axum server] ──► SQLx ──► server DB ──► specialist mode of the same app
```

- **Astronaut mode:** fully local; the server is optional.
- **Specialist mode:** the same desktop app; signs in to the server and reads
  synchronized crew data (cached locally). No separate website.

## 4. Data rules

Every medical record has: `id` (UUID v7, created on the device), `created_at`
(device clock), `received_at` (server clock, after sync), `device_id`,
`author_id`, `origin` (`DataOrigin`), `content_hash`, `sync_status`.

- **Append-only:** records are never edited in place; corrections are new
  annotation records (old value, new value, reason, author).
- **Separate tables** for machine output (findings, assistant answers) and
  human decisions (notes, decisions, second review).
- **Baselines** are versioned; a change is *proposed* by the system and
  *approved* by a specialist.
- **Thresholds** in risk configs carry `basis`: `source:<REFERENCES id>` or
  `prototype_assumption` (shown in the UI as needing specialist validation).
- Schema changes only through numbered SQL migrations.

## 5. Synchronization

1. Saving a record also writes an outbox entry in the same SQLite transaction.
2. When the server is reachable, a background task sends outbox entries in batches.
3. The server inserts by `id`: new → `accepted`; same id and hash → `duplicate`
   (safe); same id, different hash → `conflict` (both versions kept for a specialist).
4. Each record is acknowledged separately, so an interrupted sync simply resumes.
5. The client pulls specialist notes, decisions and approved baselines since its
   last cursor.

## 6. Server database: SQLite vs PostgreSQL

| | SQLite (beta choice) | PostgreSQL (later) |
|-|----------------------|--------------------|
| Setup | none (one file) | separate service |
| Concurrent writers | one at a time | many |
| Good for | demo crew, one server process | real multi-user deployment, roles, backups |

SQLx supports both; SQL is kept portable so only migrations need dialect changes.

## 7. Security (prototype level)

- Synthetic data only; banner on every screen.
- Strict CSP in `tauri.conf.json`; minimal capabilities; no shell/file plugins.
- Role checks in Rust (desktop commands and server routes), never only in the UI.
- Passwords hashed with argon2; HTTPS for sync; secrets never in the repository.
- Local SQLite is **not encrypted** in the beta (documented limitation;
  SQLCipher is an option later).
- Audit log of medical data changes.

## 8. Packaging

`npm run tauri build` (in `apps/desktop`) builds the UI, compiles Rust in release
mode and produces an **NSIS installer `.exe`**. Target PC: Windows 10/11 with
WebView2 (preinstalled on Windows 11). The beta installer is **not code-signed**,
so Windows SmartScreen / Smart App Control may warn or block it (see §9).

## 9. Development-environment notes

- **Smart App Control (SAC)** on the development PC blocks unsigned programs that
  have no cloud reputation. Observed 2026-11-14:
  - Rollup's native module (`rollup.win32-x64-msvc.node`) was blocked → the UI
    uses `@rollup/wasm-node` via `overrides` in `apps/desktop/package.json`
    (same Rollup, compiled to WebAssembly).
  - Every Cargo build script compiled locally was blocked (os error 4551), so
    **no Rust code can be built or run while SAC is on**. Options: turn SAC off
    (user decision), build in another environment, or sign binaries with a
    trusted certificate. Signing the beta is not planned.
  - Resolution: the developer turned SAC off on 2026-11-14. Other PCs with SAC on
    may block the unsigned Nurbit installer.
