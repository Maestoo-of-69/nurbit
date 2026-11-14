# Nurbit — "nurse in orbit"

**English** · [Русский](README.ru.md)

Nurbit is a beta desktop application for routine astronaut health monitoring,
built for the NASA Space Apps Challenge. It replaces long medical questionnaires
with a one-minute daily check, asks extra questions only when the answers call
for it, compares every person with their **own** history, follows five NASA
Human Research Program risks, works fully **offline**, and sends records to the
ground medical team when a link is available.

> **Research/education prototype.** Not a medical device, not clinically
> validated, not certified for spaceflight or clinical use. All crew profiles
> and medical records are **synthetic**. The software never diagnoses, never
> prescribes, and never declares anyone unfit for duty.

## Contents
1. [Quick start](#1-quick-start)
2. [Users and profiles](#2-users-and-profiles)
3. [Astronaut features](#3-astronaut-features)
4. [Specialist (flight surgeon) features](#4-specialist-flight-surgeon-features)
5. [Sync server](#5-sync-server)
6. [How data is protected](#6-how-data-is-protected)
7. [Architecture and technologies](#7-architecture-and-technologies)
8. [Tests and demo](#8-tests-and-demo)
9. [Limitations](#9-limitations)
10. [Documents](#10-documents)

## 1. Quick start

### Install (Windows 10/11)
1. Take `Nurbit_0.1.0_x64-setup.exe` (≈4 MB; built into `target/release/bundle/nsis/`).
2. Run it. It installs for the current user only (no admin rights) into
   `%LOCALAPPDATA%\Nurbit`, adds Start-menu and desktop shortcuts and an entry in
   *Settings → Apps* for uninstalling.
3. Start **Nurbit** from the Start menu.

*Why so small:* Tauri uses the WebView2 engine built into Windows instead of
bundling a browser. *Note:* the installer is not code-signed, so SmartScreen /
Smart App Control may warn on other PCs.

### Run from source
Requirements: Rust (MSVC, 1.97+), Node 24, WebView2.

```bash
cargo test --workspace                      # all Rust tests (repo root)
cd apps/desktop && npm install              # once: UI dependencies
cd apps/desktop && npm run tauri dev        # app in its own window, hot reload
cd apps/desktop && npm run tauri build      # release app + NSIS installer
```

### Fresh demo data
Close the app and delete `%APPDATA%\com.nurbit.desktop\nurbit.db`. The synthetic
crew and 14 days of synthetic history are created again on the next start.

## 2. Users and profiles

On start Nurbit asks **who is using it**. Pick a profile with one click.

| Profile | Role | Story in the demo |
|---------|------|-------------------|
| Vega, Orion, Lyra, Atlas | Astronauts | Vega: steady days. Orion: sleep getting worse. Lyra: lower mood in week 2. Atlas: missed resistive exercise last week. |
| Dr. Sirius | Flight surgeon (ground) | Reviews the crew, writes notes and decisions |
| Dr. Rigel | Flight surgeon (second reviewer) | Confirms or rejects duty-restriction recommendations |

- **How:** click a card; **Switch** (bottom left) changes the user.
- **Why:** astronauts and specialists get different menus and actions.
  Permissions are checked again in Rust on every action, so the menu alone
  cannot unlock anything.
- **Beta limitation:** no password yet (all profiles are synthetic).

The yellow banner *"Prototype – synthetic data only – not a medical device"* is
always visible.

## 3. Astronaut features

### 3.1 Overview (home screen)
Today's check status, the **monitoring status** (highest level across the five
risk modules and safety findings), records waiting to be sent, each module's
level and a shortcut to the assistant.
- **Why:** one glance answers "did I do my check?" and "does anything need attention?".

### 3.2 Daily check — 3 mandatory questions
| Question | 0 means | 10 means |
|----------|---------|----------|
| How rested do you feel after your last sleep period? | Not rested at all | Fully rested |
| How is your body today: pain, discomfort, illness or other symptoms? | Worst I can imagine | No symptoms at all |
| How are your mood and mental energy today? | Very low | Very good |

- **How:** click a number on each scale, then **Continue**.
- **Why:** a routine check should take about a minute. All scales use
  *10 = best*, so the direction never flips. The answers feed the sleep,
  behavioral, bone and SANS modules.

### 3.3 Adaptive follow-up questions
After **Continue**, Rust decides whether more questions are needed:
- **Low score:** an answer ≤ 3 opens that area's questions.
- **Drop from your own average:** an answer 3+ points below your average of the
  last 7 check-ins (at least 5 needed) opens them too.

| Area | Follow-up questions |
|------|---------------------|
| Sleep | hours slept, interrupted sleep, alertness now |
| Body | **safety question** (chest pain, fainting, sudden vision loss, sudden severe headache, difficulty breathing), where the problem is, fall or impact, vision change |
| Mood | stress, concentration, "would you like the medical team to contact you?" |

- **How:** each block shows *Why* it was asked; answer all, then **Save check-in**.
- **Why:** nobody answers every question every day, yet a concerning answer is
  never ignored. The safety question can never be skipped. Thresholds are
  marked *prototype assumption – needs specialist validation*.

### 3.4 Check summary and flags
After saving: time taken, questions asked, follow-ups triggered and notes:
**urgent symptoms** (safety question = yes) → red banner "contact the crew
medical officer now"; **contact requested**, **vision change**, **fall or
impact** → recorded for the medical team.
- **Why:** immediate feedback; normal results never create notifications.

### 3.5 History and trends
- **Trend charts** for the three ratings. The grey band is **your usual range**
  (own average ± 2 points from earlier check-ins). Filled dots = your answers,
  hollow = synthetic history; hover a dot for details.
- **Status:** *Within your usual range*, *Once below/above* (temporary
  deviation), *3× below · review* (persistent change), *Not enough history*.
- **Patterns between indicators:** rested ↔ mood and rested ↔ body in your data,
  always labelled *observed correlation*, next to the evidence level in the
  literature.
- **Table** of all check-ins (text view of the same data).
- **Why:** your own history says more than a population norm, and 0–10
  self-ratings have no medical reference range. The baseline is **never changed
  automatically**; only a specialist approves a new one. Correlations are never
  shown as causes.

### 3.6 Risk modules
| Module | Watched in flight | Cannot do |
|--------|-------------------|-----------|
| Bone changes | falls/impacts, back or limb pain, resistive-exercise minutes (simulated device) | measure bone density in flight (DXA is done before/after flight) |
| SANS (neuro-ocular) | vision change and headache reports | replace an eye examination |
| Behavioral health | mood trend, contact requests | diagnose a psychiatric disorder |
| Sleep and schedules | rested trend, sleep hours, interruptions | replace actigraphy (not connected in the beta) |
| Radiation exposure | cumulative dose vs NASA career limit **600 mSv** (simulated dosimeter) | detect cancer |

- **How:** choose a tab: the **automated assessment** (*No data / Nothing to
  report / Needs attention / Specialist review*) with each finding and its
  basis and missing data, then every indicator with its type (*measured,
  self-reported, derived, risk estimate, specialist finding*), in-flight
  availability, baseline comparison and beta data source.
- **Why:** monitoring is not diagnosis. The only numeric limits are NASA's own
  (600 mSv; bone density ≥ 95 % / 90 % of pre-mission, evaluated after flight).
  See [docs/FEASIBILITY.md](docs/FEASIBILITY.md).

### 3.7 Assistant (verified-source search)
- **How:** describe the symptom (e.g. *"My vision is blurred when I read"*),
  optionally pick body area, severity (0 = none, 10 = worst) and onset, then
  **Search verified sources**.
- **Result:** up to 3 source cards — summary, organization, document, date,
  section, *why shown*, reference id — the related risk module and an
  uncertainty note.
- **Safety rules:** red-flag words or severity ≥ 8 → urgent banner; medication
  or dose questions → refusal; no match → **"No reliable source found"**.
- **Providers:** *Verified-source search* — active, offline. *Local language
  model* — not installed. *Cloud assistant* — unavailable. **Voice input** is
  off: browser speech recognition sends audio online by default.
- **Why:** a safe first assistant that only repeats checked sources (14
  excerpts in `knowledge-base/entries.json`, each tied to
  [docs/REFERENCES.md](docs/REFERENCES.md)). Retrieval, not generative AI.
  Every question is logged for specialists.

### 3.8 Sync
- **How:** server address (default `http://127.0.0.1:8787`) + access token →
  **Save settings** → **Sync now**. Counters: *Waiting to send*, *Sent*,
  *Conflicts*; plus the last attempt's result. A background task retries every
  30 s. Sidebar indicator: *Local mode*, *Server linked* or *Offline*.
- **Why:** missions lose contact with Earth. Records are saved locally, queued,
  and delivered once when the link returns; synthetic history is never sent.

### 3.9 About
Version and prototype disclaimers.

## 4. Specialist (flight surgeon) features

### 4.1 Crew overview
All astronauts, highest concern first: automated status, last check-in, what
needs attention (incl. red-flag **safety findings**), decisions awaiting a
second review.
- **Why:** the whole crew in one place; know where to look first.

### 4.2 Astronaut record
- **Summary:** red banner for safety findings; automated assessment of all
  modules (*rule-based prototype · not a diagnosis*); trends. On a *persistent
  change* a **baseline review** appears — *Approve new baseline from now* or
  *Keep current baseline*, reason required. Also the assistant questions asked.
- **Check-ins:** every check-in with all answers and why each follow-up was
  asked. **Add annotation** stores a corrected value and/or comment with a
  required reason; the original stays unchanged.
- **Notes & decisions** (blue border = human judgment): follow-up
  examination, clear for duty, **recommend removal from duty (needs 2nd
  specialist)**, other; free-text notes.
- **Why:** specialists outrank the software; their judgment is stored apart
  from machine output and never overwrites originals.

### 4.3 Two-specialist review
A duty restriction gets *Awaiting second specialist*. In **Pending reviews**
another specialist can **Confirm / Reject** with a comment; the author cannot
(also enforced in Rust).
- **Why:** removing someone from duty must not rest on one person or on software.

## 5. Sync server

```bash
# PowerShell
$env:NURBIT_SYNC_TOKEN = "<your own secret, 16+ characters>"
cargo run -p nurbit-server        # http://127.0.0.1:8787
```

| Variable | Meaning | Default |
|----------|---------|---------|
| `NURBIT_SYNC_TOKEN` | required access token | — (server will not start) |
| `NURBIT_SERVER_DB` | database file | `nurbit-server.db` |
| `NURBIT_SERVER_ADDR` | listen address | `127.0.0.1:8787` |

| Endpoint | Purpose |
|----------|---------|
| `GET /health` | liveness, no token |
| `POST /api/v1/sync/push` | receive a batch of records |
| `GET /api/v1/crew` | synchronized crew profiles |
| `GET /api/v1/crew/{id}/checkins` | check-ins of one person |

Each record has an id and a SHA-256 content hash. Per-record answers:
**accepted** (new), **duplicate** (same content, safe re-send), **conflict**
(same id, different content — both kept, nothing overwritten), **invalid**.
`/api` needs `Authorization: Bearer <token>` (constant-time comparison).
- **Why:** idempotent, never-overwriting sync over unreliable links. Plain HTTP
  is for a local demo only; real use needs HTTPS.

## 6. How data is protected
- **Append-only records:** SQLite triggers reject `UPDATE`/`DELETE` on medical
  records, notes, decisions, reviews, assistant log and audit log.
- **One transaction per save:** check-in, answers, outbox and audit entry together.
- **Machine output vs human judgment:** separate tables, different look.
- **Origin and timestamp** on every value; synthetic data labelled everywhere.
- **Thresholds carry a basis:** source id or *prototype assumption*.
- **Secrets:** none in the repository; the server token comes from the environment.

## 7. Architecture and technologies

```
crates/shared/        data types, sync protocol, content hash
crates/medical-core/  pure medical logic: questions, follow-ups, baselines, risk
                      modules, patterns, review rules, assistant retrieval
crates/local-store/   desktop SQLite: migrations, repositories, outbox, seed
apps/desktop/         Tauri app: src/ React UI, src-tauri/ Rust commands + sync client
apps/server/          Axum sync server
knowledge-base/       assistant excerpts (JSON)
docs/                 specification, architecture, plan, feasibility, references, demo
```

### Technology stack — what is used and why

**Core language — Rust (1.97, MSVC).** All medical logic, storage, sync and the
server. *Why:* memory-safe without a garbage collector, fast, and its strict
type system catches wrong fields in medical records at compile time. One
language for the desktop core and the server. *Alternative:* Node/TypeScript
(the first plan) — easier to start, but more runtime errors.

**Desktop shell — Tauri 2 (2.12).** Window, installer, Rust ↔ UI bridge.
*Why:* real installable app (not a website), ~4 MB installer because it uses
the WebView2 engine built into Windows, Rust backend by design, and a
**capabilities** allow-list (the UI gets only `core:default`; no file, shell or
network access). *Alternative:* Electron — bundles a whole browser (100+ MB).

**Frontend — React 19 + TypeScript 5.9.** Screens, forms, navigation.
*Why:* the most established UI library; TypeScript types mirror the Rust
structs so wrong fields are caught before running. Navigation is plain React
state (a desktop app has no address bar), so no router library is needed.
**No medical rules live in TypeScript.**

**Frontend build — Vite 7** (with `@rollup/wasm-node`). Dev server with hot
reload; builds static files embedded into the `.exe`. *Why:* fast, simple
config. The WebAssembly build of Rollup is used because Windows Smart App
Control blocked Rollup's unsigned native module on the development PC.

**Styling — plain CSS with design tokens** (`tokens.css`). *Why:* every colour,
size and timing in one place; no CSS framework needed for a small app; colours
carry meaning (normal / attention / alert / review) and always come with text.

**Charts — hand-written SVG** (`TrendChart.tsx`). *Why:* one small chart type
(line + baseline band + hover tooltip); avoids a chart library dependency.

**Icons — inline SVG + generated app icon.** The app icon is drawn by a
standard-library Python script and cut into all sizes by `tauri icon`.

**UI ↔ core communication — Tauri commands (IPC).** The UI calls
`invoke("save_checkin", …)`; Rust runs it in-process. *Why:* no HTTP, no
localhost server in the installed app; every command re-validates input and
checks the user's role.

**Local database — SQLite** (bundled via SQLx). One file in the user's
app-data folder. *Why:* works fully offline, zero installation, reliable
transactions (WAL mode). Triggers make medical tables append-only.

**Database access — SQLx 0.8.** *Why:* async (does not block while waiting),
built-in migrations (`migrations/*.sql`, applied automatically at start), same
library on desktop and server, supports PostgreSQL later. Queries are plain
runtime-checked SQL, so builds never need a live database. The 0.8 line was
chosen over the very new 0.9.

**Server database — SQLite for the beta.** *Why:* one file, one command to run
the demo. *Trade-off:* one writer at a time; PostgreSQL is the planned path for
real multi-user use (SQL kept portable).

**Server framework — Axum 0.8 + Tokio.** HTTP API for sync and specialist
reads. *Why:* mainstream Rust web framework from the Tokio team, small and
easy to test (tests send requests through the router in memory with `tower`).

**HTTP client — reqwest 0.12 with rustls.** Desktop → server sync. *Why:*
standard Rust client; rustls is TLS written in Rust (no OpenSSL), ready for HTTPS.

**Serialization — serde + serde_json.** Rust structs ↔ JSON for commands, sync
and the knowledge base. *Why:* the standard, derived automatically, no hand
parsing.

**Record ids — UUID v7.** *Why:* created on the device even offline, unique
without a server, and sorted by creation time.

**Content hashes — SHA-256 (sha2).** *Why:* detects duplicates (same id, same
hash) and conflicts (same id, different hash) during sync without comparing
whole records.

**Errors — thiserror.** *Why:* readable error types; the UI shows a clear
message instead of crashing.

**Assistant knowledge base — JSON + keyword retrieval.** 14 source-checked
excerpts embedded at compile time. *Why:* works offline and only repeats
verified sources; no language model, so no invented medical text.

**Testing — `cargo test` (Rust built-in) + Tokio test runtime.** Unit tests,
in-memory database tests, server HTTP tests and demo-scenario tests (93 in
total). **Linting — Clippy**; **type checking — `tsc`**.

**Packaging — NSIS via Tauri bundler.** Per-user Windows installer with
shortcuts and an uninstaller. *Why:* no admin rights needed; standard Windows
experience. Not code-signed in the beta.

**Version control — Git + GitHub (private repo).**

Medical logic is **pure functions** (data in, result out): the same rules run
offline on the astronaut's computer and are easy to test.

## 8. Tests and demo

```bash
cargo test --workspace                                                    # 93 tests
cargo test -p nurbit-medical-core --test demo_scenarios -- --nocapture    # demo metrics
cargo clippy --workspace --all-targets                                    # lint
cd apps/desktop && npm run build                                          # TypeScript check + build
```

The 10-step demo script and measured metrics: [docs/DEMO.md](docs/DEMO.md).

## 9. Limitations
- Thresholds other than NASA limits are prototype assumptions.
- No password; sync token stored unencrypted locally.
- Sync over HTTP on localhost only; specialist notes not yet synced back.
- Assistant: English keywords only, no language model.
- Exercise and dosimeter data are simulated; actigraphy and exam entry not implemented.
- Installer not code-signed.

## 10. Documents
[Specification](docs/PROJECT_SPECIFICATION.md) · [Architecture](docs/ARCHITECTURE.md) ·
[Development plan](docs/DEVELOPMENT_PLAN.md) · [Feasibility](docs/FEASIBILITY.md) ·
[References](docs/REFERENCES.md) · [Demo](docs/DEMO.md) · [AI usage](docs/AI_USAGE.md)
