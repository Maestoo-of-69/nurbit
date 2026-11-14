# References — source register

Only sources that were actually opened and read are listed. "Accessed" is the
date this project read them; "Updated" is the page's own "Page Last Updated".
Findings are paraphrased from the page; anything not on the page is not
claimed here.

| ID | Source | Organization | Updated | URL | Relevant finding | Used for | Limitations |
|----|--------|--------------|---------|-----|------------------|----------|-------------|
| HRP-RISKS | Human Research Program — Risks | NASA HRP | 2026-07-21 | https://www.nasa.gov/hrp/risks/ | Lists 17 risks in three groups: human physiology (11), behavioral performance and human factors (5), space radiation (1); risks are associated with five hazards | Choosing candidate modules | Overview page; no measurement protocols |
| HRP-SLEEP | Risk from Inadequate Sleep and Irregular Schedules | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-from-inadequate-sleep-and-irregular-schedules/ | Sleep loss, circadian desynchronization and work overload can reduce performance and harm health; ISS LED lighting simulates a light-dark cycle | Module 1 rationale | Public summary; names no measurement tools or thresholds |
| HRP-BHV | Risk of Behavioral Changes and Psychiatric Disorders | NASA HRP | 2026-02-24 | https://www.nasa.gov/reference/risk-of-behavioral-conditions-and-psychiatric-disorders/ | Isolation and confinement can increase anxiety/depression risk; may affect sleep, morale, decision making | Module 2 rationale | Names no screening instruments |
| HRP-CV | Risk of Cardiovascular Adaptations | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-of-cardiovascular-adaptations/ | Low gravity can alter heart/vessels: clots, arrhythmias, low blood pressure at landing; ultrasound used to view the heart | Module 3 rationale | No thresholds |
| HRP-SANS | Risk of Spaceflight Associated Neuro-ocular Syndrome (SANS) | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-of-spaceflight-associated-neuro-ocular-syndrome-sans/ | Headward fluid shift → optic nerve swelling, retinal folds, globe flattening; higher risk on longer missions; fundoscope imaging | Module 4 rationale | Exam-based; no self-report protocol given |
| HRP-FIT | Risk of Reduced Cardiorespiratory and Musculoskeletal Fitness | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-of-reduced-cardiorespiratory-and-musculoskeletal-fitness/ | ~1.5 h/day exercise on ISS; periodic crew fitness self-evaluations; average 10–12 % loss of strength/aerobic capacity | Module 5 rationale | Self-evaluation protocol not described |
| HRP-BONE | Risk of Spaceflight-Induced Bone Changes | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-of-spaceflight-induced-bone-changes/ | ~1–1.5 % bone density loss per month on 4–6-month missions; scans before and after flight | Alternative module | Little in-flight measurement |
| HRP-RAD | Risk of Radiation-Induced Cancers | NASA HRP | 2025-05-16 | https://www.nasa.gov/reference/risk-of-radiation-induced-cancers/ | Higher exposure beyond Earth's magnetosphere; ISS instruments measure exposure | Alternative module | Exposure ≠ health outcome |

## In-flight monitoring evidence (Stage 3 research, opened 2026-11-14)

Opened and checked by an AI research assistant (see `AI_USAGE.md`); the developer
should re-read key sources before relying on them. **No source below gives an
in-flight alert threshold for heart rate, blood pressure, reaction time or mood
scores.** Do not hard-code any such value as a "NASA threshold".

