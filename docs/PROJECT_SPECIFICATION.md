# Project Specification — Nurbit (beta)

Status: draft for approval · Last updated: 2026-11-14

## 1. Problem and goal

Astronauts spend crew time on routine medical questionnaires and reports. The
beta demonstrates a system that collects health information systematically and
briefly, compares it with each person's own history, flags changes for a
qualified specialist, works without a link to Earth, and synchronizes safely when
the link returns.

The system **supports** medical specialists. It does not diagnose, prescribe, or
decide fitness for duty.

## 2. Users and roles

| Role | Can do | Cannot do |
|------|--------|-----------|
| `astronaut` | Own check-ins, own profile and history, own alerts, assistant | See other crew members' medical data |
| `specialist` (flight surgeon) | Read all assigned crew records, add notes/annotations, approve baseline changes, record decisions | Delete original measurements (only annotate/correct with history) |
| `specialist` (second reviewer) | Same, plus confirm or reject a duty-restriction recommendation | Approve own recommendation |
| `crew_responder` | Receive an **emergency** notice: who, where, what kind of help — minimal data | Read medical history |
| `admin` (demo only) | Reset demo data, manage synthetic accounts | Read medical content beyond what is needed for admin |

## 3. Functional modules

### A — Daily Crew Health Check
- Three mandatory questions, 0–10 scale, same wording every day, with anchor text
  at both ends and the scale direction shown on screen.
- Proposed wording (pending specialist review; see §5):

  | # | Question | 0 means | 10 means | Why it is asked |
  |---|----------|---------|----------|-----------------|
  | Q1 | "How rested do you feel after your last sleep period?" | Not rested at all | Fully rested | Sleep loss and schedule disruption are an HRP risk; a daily subjective signal is cheap to collect and comparable over time. |
  | Q2 | "How is your body today — pain, discomfort, illness or other physical symptoms?" | Worst I can imagine | No symptoms at all | A single physical screen that opens body-system follow-ups (pain location, vision, dizziness, etc.). |
  | Q3 | "How are your mood and mental energy today?" | Very low | Very good | Behavioral health is an HRP risk; this item opens a short mood/stress/concentration follow-up. |

- All three use **10 = best** so the direction never flips between items.
  Trade-off: common clinical pain scales use 0 = no pain. Final direction is a
  specialist decision (open decision D-5).
- A separate, always-visible **"Report a symptom now"** entry exists, so an
  urgent problem is never forced into the daily check.
- Target: ~1–3 min for the routine check; extended assessment ≤ ~10 min.
- Every answer is stored with timestamp, author, device, and source.

### B — Adaptive questionnaires
- Follow-up questions are chosen by **explicit, readable rules** (no hidden
  model), using: the daily answers, deviation from the person's baseline, recent
  symptoms, open findings, profile conditions, measurements, and mission context.
- Rules cannot remove mandatory questions or safety ("red-flag") questions.
- Every triggered question records **why** it was asked (rule id), so test
  metrics can count triggers and reviewers can audit them.
- Trigger thresholds are **prototype assumptions** and labeled as such until a
  source or specialist validates them.

### C — Individual medical profile
- History, conditions, injuries, restrictions, medications, allergies, previous
  assessments, baseline values, measurement history (synthetic only).
- Five kinds of comparison are kept separate in data and UI:
  reference range · individual baseline · temporary deviation · persistent change ·
  clinically concerning finding (the last only when a specialist marks it).
- The system **proposes** a baseline change after a persistent change; only a
  specialist can approve it. The old baseline is kept (versioned).

### D — Five risk modules (configurable)
One generic risk-module framework; each risk is a configuration file plus a few
pure functions. The five approved risks (bone, SANS, behavioral health, sleep, radiation) are
listed in `DEVELOPMENT_PLAN.md` §3; feasibility per risk is in `FEASIBILITY.md`.

Each module defines: id and HRP source, indicators, measurement sources
(manual / device / lab / questionnaire), frequency, baseline comparison method,
warning conditions, follow-up triggers, explanations, references, next steps
(only from cited protocols, otherwise "refer to specialist"), and review status.

### E — Cross-risk analysis
- Looks at indicator pairs within one person over time (e.g. sleep rating vs.
  mood rating vs. reaction-time test).
- Output is labeled with exactly one evidence level:
  `observed_correlation` · `possible_association` · `established_relationship`
  (requires a cited source) · `unverified_hypothesis`.
- Never phrased as cause. Small samples are reported as insufficient.

### F — Medical assistant
- Beta version is a **retrieval assistant**: it searches a small local
  knowledge base of verified excerpts and shows them with source, title,
  version/date, section, and why it matches. This part is real, not simulated.
- Symptom intake: body area, severity, characteristics, adaptive follow-ups
  (same rule engine as Module B).
- "Possible states" are shown only when a knowledge-base entry supports them,
  always with uncertainty text and "not a diagnosis".
- No medication choices or dosages are generated.
- A language-model provider is an interface (`AssistantProvider`) with clearly
  labeled implementations: `retrieval-only` (beta), `local-llm` (future),
  `cloud` (future, online and authorized only). The UI always shows which one
  answered. When offline, the cloud option is shown as unavailable.
- Voice input: optional. Browser speech recognition may send audio to a cloud
  service, so it is off by default and marked "online only" (to verify).

### G — Offline / online
- The astronaut app works fully offline for: profile, check-ins, history, local
  risk evaluation, downloaded knowledge base, retrieval assistant.
- Sync when online: unique ids, timestamps, status per record, idempotent
  upload (no duplicates), conflict detection, resumable partial sync. Records
  are never silently overwritten.

### H — Specialist dashboard
Crew overview, individual records, check-in history, measurement charts, alerts,
cross-risk patterns, assistant outputs (labeled AI-generated), sources, notes,
annotations/corrections, baseline approvals, decisions. Duty-restriction
recommendations require a second specialist. The software never declares anyone
unfit.

### I — Notifications and escalation
- Normal results: no notification.
- Relevant deviation: astronaut notice with the reason.
- Potentially serious: high-priority alert with reason and frozen copies of the
  supporting data; queued to the specialist until a link exists.
- Emergency crew notice: configurable, minimal data, separate flow from the
  questionnaire, logged.

## 4. UX requirements
Minimal, fast, repeatable. Home screen: monitoring status, today's check status,
notifications, history, risk overview, assistant entry. Per-domain indicators and
trends; **no universal 0–100 health score**.

## 5. Scientific responsibility rules
- No invented thresholds, norms, protocols, sensor capabilities, or citations.
- Every threshold in config has a `basis` field: `source:<ref-id>` or
  `prototype_assumption` (shown in the UI as "needs specialist validation").
- AI output and specialist decisions are stored in different tables and shown
  differently.

## 6. Non-goals for the beta
Real medical data; real sensor integration (simulated device feeds are labeled);
clinical validation; regulatory compliance claims; a trained medical model;
multi-mission or multi-vehicle scale.

## 7. Evaluation (synthetic scenario)
The 10-step demo scenario from the brief, with metrics: check-in duration,
questions asked, follow-ups triggered, completeness, sync success/duplicates/
conflicts, specialist review time, unnecessary alerts. Results describe the
prototype only and do not show clinical effectiveness.
