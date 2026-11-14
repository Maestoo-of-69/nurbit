# Nurbit demo script and beta metrics

Synthetic crew only. Nothing here demonstrates clinical effectiveness; it shows
that the prototype behaves as designed.

## Install / run
- Installer: `target/release/bundle/nsis/Nurbit_0.1.0_x64-setup.exe` (≈4 MB, built
  with `npm run tauri build` in `apps/desktop`). Not code-signed: Windows
  SmartScreen / Smart App Control may warn on other PCs.
- Fresh demo data: close the app and delete
  `%APPDATA%\com.nurbit.desktop\nurbit.db`; the synthetic crew is re-created on start.
- Optional server: set `NURBIT_SYNC_TOKEN` (16+ characters) and run
  `cargo run -p nurbit-server`; enter the same token on the Sync screen.

## The 10 steps from the specification

| # | Step | Where in Nurbit | Expected result |
|---|------|-----------------|-----------------|
| 1 | Routine daily check | Astronaut (e.g. Vega) → Daily check, answer 8/8/8 | Saved after 3 questions, no follow-ups, no alerts |
| 2 | Answers are stored | History | New row "Self-report" on top of the synthetic history |
| 3 | Compared with own records | History → trend charts | Value shown against the grey "usual range" band |
| 4 | Concerning answer → more questions | Daily check with Body = 2 | 4 body follow-ups incl. the red "Safety question"; the reason is shown |
| 5 | Risk module reacts | Risk modules / Overview | e.g. vision change → SANS "Specialist review" |
| 6 | Assistant explains with sources | Assistant: "My vision is blurred when I read" | Source cards (organization, document, date, section, why shown), "not a diagnosis" |
| 7 | Stored offline | Sync screen, server stopped → Sync now | "Server unreachable… stay queued", Waiting to send = 1 |
| 8 | Connectivity returns | Start the server | Background retry within 30 s |
| 9 | Pending data synchronized | Sync screen | Waiting 0, Sent 1; re-sync = "nothing to send" |
| 10 | Specialist reviews and decides | Dr. Sirius → Crew → record → Notes & decisions | Duty restriction "Awaiting second specialist"; Dr. Rigel confirms in Pending reviews |

## Measured in the beta

Automated scenario test: `cargo test -p nurbit-medical-core --test demo_scenarios -- --nocapture`.

| Scenario (synthetic) | Questions asked | Follow-ups | Modules at attention/review | Flags |
|----------------------|-----------------|------------|-----------------------------|-------|
| Routine day (8/8/7, steady history) | 3 | 0 | 0 | none |
| Sleep drop (rested 4 vs usual 8) | 6 | 3 | 1 (sleep) | none |
| Vision symptom (body 3, vision change) | 7 | 4 | 1 (SANS) | vision change |
| Red-flag symptom (body 1, chest pain) | 7 | 4 | 0 modules + 1 safety finding | urgent |

Other measurements:
- **Unnecessary alerts:** 0 on the routine day; in each concerning scenario only
  the relevant module reacted.
- **Sync reliability:** server tests cover accepted / duplicate / conflict /
  invalid / wrong token; in the app, offline → online delivered the queued record
  once (no duplicate on re-sync).
- **Check-in time:** not measured with real users. The automated UI run took
  2:29 because of scripted clicking; a real timing needs test participants.
- **Specialist review time:** not measured (needs real reviewers).

## Found by the demo test and fixed
A red-flag answer belonged to no risk module, so specialists would not have seen
it in the crew overview. A separate **safety finding** now reaches the crew
overview, the record (red banner) and the overall status.

## Known limitations
- All thresholds except the NASA limits (600 mSv; ≥95/90 % bone density) are
  prototype assumptions needing specialist validation.
- No password: the profile is chosen at start-up. Sync token stored unencrypted.
- Plain HTTP sync for localhost only; HTTPS needed for real use.
- Assistant: English keyword search over 14 excerpts; no language model.
- Specialist notes and decisions are not yet synchronized back from the server.