| ID | Source | Year | URL / DOI | Supports | Limitations |
|----|--------|------|-----------|----------|-------------|
| SLEEP-RST | Dinges, Mollicone, Ecker — Reaction Self Test (PVT self test on ISS) | 2009 | https://ntrs.nasa.gov/citations/20090029851 | Self-administered 5-min test: 2-min questions + 3-min PVT (reaction time, lapses, false starts); feedback vs astronaut norms; 2×/day every 4th mission day and around EVAs/sleep shifts | Slide deck; quoted work/sleep hour figures are study hypotheses, not alert limits |
| SLEEP-BARGER | Barger, Flynn-Evans et al., *Lancet Neurol* 13(9):904–912 | 2014 | DOI 10.1016/S1474-4422(14)70122-X · https://pmc.ncbi.nlm.nih.gov/articles/PMC4188436/ | Wrist actigraphy (Actiwatch) in flight + daily computerized sleep/medication log; ISS mean sleep ≈6.1 h | Observational |
| SLEEP-ER | Flynn-Evans et al., Evidence Report: sleep loss, circadian desynchronization, work overload | 2016 | https://ntrs.nasa.gov/citations/20160003864 | No easily collected in-flight circadian biomarkers; need for standardized performance metrics | Only the abstract was read |
| BHV-COG | Basner et al., Cognition Test Battery, *Aerosp Med Hum Perform* 86(11):942–952 | 2015 | DOI 10.3357/AMHP.4343.2015 · https://pmc.ncbi.nlm.nih.gov/articles/PMC4691281/ | 10 tests incl. 3-min PVT, ~18 min, laptop/iPad, used on ISS | Normative data still in development at publication |
| BHV-DEV | Dev et al., *Front Physiol* 15:1451269 | 2024 | DOI 10.3389/fphys.2024.1451269 | Cognition in HRP Standard Measures (n=25); self-reported sleep hours and 0–10 alertness before sessions | 2 in-flight time points |
| BHV-APEX | Bell et al., APEX standardized suite (HRP IWS slides) | 2026 | https://ntrs.nasa.gov/api/citations/20260002661/downloads/Bell_APEX_IWS_2026_final.pdf | Behavioral-health measure suites (weekly depression scale, daily mood/behavioral-health items) in analogs and ISS subset | Slides; ISS item subset not specified; no thresholds |
| CV-ECHO | Hamilton, Sargsyan, Martin et al. — on-orbit echocardiography | 2010 | https://ntrs.nasa.gov/citations/20100042124 | Crew-operated ultrasound with ground guidance (60-min sessions) | n=6 |
| CV-HOLTER | Fraser, Greaves, Blaber, Shoemaker, Hughson — IAC-11 abstract | 2011 | https://iafastro.directory/iac/archive/browse/IAC-11/A1/2/9471/ | 24-h Holter heart rate + actigraphy before/during/after flight | Abstract; no blood pressure |
| CV-RHYTHM | Lee, Stenger, Laurie, Macias — Risk of Cardiac Rhythm Problems | 2017 | https://ntrs.nasa.gov/citations/20170005625 | Uncertain whether spaceflight alone raises arrhythmia risk | Abstract only |
| SANS-ER | Stenger, Tarver et al., Evidence Report: Risk of SANS | 2017 | https://ntrs.nasa.gov/citations/20180000936 | In-flight ocular ultrasound, OCT, fundoscopy, visual acuity (logMAR), Amsler grid, vision-symptom questionnaire (mild/moderate/severe); research case definitions (e.g. Frisén grade ≥1, refraction change 0.75 D) | Case definitions are post-flight research criteria, not in-flight alarms; 2022 update not yet read |
| SANS-OCT | Macias, Patel, Gibson et al., *JAMA Ophthalmol* 138(5):553–559 | 2020 | DOI 10.1001/jamaophthalmol.2020.0673 · https://pmc.ncbi.nlm.nih.gov/articles/PMC7118682/ | In-flight OCT around FD10–150; choroid thickening | n=11; operator not described |
| FIT-VO2 | Moore et al., *J Appl Physiol* 117(3):231–238 | 2014 | DOI 10.1152/japplphysiol.01251.2013 | In-flight peak cycle tests ~every 30 days (VO2, heart rate); −17 % at FD15 | Research protocol |
| FIT-PFE | Moore et al., ISS experiment description "VO2max" | 2008 | https://ntrs.nasa.gov/api/citations/20090015252/downloads/20090015252.pdf | CEVIS ergometer with heart rate, blood pressure, workload, perceived effort; names the routine Periodic Fitness Evaluation | Experiment description |
| FIT-ESA | ESA astronaut blog "L+477: Periodic fitness" | 2013 | https://outpost42.esa.int/blog/diario-di-bordo/single/l-477-periodic-fitness/ | PFE ~14 days after arrival, then monthly; ECG, BP cuff | Informal blog |
| TECH-SPEECH | MDN "Using the Web Speech API"; Chrome 139 release notes | 2025 | https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API/Using_the_Web_Speech_API · https://developer.chrome.com/blog/new-in-chrome-139 | Default speech recognition sends audio to a server (no offline); on-device mode is opt-in and experimental | Relevance to Tauri/WebView2 not yet tested |

