# Feasibility analysis — five approved risks

Status: research summary, 2026-11-14. Scope approved by the developer:
bone, SANS, behavioral health, sleep, radiation. Source IDs → `REFERENCES.md`.

**Monitoring is not diagnosis.** Nurbit collects, organizes and forwards
information. It does not detect cancer, measure bone density in flight,
replace an eye exam, or diagnose a psychiatric disorder.

Indicator classes: **M** directly measured · **S** self-reported ·
**D** derived · **R** risk estimate · **F** finding requiring specialist assessment.

## Summary table

| | Bone | SANS | Behavioral | Sleep | Radiation |
|-|------|------|------------|-------|-----------|
| Official NASA title | Risk of Bone Fracture due to Spaceflight-induced Changes to Bone | Risk of Spaceflight Associated Neuro-ocular Syndrome (SANS) | Risk of Adverse Cognitive or Behavioral Changes and Psychiatric Disorders | Risk of Performance Decrements … Sleep Loss, Circadian Desynchronization, and Work Overload | Risk of Radiation Carcinogenesis |
| Key in-flight data | exercise device logs, pain/fall reports | vision symptoms, acuity, Amsler, OCT/fundus/ultrasound | mood ratings, cognitive self-tests | rested rating, sleep log, actigraphy, PVT | personal dosimeter, area monitors |
| Main measurement in flight? | **No** — DXA only before/after flight | Yes (imaging in flight) | Yes (self-tests) | Yes | Yes (automatic) |
| Daily questionnaire useful? | Partly (pain, falls) | Partly (symptoms) | Yes | Yes | **No** |
| Own-baseline comparison | post-flight DXA vs pre-flight | imaging vs pre-flight | ratings, test scores | sleep, PVT | not applicable (cumulative) |
| Official limit found | ≥95 % hip/spine, ≥90 % femoral neck of pre-mission BMD (post-flight) | none | none | none | career effective dose < 600 mSv |
| Questionnaire trigger thresholds in sources | none | none | none | none | none |
| Beta data | simulated exercise log, follow-up questions | follow-up questions, manual exam entry | daily + follow-up questions | daily + follow-up questions, simulated actigraphy | simulated dosimeter |

## 1. Bone changes

1. **Monitored:** countermeasure adherence (exercise), bone-related symptoms;
   bone density only from ground scans. [BONE-MTB, BONE-ER]
2. **Indicators:** DXA BMD hip/spine (M→D change from pre-flight); QCT (M,
   research); FRAX (R); vitamin D, bone turnover markers (M, F); resistive
   exercise load and sessions (M); pain, falls (S).
3. **Collection:** DXA before and after flight and every 3 years; exercise
   devices (ARED, T2) record sessions and downlink them. [BONE-MTB]
4. **Type:** DXA/QCT medical tests; exercise data automatic; markers laboratory;
   symptoms self-reported.
5. **Frequency:** DXA pre/post flight; exercise daily; markers in-flight only in
   research (operational schedule not verified). [BONE-SMITH]
6. **Equipment:** DXA/QCT (ground), ARED/T2, lab assays.
7. **Self-collected:** exercise sessions, pain and fall reports, diet.
8. **Baseline:** post-flight BMD vs pre-flight (official goal above). In flight,
   only exercise adherence vs the person's own plan.
9. **Triggers:** FRAX and Z-score referral rules exist but apply to scans, not
   to questionnaires [BONE-MTB]. Pain/fall/missed-exercise triggers → *needs
   specialist validation*.
10. **To specialists:** exercise history, pain/fall reports, scan results.
11. **Sources:** HRP-BONE, BONE-ER, BONE-MTB, BONE-SMITH.
12. **Limits:** no in-flight bone density; exercise ≠ bone health; in-mission
    fracture risk cannot be estimated with confidence.

## 2. SANS

1. **Monitored:** vision symptoms and eye/optic-nerve structure. [HRR-SANS]
2. **Indicators:** vision/headache questionnaire (S); visual acuity, Amsler grid
   (M/S); OCT, fundoscopy, ocular ultrasound (F).
3. **Collection:** in-flight imaging guided from the ground; acuity on laptop;
   Amsler grid each session. [SANS-ER, SANS-OCT]
4. **Type:** symptoms self-reported; imaging medical tests.
5. **Frequency:** ultrasound FD30/FD90; OCT roughly monthly in research; symptoms any time.
6. **Equipment:** OCT, fundoscope, ultrasound; MRI on the ground only.
7. **Self-collected:** symptoms, acuity chart, Amsler grid.
8. **Baseline:** all imaging compared with the person's pre-flight values.
9. **Triggers:** reported vision change → specialist review; thresholds *need
   specialist validation*. Post-flight research case definitions (e.g. Frisén
   grade ≥ 1) are not in-flight alarms.
