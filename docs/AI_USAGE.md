# AI Usage

## 1. AI used to build the project
- Tool: Claude Code (Anthropic), used as a coding and research assistant.
- Used for: reading official NASA pages and summarizing them, drafting the
  planning documents, and (later) proposing code in small stages.
- Controls:
  - Every source in `REFERENCES.md` was opened and read; nothing is cited from
    memory.
  - AI-written code is treated as unverified until tests pass and the developer
    has reviewed and can explain it.
  - Medical thresholds and protocols are never taken from AI output; they need a
    cited source or are marked `prototype_assumption`.
- The developer remains responsible for every decision and every line submitted.

## 2. AI inside the product

Implemented in stage 11 (`crates/medical-core/src/assistant.rs`, `knowledge-base/entries.json`):
- 14 paraphrased excerpts, each tied to a source row in `REFERENCES.md` (a test enforces this).
- Retrieval is plain keyword overlap plus body-area match; no language model is used.
- Red-flag words or severity >= 8 show an urgent "contact the medical officer" message (prototype rule).
- Questions about medication or doses get a refusal notice; the knowledge base contains no doses.
- Every question is logged (automated output) and visible to specialists.

- Beta assistant = **retrieval over a local, verified knowledge base**. It
  returns excerpts with their sources; it does not generate medical facts.
- Any language model (local or cloud) is a future, separately labeled provider.
  The UI always shows which provider produced an answer; offline, the cloud
  provider is shown as unavailable.
- Assistant output is stored separately from specialist decisions and is never
  treated as a diagnosis, prescription, or fitness-for-duty decision.
