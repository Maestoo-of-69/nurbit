// Assistant: symptom intake -> retrieval over a verified local knowledge base.
// No language model in the beta; every answer shows its sources, or says
// that no reliable source was found. Never a diagnosis, never a dose.
import { useState } from "react";
import { askAssistant, assistantProviders, type AssistantAnswer, type CrewMember } from "../api";
import { ScaleInput } from "../components/AnswerInputs";
import { ScreenHeader } from "../components/ScreenHeader";
import { useCommand } from "../useCommand";

const AREAS: [string, string][] = [
  ["eyes", "Eyes / vision"],
  ["head", "Head"],
  ["back", "Back"],
  ["limbs", "Arms / legs"],
  ["mind", "Mood / thinking"],
  ["general", "General / sleep"],
];
const ONSETS: [string, string][] = [
  ["sudden", "Sudden"],
  ["gradual", "Gradual"],
  ["unsure", "Not sure"],
];
const MODULE_NAMES: Record<string, string> = {
  bone: "Bone changes",
  sans: "Neuro-ocular (SANS)",
  "behavioral-health": "Behavioral health",
  "sleep-schedules": "Sleep and schedules",
  radiation: "Radiation exposure",
};
const MISSING_LABEL = { body_area: "body area", severity: "severity", onset: "when it started" };

function Chips({ options, value, onChange, label }: { options: [string, string][]; value: string | null; onChange: (v: string) => void; label: string }) {
  return (
    <div className="options" role="radiogroup" aria-label={label}>
      {options.map(([id, text]) => (
        <button key={id} type="button" role="radio" aria-checked={value === id} className="option" onClick={() => onChange(id)}>
          {text}
        </button>
      ))}
    </div>
  );
}

export function AssistantScreen({ member }: { member: CrewMember }) {
  const providers = useCommand(assistantProviders);
  const [text, setText] = useState("");
  const [area, setArea] = useState<string | null>(null);
  const [severity, setSeverity] = useState<number | null>(null);
  const [onset, setOnset] = useState<string | null>(null);
  const [result, setResult] = useState<AssistantAnswer | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);

  async function ask() {
    setIsBusy(true);
    setError(null);
    try {
      setResult(await askAssistant(member.id, { text, body_area: area, severity, onset }));
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  }

  return (
    <>
      <ScreenHeader title="Assistant" subtitle="Describe a symptom. Answers come only from verified sources stored on this computer." />
      <div className="callout">
        Not a diagnosis and not medical advice. The assistant never suggests medication or doses. For anything
        urgent, contact the crew medical officer.
      </div>

      <section className="panel panel-wide human-panel assistant-form">
        <label className="panel-label" htmlFor="assistant-text">What is happening?</label>
        <textarea
          id="assistant-text"
          className="text-input"
          placeholder="e.g. My vision is blurred when I read"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        <div className="panel-label">Body area</div>
        <Chips options={AREAS} value={area} onChange={setArea} label="Body area" />
        <div className="panel-label">How bad is it?</div>
        <ScaleInput label="Severity" value={severity} onChange={setSeverity} lowAnchor="None" highAnchor="Worst imaginable" />
        <div className="panel-label">How did it start?</div>
        <Chips options={ONSETS} value={onset} onChange={setOnset} label="Onset" />
        <div className="button-row">
          <button type="button" className="button-primary" disabled={isBusy || !text.trim()} onClick={ask}>
            Search verified sources
          </button>
          <button type="button" className="button-secondary" disabled title="Not available in this beta">
            Voice input
          </button>
        </div>
        <p className="cell-note">
          Voice input is off: browser speech recognition sends audio to an online service by default, which would break
          offline use [TECH-SPEECH].
        </p>
        {error && <p className="error">{error}</p>}
      </section>

      {result && <AnswerView result={result} />}

      <section className="panel panel-wide">
        <div className="panel-label">Assistant providers</div>
        <ul className="finding-list">
          {providers.data?.map((p) => (
            <li key={p.id}>
              <span className={`badge ${p.state === "active" ? "badge-normal" : "badge-inactive"}`}>
                {p.state === "active" ? "Active" : p.state === "not_installed" ? "Not installed" : "Unavailable"}
              </span>
              <span>
                <strong>{p.label}</strong> — {p.note}
              </span>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}

function AnswerView({ result }: { result: AssistantAnswer }) {
  return (
    <>
      {result.urgent_message && (
        <div className="alert-banner" role="alert">
          {result.urgent_message}
        </div>
      )}
      {result.medication_notice && <div className="callout">{result.medication_notice}</div>}
      {result.missing_info.length > 0 && (
        <p className="cell-note">
          More detail helps the search: {result.missing_info.map((m) => MISSING_LABEL[m]).join(", ")}.
        </p>
      )}

      {result.no_reliable_source ? (
        <section className="panel panel-wide">
          <div className="panel-label">No reliable source found</div>
          <p className="panel-text">
            The local library has no verified source that matches this question. Please describe it to the flight
            surgeon; the assistant will not guess.
          </p>
        </section>
      ) : (
        <>
          <h2 className="section-title">
            What the sources say <span className="section-tag">verified-source search · no language model</span>
          </h2>
          {result.matches.map((m) => (
            <article key={m.entry.id} className="panel panel-wide source-card">
              <h3 className="source-title">{m.entry.title}</h3>
              <p>{m.entry.summary}</p>
              <dl className="risk-facts">
                <dt>Source</dt>
                <dd>{m.entry.source_name}</dd>
                <dt>Document</dt>
                <dd>{m.entry.document}</dd>
                <dt>Date</dt>
                <dd>{m.entry.published}</dd>
                <dt>Section</dt>
                <dd>{m.entry.section}</dd>
                <dt>Why shown</dt>
                <dd>Matches: {m.matched_terms.join(", ")}</dd>
                <dt>Reference</dt>
                <dd className="mono">{m.entry.source_id}</dd>
              </dl>
            </article>
          ))}
          {result.related_modules.length > 0 && (
            <p className="panel-text">
              These sources belong to the monitoring of:{" "}
              <strong>{result.related_modules.map((id) => MODULE_NAMES[id] ?? id).join(", ")}</strong>. This is monitoring
              context, not a diagnosis.
            </p>
          )}
          <p className="cell-note">{result.uncertainty}</p>
        </>
      )}
    </>
  );
}
