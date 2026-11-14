# Development Plan — Nurbit (beta)

Status: Rust-first desktop plan approved 2026-11-14 · Last updated: 2026-11-14

Rule for every stage: the project stays runnable, tests pass, docs are updated,
and the developer can explain the new code before the next stage starts.

## 1. Current state
- Earlier browser/Node plan dropped before any code was written.
- Toolchain: Rust 1.97 (MSVC), Node 24.20, npm 11.19, WebView2 157, Tauri 2.12.
- Smart App Control was turned off by the developer on 2026-11-14; Rust now
  builds and runs (see `ARCHITECTURE.md` §9).

## 2. Stages

| # | Stage | Done when | State |
|---|-------|-----------|-------|
| 1 | Rust workspace and crates (`shared`, `medical-core`, `local-store`, desktop, server) | `cargo test --workspace` passes | Done (9 tests, clippy clean) |
| 2 | Tauri shell: window, icon, Rust ↔ UI command, CSP, minimal capabilities | `npm run tauri dev` opens the Nurbit window | Done (window verified) |
| 3 | Navigation and UI layout | All main sections reachable, synthetic-data banner | Done (7 screens; risk catalogue served from Rust) |
| 4 | Local SQLite (SQLx, migrations, repositories) | Repository tests on a temporary DB | Done (append-only triggers, outbox, audit log; 7 tests) |
| 5 | Astronaut profiles + synthetic seed (login without password in the beta) | Profiles load offline | Done (4 astronauts + 1 flight surgeon, 14 days synthetic history) |
| 6 | Daily questionnaire + adaptive follow-ups (rules in `medical-core`) | Table-driven rule tests; check-in saved | Done (verified in the app window) |
| 7 | Medical history and charts; baseline comparison | Trends shown; baseline/deviation tests | Done (trend charts with own-baseline band; within / temporary deviation / persistent change; baseline never auto-updated) |
| 8 | Risk-module framework + five modules + cross-risk analysis | Each threshold has `basis`; evidence labels | Done (shared evaluation, data gaps shown, simulated exercise/dosimeter feed, correlations labelled as observations) |
| 9 | Specialist mode: notes, annotations, baseline approval, two-specialist decisions | Permission tests; AI vs human records separate | Done (role checked in Rust on every action; append-only specialist tables; duty restriction needs a second specialist, verified in the app) |
| 10 | Axum server + synchronization | No duplicates, conflicts keep both versions, interrupted sync resumes | Done (token auth, SHA-256 content hash, per-record acks, outbox retry every 30 s; offline→online verified in the app) |
| 11 | Assistant: retrieval over verified local knowledge base, provider trait | Every answer shows sources or "no reliable source found" | Done (14 source-checked excerpts, keyword retrieval, red-flag and medication guards, providers labelled, questions logged for specialists) |
| 12 | End-to-end demo scenario, metrics, NSIS installer | Installer installs and launches; scenario passes | Done (NSIS installer 4 MB built; release exe launched standalone; demo scenario tests + DEMO.md; installer itself not run on this PC) |

## 3. Approved risk modules (developer decision, 2026-11-14)

1. Risk of Bone Fracture due to Spaceflight-induced Changes to Bone
2. Risk of Spaceflight Associated Neuro-ocular Syndrome (SANS)
3. Risk of Adverse Cognitive or Behavioral Changes and Psychiatric Disorders
4. Risk of Performance Decrements and Adverse Health Outcomes Resulting from Sleep Loss, Circadian Desynchronization, and Work Overload
5. Risk of Radiation Carcinogenesis

These replace the earlier candidates (cardiovascular and fitness were dropped).
Per-risk feasibility (12 questions, indicator classes, limits): `FEASIBILITY.md`.
Configuration: `crates/medical-core/src/risk_catalog.rs`.

## 4. Open decisions (need the developer)

| ID | Decision | Default if no answer |
|----|----------|----------------------|
| D-1 | TypeScript vs plain JavaScript | TypeScript |
| D-2 | Approve the dependency list in `ARCHITECTURE.md` §1 | Install nothing until approved |
| D-3 | Final five risks | Table §3 rows 1–5 |
| D-4 | Initialise a git repository in `crew-health-monitor/` | Ask before `git init` |
| D-5 | Scale direction for Q2 (10 = no symptoms vs 0 = no symptoms) | 10 = best for all three, pending specialist input |
| D-6 | Encrypt offline data on the device in the beta | Document as limitation; do not implement in beta |
| D-7 | Demo target: browser only, or also a packaged desktop app | Browser only |

## 5. Main technical and scientific risks

| Risk | Why it matters | Mitigation |
|------|----------------|------------|
| No validated thresholds for many indicators | Inventing numbers would be scientifically wrong | `basis` field; "prototype assumption — needs specialist validation" label; individual-baseline deviation instead of absolute cut-offs where possible |
| Validated questionnaires may be licensed | Copying them could be a licence violation | Use own wording for daily items; add a validated instrument only after checking its licence |
| Small synthetic datasets | Correlations/trends can be noise | Minimum-sample rule; evidence labels; no causal wording |
| Over-alerting | Alert fatigue defeats the purpose | Count unnecessary alerts in test scenarios; tune with persistence windows |
| Sync bugs can lose or duplicate medical data | Core safety requirement | Append-only records, idempotent ids, per-record acks, dedicated tests |
| Assistant looks more capable than it is | Misleading in a medical context | Retrieval-only beta, provider label, "not a diagnosis", no dosages |
| Browser speech recognition may need the cloud | Breaks the offline promise | Voice off by default, labeled "online only" until verified |
| Scope is very large for one student | Unfinished, unexplained code | Stage gates; small commits; explanation per stage |
| Device clock drift | Wrong ordering of records | Keep device and server timestamps |