10. **To specialists:** symptom history, acuity/Amsler, imaging results.
11. **Sources:** HRP-SANS, HRR-SANS, SANS-ER, SANS-OCT.
12. **Limits:** questionnaire ≠ eye exam; mechanism and long-term outcome unknown.

## 3. Behavioral health

1. **Monitored:** mood, stress, concentration, cognitive performance.
2. **Indicators:** mood/energy rating (S); stress, concentration (S); cognitive
   battery and PVT (M); contact requests (S).
3. **Collection:** self-administered on a laptop. [BHV-COG, BHV-DEV]
4. **Type:** self-report and self-tests.
5. **Frequency:** daily ratings; cognitive tests periodic (protocol-defined).
7. **Self-collected:** all of the above.
8. **Baseline:** test scores and ratings vs the person's own earlier sessions.
9. **Triggers:** none in sources; the prototype uses low score / drop from own
   average → *needs specialist validation*.
10. **To specialists:** mood/stress trends, contact requests, test results.
11. **Sources:** HRP-BHV, HRR-BHV, BHV-COG, BHV-DEV, BHV-APEX.
12. **Limits:** ratings cannot diagnose; validated scales (e.g. BDI-II) are licensed.

## 4. Sleep and schedules

1. **Monitored:** sleep duration/quality, alertness, vigilant attention.
2. **Indicators:** rested rating, sleep log (S); actigraphy (M); PVT reaction
   time and lapses (M); hypnotic use (S).
3. **Collection:** wrist actigraph; daily computerized log; PVT self-test. [SLEEP-BARGER, SLEEP-RST]
4. **Type:** automatic (actigraphy) and self-collected.
5. **Frequency:** actigraphy continuous; log daily; PVT repeated periodically.
7. **Self-collected:** log, ratings, PVT.
8. **Baseline:** sleep duration and PVT vs the person's own values.
9. **Triggers:** none in sources → *needs specialist validation*.
10. **To specialists:** sleep and rested trends, interruptions, alertness.
11. **Sources:** HRP-SLEEP, HRR-SLEEP, SLEEP-RST, SLEEP-BARGER, SLEEP-ER.
12. **Limits:** self-report less precise; actigraphy used mainly as a research tool in flight.

## 5. Radiation

1. **Monitored:** radiation dose and its running total. Not cancer.
2. **Indicators:** personal absorbed dose and rate (M); cumulative effective
   dose (D, computed by specialists); REID (R); space-weather alerts (M).
3. **Collection:** personal active dosimeter (data via laptop, downlinked);
   area monitors; dose-rate alarm for sheltering. [RAD-SRAG]
4. **Type:** automatic; effective dose and REID calculated by specialists.
5. **Frequency:** continuous.
6. **Equipment:** personal and area dosimeters, space-weather service.
7. **Self-collected:** wearing the dosimeter; shelter and EVA log.
8. **Baseline:** not applicable; cumulative total vs career limit.
9. **Limit:** career effective dose < 600 mSv [RAD-STD]. Questionnaire triggers:
   none — dose does not produce symptoms a questionnaire could detect.
10. **To specialists:** dose series, shelter/EVA log, alerts.
11. **Sources:** HRP-RAD, HRR-RAD, RAD-SRAG, RAD-STD, RAD-NASEM.
12. **Limits:** dose tracking estimates risk, it cannot detect cancer; no
    epidemiological data for space radiation; large uncertainty.

## Cross-risk links (shown only with these labels)

| Link | Evidence level | Source |
|------|----------------|--------|
| Sleep loss → reduced vigilant attention | established (mainly ground studies) | HRR-SLEEP |
| Sleep loss → mood, psychosocial effects | established on the ground; association in flight not verified | HRR-SLEEP |
| Resistive exercise → preserved hip/pelvis BMD | association in a small flight study (n = 13) | BONE-SMITH |
| Radiation dose → long-term cancer risk | modelled, large uncertainty | HRR-RAD, RAD-NASEM |
| Radiation → CNS / degenerative effects | hypothesis / research | RAD-STD |

Within one person's data, Nurbit may show **observed correlations** (e.g. sleep
rating and mood rating moving together). They are labelled as observations, never as causes.

## What the beta can demonstrate honestly
- Daily and follow-up questions for sleep, behavioral health, body symptoms
  (bone: pain/falls; SANS: vision).
- Manual entry of exam results (SANS imaging, DXA) by staff — planned.
- Simulated device feeds, clearly labelled: exercise log, actigraphy, dosimeter.
- Running radiation dose vs the 600 mSv limit, explicitly not a cancer indicator.
