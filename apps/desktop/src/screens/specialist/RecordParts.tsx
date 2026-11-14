// Tabs of the specialist's medical record. Automated output and human
// decisions are visually separate; every write goes through a Rust command
// that checks the specialist role and stores a new record (nothing edited).
import { useState } from "react";
import {
  addNote,
  annotateAnswer,
  dailyScore,
  getRiskModules,
  recordDecision,
  reviewBaseline,
  type AnswerValue,
  type BaselineDecision,
  type CheckInRecord,
  type CrewMember,
  type DecisionKind,
  type DecisionStatus,
  type IndicatorSeries,
  type MemberRecord,
} from "../../api";
import { AssessmentPanel } from "../../components/AssessmentPanel";
import { BaselineBadge } from "../../components/LevelBadge";
import { StatusBadge, type Status } from "../../components/StatusBadge";
import { TrendChart } from "../../components/TrendChart";
import { useCommand } from "../../useCommand";

type PartProps = { actor: CrewMember; record: MemberRecord; onChanged: () => void };

const fmt = (ms: number) =>
  new Date(ms).toLocaleString("en-GB", { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" });

/** Runs one write action with a busy flag and a readable error. */
function useAction() {
  const [isBusy, setIsBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function run(action: () => Promise<void>, onDone: () => void) {
    setIsBusy(true);
    setError(null);
    try {
      await action();
      onDone();
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  }
  return { isBusy, error, run };
}

// ---------------- Summary ----------------

export function RecordSummary({ actor, record, onChanged }: PartProps) {
  const catalogue = useCommand(getRiskModules);
  const title = (id: string) => catalogue.data?.find((m) => m.id === id)?.short_title ?? id;

  return (
    <>
      {record.safety.map((f) => (
        <div key={f.message} className="alert-banner" role="alert">
          {f.message}
        </div>
      ))}
      <h2 className="section-title">Automated assessment <span className="section-tag">rule-based prototype · not a diagnosis</span></h2>
      {record.assessments.map((a) => (
        <AssessmentPanel key={a.module_id} assessment={a} title={title(a.module_id)} />
      ))}
      <h2 className="section-title">Trends and baselines</h2>
      {record.trends.map((s) => (
        <TrendWithReview key={s.question_id} actor={actor} record={record} series={s} onChanged={onChanged} />
      ))}
      <h2 className="section-title">
        Assistant questions <span className="section-tag">automated · verified-source search, not a diagnosis</span>
      </h2>
      <section className="panel panel-wide">
        {record.assistant_log.length === 0 ? (
          <p className="panel-text">No questions asked.</p>
        ) : (
          <ul className="finding-list">
            {record.assistant_log.map((q) => (
              <li key={q.id}>
                <StatusBadge status={q.urgent ? "alert" : "inactive"} label={q.urgent ? "Urgent rule shown" : "Info"} />
                <span>
                  “{q.question}”
                  <span className="cell-note">
                    {fmt(q.created_at)}
                    {q.body_area && ` · area ${q.body_area}`}
                    {q.severity !== null && ` · severity ${q.severity}/10`} · sources shown:{" "}
                    {q.matched_entries.length > 0 ? q.matched_entries.join(", ") : "none (no reliable source)"}
                  </span>
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </>
  );
}

function TrendWithReview({ actor, record, series: s, onChanged }: PartProps & { series: IndicatorSeries }) {
  const [note, setNote] = useState("");
  const action = useAction();
  const reviews = record.baseline_reviews.filter((b) => b.indicator === s.question_id);
  const decide = (decision: BaselineDecision) =>
    action.run(() => reviewBaseline(actor.id, record.member.id, s.question_id, decision, note), () => {
      setNote("");
      onChanged();
    });

  return (
    <section className="panel panel-wide trend-card">
      <div className="trend-head">
        <h2>{s.label}</h2>
        <BaselineBadge status={s.trend.status} />
      </div>
      {s.baseline_since && (
        <p className="cell-note">New baseline approved on {fmt(s.baseline_since)}; only values since then form it.</p>
      )}
      <TrendChart label={s.label} points={s.points} baseline={s.trend.baseline} />
      {s.trend.status.state === "persistent_change" && (
        <div className="review-box">
          <strong>Baseline review proposed by the system.</strong> The baseline is never changed automatically.
          <textarea
            className="text-input"
            placeholder="Reason for your decision (required)"
            value={note}
            onChange={(e) => setNote(e.target.value)}
          />
          <div className="button-row">
            <button type="button" className="button-primary" disabled={action.isBusy} onClick={() => decide("approve_new_baseline")}>
              Approve new baseline from now
            </button>
            <button type="button" className="button-secondary" disabled={action.isBusy} onClick={() => decide("keep_current_baseline")}>
              Keep current baseline
            </button>
          </div>
          {action.error && <p className="error">{action.error}</p>}
        </div>
      )}
      {reviews.map((b) => (
        <p key={b.id} className="cell-note">
          {fmt(b.decided_at)} · {record.names[b.decided_by]}:{" "}
          {b.decision === "approve_new_baseline" ? "approved a new baseline" : "kept the current baseline"} — {b.note}
        </p>
      ))}
    </section>
  );
}

// ---------------- Check-ins ----------------

function valueText(v: AnswerValue) {
  switch (v.kind) {
    case "scale":
      return `${v.value} / 10`;
    case "yes_no":
      return v.value ? "Yes" : "No";
    case "number":
      return String(v.value);
    case "choice":
      return v.value;
  }
}

export function RecordCheckins({ actor, record, onChanged }: PartProps) {
  const [openId, setOpenId] = useState<string | null>(null);
  return (
    <section className="panel panel-wide table-panel">
      <table className="history-table">
        <thead>
          <tr>
            <th>Date</th>
            <th>Rested</th>
            <th>Body</th>
            <th>Mood</th>
            <th>Follow-ups</th>
            <th>Source</th>
            <th />
          </tr>
        </thead>
        <tbody>
          {record.checkins.map((c) => (
            <CheckinRows
              key={c.id}
              checkin={c}
              isOpen={openId === c.id}
              onToggle={() => setOpenId(openId === c.id ? null : c.id)}
              actor={actor}
              record={record}
              onChanged={onChanged}
            />
          ))}
        </tbody>
      </table>
    </section>
  );
}

function CheckinRows({
  checkin: c,
  isOpen,
  onToggle,
  actor,
  record,
  onChanged,
}: PartProps & { checkin: CheckInRecord; isOpen: boolean; onToggle: () => void }) {
  const notes = record.annotations.filter((a) => a.checkin_id === c.id);
  return (
    <>
      <tr>
        <td>{fmt(c.completed_at)}</td>
        <td>{dailyScore(c, "daily.rested") ?? "—"}</td>
        <td>{dailyScore(c, "daily.physical") ?? "—"}</td>
        <td>{dailyScore(c, "daily.mood_energy") ?? "—"}</td>
        <td>{c.answers.filter((a) => a.triggered_by_rule).length || "—"}</td>
        <td>
          <span className={`origin origin-${c.origin}`}>{c.origin === "simulated" ? "Synthetic" : "Self-report"}</span>
          {notes.length > 0 && <div className="cell-note">{notes.length} annotation(s)</div>}
        </td>
        <td>
          <button type="button" className="button-link" onClick={onToggle}>
            {isOpen ? "Close" : "Details"}
          </button>
        </td>
      </tr>
      {isOpen && (
        <tr className="detail-row">
          <td colSpan={7}>
            <AnswerDetails checkin={c} actor={actor} record={record} onChanged={onChanged} />
          </td>
        </tr>
      )}
    </>
  );
}

function AnswerDetails({ checkin: c, actor, record, onChanged }: PartProps & { checkin: CheckInRecord }) {
  const [questionId, setQuestionId] = useState(c.answers[0]?.question_id ?? "");
  const [corrected, setCorrected] = useState("");
  const [reason, setReason] = useState("");
  const action = useAction();
  const notes = record.annotations.filter((a) => a.checkin_id === c.id);

  return (
    <div className="answer-details">
      <ul className="plain-list">
        {c.answers.map((a) => (
          <li key={a.question_id}>
            <span className="mono">{a.question_id}</span>: {valueText(a.value)}
            {a.triggered_by_rule && <span className="cell-note"> (asked because of rule {a.triggered_by_rule})</span>}
            {notes
              .filter((n) => n.question_id === a.question_id)
              .map((n) => (
                <div key={n.id} className="annotation">
                  Annotation by {record.names[n.author_id]} · {fmt(n.created_at)}:
                  {n.corrected_value && <> corrected value “{n.corrected_value}”.</>} {n.reason}
                </div>
              ))}
          </li>
        ))}
      </ul>
      <div className="annotate-form">
        <strong>Add annotation</strong> — the original answer stays unchanged.
        <div className="button-row">
          <select className="text-input" value={questionId} onChange={(e) => setQuestionId(e.target.value)} aria-label="Answer">
            {c.answers.map((a) => (
              <option key={a.question_id} value={a.question_id}>
                {a.question_id}
              </option>
            ))}
          </select>
          <input
            className="text-input"
            placeholder="Corrected value (optional)"
            value={corrected}
            onChange={(e) => setCorrected(e.target.value)}
          />
        </div>
        <textarea className="text-input" placeholder="Reason (required)" value={reason} onChange={(e) => setReason(e.target.value)} />
        <button
          type="button"
          className="button-primary"
          disabled={action.isBusy}
          onClick={() =>
            action.run(() => annotateAnswer(actor.id, c.id, questionId, corrected, reason), () => {
              setCorrected("");
              setReason("");
              onChanged();
            })
          }
        >
          Save annotation
        </button>
        {action.error && <p className="error">{action.error}</p>}
      </div>
    </div>
  );
}

// ---------------- Notes & decisions ----------------

const KIND_LABEL: Record<DecisionKind, string> = {
  follow_up_exam: "Follow-up examination",
  clear_for_duty: "Clear for duty",
  duty_restriction: "Recommend removal from duty (needs 2nd specialist)",
  other: "Other",
};

const STATUS: Record<DecisionStatus, { status: Status; label: string }> = {
  recorded: { status: "normal", label: "Recorded" },
  pending_second_review: { status: "attention", label: "Awaiting second specialist" },
  confirmed: { status: "review", label: "Confirmed by second specialist" },
  rejected: { status: "inactive", label: "Rejected by second specialist" },
};

export function RecordNotes({ actor, record, onChanged }: PartProps) {
  const [note, setNote] = useState("");
  const [kind, setKind] = useState<DecisionKind>("follow_up_exam");
  const [text, setText] = useState("");
  const noteAction = useAction();
  const decisionAction = useAction();

  return (
    <>
      <h2 className="section-title">Decisions <span className="section-tag human">specialist judgment</span></h2>
      <section className="panel panel-wide human-panel">
        <div className="options" role="radiogroup" aria-label="Decision type">
          {(Object.keys(KIND_LABEL) as DecisionKind[]).map((k) => (
            <button key={k} type="button" role="radio" aria-checked={kind === k} className="option" onClick={() => setKind(k)}>
              {KIND_LABEL[k]}
            </button>
          ))}
        </div>
        <textarea className="text-input" placeholder="Decision and rationale" value={text} onChange={(e) => setText(e.target.value)} />
        {kind === "duty_restriction" && (
          <p className="cell-note">
            This becomes effective only after another specialist confirms it. The software never declares anyone unfit.
          </p>
        )}
        <button
          type="button"
          className="button-primary"
          disabled={decisionAction.isBusy}
          onClick={() =>
            decisionAction.run(() => recordDecision(actor.id, record.member.id, kind, text), () => {
              setText("");
              onChanged();
            })
          }
        >
          Record decision
        </button>
        {decisionAction.error && <p className="error">{decisionAction.error}</p>}
        <ul className="finding-list">
          {record.decisions.map(({ decision: d, status, reviews }) => (
            <li key={d.id}>
              <StatusBadge status={STATUS[status].status} label={STATUS[status].label} />
              <span>
                <strong>{KIND_LABEL[d.kind]}</strong> — {d.text}
                <span className="cell-note">
                  {record.names[d.author_id]} · {fmt(d.created_at)}
                  {reviews.map((rv) => ` · ${rv.verdict === "confirm" ? "confirmed" : "rejected"} by ${record.names[rv.reviewer_id]}${rv.comment ? `: ${rv.comment}` : ""}`)}
                </span>
              </span>
            </li>
          ))}
        </ul>
      </section>

      <h2 className="section-title">Notes <span className="section-tag human">specialist judgment</span></h2>
      <section className="panel panel-wide human-panel">
        <textarea className="text-input" placeholder="Medical note" value={note} onChange={(e) => setNote(e.target.value)} />
        <button
          type="button"
          className="button-primary"
          disabled={noteAction.isBusy}
          onClick={() =>
            noteAction.run(() => addNote(actor.id, record.member.id, note), () => {
              setNote("");
              onChanged();
            })
          }
        >
          Add note
        </button>
        {noteAction.error && <p className="error">{noteAction.error}</p>}
        <ul className="plain-list notes-list">
          {record.notes.map((n) => (
            <li key={n.id}>
              {n.text}
              <span className="cell-note">
                {record.names[n.author_id]} · {fmt(n.created_at)}
              </span>
            </li>
          ))}
        </ul>
      </section>
    </>
  );
}