Questionnaire licences: BDI-II is sold commercially (Pearson, qualification
level B) — not usable without a licence. POMS 2 and CES-D licences **not verified**.

## Five approved risks: additional sources (opened 2026-11-14)

Opened by an AI research assistant; re-read before relying on them.

| ID | Source | Year | URL / DOI | Supports | Limitations |
|----|--------|------|-----------|----------|-------------|
| HRR-SANS | NASA Human Research Roadmap — Risk of SANS | ratings 2022 | https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=105 | Official title; pre/in/post-flight imaging; ~70 % of crew show at least one sign | Page only partly read |
| HRR-BHV | NASA HRR — Risk of Adverse Cognitive or Behavioral Changes and Psychiatric Disorders | — | https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=99 | Official title; related tasks (Reaction Self Test, actigraphy) | Body text could not be read |
| HRR-SLEEP | NASA HRR — Risk of Performance Decrements … Sleep Loss, Circadian Desynchronization, and Work Overload | ratings 2022 | https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=100 | Crews sleep ~6 h; chronic short sleep decreases vigilant attention | Page only partly read |
| HRR-RAD | NASA HRR — Risk of Radiation Carcinogenesis | — | https://humanresearchroadmap.nasa.gov/Risks/risk.aspx?i=96 | Official title; large REID uncertainty | Page only partly read |
| BONE-ER | Sibonga et al., Evidence Report: Risk of Bone Fracture due to Spaceflight-induced Changes to Bone | 2024 | https://ntrs.nasa.gov/citations/20240005190 | DXA misses hip trabecular changes; QCT; countermeasures | Record page read, not full PDF |
| BONE-MTB | NASA OCHMO Medical Technical Brief "Bone Loss" (Rev D) | 2026 | https://www.nasa.gov/wp-content/uploads/2023/12/ochmo-tb-030-bone-loss.pdf | NASA-STD-3001 V1 4026–4028 (≥95 % / ≥90 % of pre-mission BMD); no in-flight BMD; DXA pre/post and triennial; FRAX referral | "For reference only"; text extraction partial |
| BONE-SMITH | Smith SM et al., J Bone Miner Res 27(9):1896–1906 | 2012 | DOI 10.1002/jbmr.1647 | In-flight bone markers (research); ARED preserved hip/pelvis BMD | n = 13; abstract only |
| RAD-SRAG | NASA JSC Space Radiation Analysis Group — "How do we protect the astronauts from space radiation?" | — | https://srag.jsc.nasa.gov/spaceradiation/how/how.cfm | Crew Active Dosimeter, area monitors, dose-rate alarm, console support | No effective-dose method |
| RAD-STD | NASA OCHMO-TB-020 "Design for Ionizing Radiation Protection" (Rev E 2022; Rev G 2025) | 2022/2025 | https://www.nasa.gov/wp-content/uploads/2023/03/radiation-protection-technical-brief-ochmo.pdf | NASA-STD-3001 V1 4030: career effective dose < 600 mSv; V1 4029 ALARA; V2 6100 alerting | Brief quotes the standard; standard PDF not parsed |
| RAD-NASEM | National Academies, "Space Radiation and Astronaut Health" | 2021 | https://www.nationalacademies.org/news/nasa-should-update-astronaut-radiation-exposure-limits-improve-communication-of-cancer-risks | Basis of the 600 mSv limit (3 % REID) | News page |

## Still to research
- NASA Human Research Roadmap pages (not reached; NTRS copies used instead).
- SANS evidence report 2022 update; full sleep evidence report.
- Exercise-log practice and in-flight musculoskeletal monitoring (no source found yet).
