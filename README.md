# Nurbit (beta) — "nurse in orbit": astronaut health monitoring, NASA Space Apps prototype

Installable **desktop application** (Tauri 2 + Rust) for routine crew health
monitoring: short daily check-ins, adaptive follow-up questions, individual
baselines, configurable NASA HRP risk modules, local-first storage with later
synchronization, a specialist mode, and a source-grounded assistant.

> Research/education prototype. **Not** a medical device, **not** clinically
> validated, **not** certified for spaceflight or clinical use. All crew profiles
> and medical records are **synthetic**.

**Status:** all 12 beta stages done; demo script and metrics in docs/DEMO.md; (incl. retrieval-only assistant) (incl. specialist mode with two-specialist review and the Rust sync server): Rust workspace, Tauri desktop app, local SQLite with append-only records, synthetic crew, daily check with adaptive follow-ups, trends against each person's own baseline, evaluation of the five approved risk modules, cross-indicator patterns. All thresholds are prototype assumptions except sourced NASA limits.

## Layout
| Path | What |
|------|------|
| `crates/shared` | Shared data types |
| `crates/medical-core` | Pure medical logic (no I/O), unit-tested |
| `crates/local-store` | Local SQLite storage (Stage 4) |
| `apps/desktop` | Tauri desktop app: `src/` React UI, `src-tauri/` Rust |
| `apps/server` | Rust server (Stage 10) |
| `docs/` | Specification, architecture, plan, references, AI usage |

## Commands (Windows, PowerShell)
```
cargo test --workspace                 # Rust tests (from repo root)
cd apps/desktop; npm install           # once: UI dependencies
cd apps/desktop; npm run tauri dev     # run the app in its own window (dev mode)
cd apps/desktop; npm run tauri build   # NSIS installer -> target/release/bundle/nsis/
$env:NURBIT_SYNC_TOKEN = "<your own secret, 16+ characters>"
cargo run -p nurbit-server             # sync server on http://127.0.0.1:8787 (enter the same token on the Sync screen)
```

## Documents
[Specification](docs/PROJECT_SPECIFICATION.md) ·
[Architecture](docs/ARCHITECTURE.md) ·
[Development plan](docs/DEVELOPMENT_PLAN.md) ·
[References](docs/REFERENCES.md) ·
[Feasibility](docs/FEASIBILITY.md) · [Demo](docs/DEMO.md) ·
[AI usage](docs/AI_USAGE.md)
